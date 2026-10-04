//! Lokale HTTP-Schnittstelle (ADR 0013): startet und stoppt den Server von `nunc-api` gemäß den Einstellungen,
//! verwaltet das Token im Tresor und verbindet die Schnittstelle über [`AppBackend`] mit den Todos der App.

use std::sync::Arc;

use nunc_api::server::{ApiServer, start};
use nunc_api::token::generate_token;
use nunc_api::{ApiError, ImportOutcome, TodoBackend};
use nunc_core::model::Todo;
use nunc_platform::SecretStore;
use nunc_store::StoreError;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;

use crate::commands::api::ApiStatus;
use crate::error::AppError;
use crate::events::{self, AppReload};
use crate::state::AppState;

/// Eintrag im Tresor: Dienst `nunc`, Konto `local-api`.
const SECRET_SERVICE: &str = "nunc";
const SECRET_ACCOUNT: &str = "local-api";

struct Running {
    server: ApiServer,
    /// Der gewünschte Port, mit dem er gestartet wurde.
    wanted_port: u32,
}

#[derive(Default)]
struct Inner {
    running: Option<Running>,
    error: Option<String>,
}

pub struct ApiService {
    secrets: Arc<dyn SecretStore>,
    backend: Arc<dyn TodoBackend>,
    inner: Mutex<Inner>,
}

impl ApiService {
    pub fn new(secrets: Arc<dyn SecretStore>, backend: Arc<dyn TodoBackend>) -> Self {
        Self {
            secrets,
            backend,
            inner: Mutex::new(Inner::default()),
        }
    }

    /// Das Token; beim ersten Zugriff wird eines erzeugt und im Tresor abgelegt.
    pub fn token(&self) -> Result<String, AppError> {
        if let Some(token) = self.secrets.get(SECRET_SERVICE, SECRET_ACCOUNT)?
            && !token.is_empty()
        {
            return Ok(token);
        }
        self.new_token()
    }

    fn new_token(&self) -> Result<String, AppError> {
        let token = generate_token().map_err(|error| {
            AppError::internal(format!("Zufallszahlen nicht verfügbar: {error}"))
        })?;
        self.secrets.set(SECRET_SERVICE, SECRET_ACCOUNT, &token)?;
        Ok(token)
    }

    /// Bringt den Server in den gewünschten Zustand (idempotent: derselbe Wunsch startet nichts neu).
    pub async fn apply(&self, enabled: bool, port: u32) -> ApiStatus {
        let mut inner = self.inner.lock().await;
        if enabled
            && inner
                .running
                .as_ref()
                .is_some_and(|running| running.wanted_port == port)
        {
            return status(&inner);
        }
        Self::stop(&mut inner).await;
        if enabled {
            self.launch(&mut inner, port).await;
        }
        status(&inner)
    }

    /// Erzeugt ein neues Token (das alte gilt sofort nicht mehr) und startet einen laufenden Server damit neu.
    pub async fn regenerate(&self) -> Result<String, AppError> {
        let mut inner = self.inner.lock().await;
        let token = self.new_token()?;
        if let Some(wanted_port) = inner.running.as_ref().map(|running| running.wanted_port) {
            Self::stop(&mut inner).await;
            self.launch(&mut inner, wanted_port).await;
        }
        Ok(token)
    }

    pub async fn status(&self) -> ApiStatus {
        status(&*self.inner.lock().await)
    }

    async fn stop(inner: &mut Inner) {
        if let Some(running) = inner.running.take() {
            running.server.stop().await;
            log::info!("Lokale Schnittstelle beendet");
        }
        inner.error = None;
    }

    async fn launch(&self, inner: &mut Inner, wanted_port: u32) {
        let result = match (u16::try_from(wanted_port), self.token()) {
            (Ok(port), Ok(token)) => start(port, token, self.backend.clone())
                .await
                .map_err(|error| format!("Port {port} nicht nutzbar: {error}")),
            (Err(_), _) => Err(format!("Ungültiger Port {wanted_port}")),
            (_, Err(error)) => Err(error.message),
        };
        match result {
            Ok(server) => {
                log::info!(
                    "Lokale Schnittstelle lauscht auf 127.0.0.1:{}",
                    server.port()
                );
                inner.running = Some(Running {
                    server,
                    wanted_port,
                });
            }
            Err(message) => {
                log::warn!("Lokale Schnittstelle nicht gestartet: {message}");
                inner.error = Some(message);
            }
        }
    }
}

fn status(inner: &Inner) -> ApiStatus {
    ApiStatus {
        running: inner.running.is_some(),
        port: inner.running.as_ref().map(|r| u32::from(r.server.port())),
        error: inner.error.clone(),
    }
}

/// Die Todos der App für die Schnittstelle. Hält nur den `AppHandle`; der `AppState` wird je Aufruf geholt.
pub struct AppBackend<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> AppBackend<R> {
    fn state(&self) -> Result<tauri::State<'_, AppState>, ApiError> {
        self.0
            .try_state::<AppState>()
            .ok_or_else(|| ApiError::Internal("App nicht bereit".to_owned()))
    }

    /// Die Oberfläche lädt neu, der Sync wird angestoßen.
    fn changed(state: &AppState) {
        state.todos_changed();
        events::emit(&*state.events, &AppReload);
    }
}

fn internal(error: StoreError) -> ApiError {
    // Einzelheiten ins Protokoll, nicht in die Antwort
    log::error!("Schnittstelle: Datenbankfehler: {error}");
    ApiError::Internal(error.to_string())
}

impl<R: Runtime> TodoBackend for AppBackend<R> {
    fn list(&self) -> Result<Vec<Todo>, ApiError> {
        let state = self.state()?;
        state
            .db
            .with_store(|conn| nunc_store::list_todos(conn))
            .map_err(internal)
    }

    fn import(&self, text: &str) -> Result<ImportOutcome, ApiError> {
        let state = self.state()?;
        let now = state.clock.now();
        let result = state
            .db
            .with_store(|conn| {
                nunc_store::import_todotxt(conn, text, now, &mut || {
                    uuid::Uuid::new_v4().to_string()
                })
            })
            .map_err(internal)?;
        if !result.added.is_empty() {
            Self::changed(&state);
        }
        Ok(ImportOutcome {
            added: result.added,
            duplicates: result.duplicates,
        })
    }

    fn set_completed(&self, id: i32, completed: bool) -> Result<Todo, ApiError> {
        let state = self.state()?;
        let (today, now) = (state.today(), state.clock.now());
        let todo = state
            .db
            .with_store(|conn| nunc_store::set_todo_completed(conn, id, completed, today, now))
            .map_err(|error| match error {
                StoreError::NotFound { .. } => ApiError::NotFound(id),
                other => internal(other),
            })?;
        Self::changed(&state);
        Ok(todo)
    }
}

#[cfg(test)]
mod tests {
    use nunc_platform::MemorySecretStore;

    use super::*;

    struct Empty;

    impl TodoBackend for Empty {
        fn list(&self) -> Result<Vec<Todo>, ApiError> {
            Ok(Vec::new())
        }
        fn import(&self, _text: &str) -> Result<ImportOutcome, ApiError> {
            Ok(ImportOutcome::default())
        }
        fn set_completed(&self, id: i32, _completed: bool) -> Result<Todo, ApiError> {
            Err(ApiError::NotFound(id))
        }
    }

    fn service() -> ApiService {
        ApiService::new(Arc::new(MemorySecretStore::default()), Arc::new(Empty))
    }

    /// Freier Port, den niemand hält (kurz gebunden und wieder frei gegeben).
    fn free_port() -> u32 {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        u32::from(listener.local_addr().unwrap().port())
    }

    #[test]
    fn the_token_is_created_once_and_then_kept() {
        let service = service();
        let first = service.token().unwrap();
        assert_eq!(first.len(), 43);
        assert_eq!(service.token().unwrap(), first);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn regenerating_replaces_the_token() {
        let service = service();
        let first = service.token().unwrap();
        let second = service.regenerate().await.unwrap();
        assert_ne!(first, second);
        assert_eq!(service.token().unwrap(), second);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn the_server_follows_the_settings() {
        let service = service();
        assert_eq!(service.status().await, ApiStatus::default());

        let port = free_port();
        let on = service.apply(true, port).await;
        assert!(on.running && on.error.is_none());
        assert_eq!(on.port, Some(port));

        // derselbe Wunsch startet nichts neu
        assert_eq!(service.apply(true, port).await, on);

        // ein anderer Port stellt um
        let other = free_port();
        assert_eq!(service.apply(true, other).await.port, Some(other));

        let off = service.apply(false, other).await;
        assert_eq!(off, ApiStatus::default());
        assert!(
            tokio::net::TcpStream::connect(("127.0.0.1", other as u16))
                .await
                .is_err()
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn an_invalid_port_is_reported_instead_of_started() {
        let status = service().apply(true, 70_000).await;
        assert!(!status.running);
        assert!(status.error.is_some_and(|text| text.contains("70000")));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn regenerating_restarts_a_running_server_on_its_wanted_port() {
        let service = service();
        let port = free_port();
        service.apply(true, port).await;
        service.regenerate().await.unwrap();
        let status = service.status().await;
        assert!(status.running);
        assert_eq!(status.port, Some(port));
    }
}
