//! Steuerung der Synchronisierung: wann ein Lauf startet, dass immer nur einer läuft, und was die Oberfläche
//! davon erfährt (Go: `WebDAVSyncService.Sync`, `App.triggerWebDAVSyncAsync`, `ServiceShutdown`).
//!
//! Auslöser ([`Trigger`]): App-Start, Ende oder Abbruch einer Session, Todo-Änderung (entprellt), Beenden
//! (Desktop, begrenzt) sowie Rückkehr aus dem und Wechsel in den Hintergrund (Mobil). Läufe sind serialisiert:
//! ein zweiter Aufruf wartet, bis der erste fertig ist.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nunc_core::model::{SyncResult, SyncState, SyncStatus};
use tokio::runtime::Handle;

use crate::SyncError;

/// Ergebnis eines Laufs: `None`, wenn nichts zu tun war (kein Anbieter gewählt, Sync pausiert); sonst, was
/// übernommen wurde, und der Zeitpunkt (RFC 3339) für „zuletzt synchronisiert“.
pub type RunOutcome = Result<Option<(SyncResult, String)>, SyncError>;

pub type RunFuture = Pin<Box<dyn Future<Output = RunOutcome> + Send>>;

/// Führt einen Sync-Lauf aus (WebDAV, später Google Drive).
pub trait SyncRunner: Send + Sync {
    fn run(&self) -> RunFuture;
}

/// Was die Oberfläche mitbekommt: `sync:start`, `sync:status`, `sync:complete`.
pub trait SyncObserver: Send + Sync {
    fn started(&self);
    fn status(&self, status: &SyncStatus);
    fn completed(&self, result: &SyncResult);
}

/// Anlass für einen Lauf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// App-Start.
    Start,
    /// Eine Session wurde beendet oder abgebrochen.
    SessionFinished,
    /// Ein Todo wurde angelegt, geändert, abgehakt oder gelöscht (entprellt).
    TodoChanged,
    /// Die App kehrt in den Vordergrund zurück (nur Mobil).
    Resume,
    /// Die App geht in den Hintergrund (nur Mobil).
    Background,
    /// Ein Konto wurde soeben verbunden: erster Abgleich.
    Connected,
}

impl Trigger {
    /// Gilt der Auslöser auf dieser Plattform? Desktop-Apps laufen im Hintergrund weiter und synchronisieren
    /// beim Beenden; Mobilgeräte frieren sie ein und beenden sie ohne Vorwarnung.
    pub fn applies(self, mobile: bool) -> bool {
        match self {
            Self::Resume | Self::Background => mobile,
            Self::Start | Self::SessionFinished | Self::TodoChanged | Self::Connected => true,
        }
    }
}

/// Zeiten der Steuerung; die Standardwerte gelten in der App, Tests setzen kurze.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// Ruhezeit nach der letzten Todo-Änderung, bevor synchronisiert wird.
    pub debounce: Duration,
    /// So lange darf der Sync beim Beenden das Schließen höchstens verzögern.
    pub quit_limit: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            debounce: Duration::from_secs(3),
            quit_limit: Duration::from_secs(5),
        }
    }
}

pub struct SyncCoordinator {
    runner: Arc<dyn SyncRunner>,
    observer: Arc<dyn SyncObserver>,
    runtime: Handle,
    mobile: bool,
    timing: Timing,
    status: Mutex<SyncStatus>,
    /// Serialisiert die Läufe.
    gate: tokio::sync::Mutex<()>,
    /// Zählt Todo-Änderungen: nur die neueste wartende läuft nach der Ruhezeit los.
    debounce_generation: AtomicU64,
}

impl SyncCoordinator {
    pub fn new(
        runner: Arc<dyn SyncRunner>,
        observer: Arc<dyn SyncObserver>,
        runtime: Handle,
        mobile: bool,
        timing: Timing,
        last_sync_at: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runner,
            observer,
            runtime,
            mobile,
            timing,
            status: Mutex::new(SyncStatus::idle(last_sync_at)),
            gate: tokio::sync::Mutex::new(()),
            debounce_generation: AtomicU64::new(0),
        })
    }

    pub fn status(&self) -> SyncStatus {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_else(|_| SyncStatus::idle(None))
    }

    /// Ersetzt den Zeitpunkt des letzten Syncs, wenn ein anderer Anbieter gewählt wird (jeder hat seinen eigenen).
    pub fn set_last_sync_at(&self, last_sync_at: Option<String>) {
        let mut status = self.status();
        if status.state == SyncState::Syncing {
            return;
        }
        status.last_sync_at = last_sync_at;
        status.error_message = None;
        status.state = SyncState::Idle;
        self.set_status(status);
    }

    fn set_status(&self, status: SyncStatus) {
        if let Ok(mut current) = self.status.lock() {
            *current = status.clone();
        }
        self.observer.status(&status);
    }

    /// Ein Lauf, sobald der vorige fertig ist. Fehler stehen im Status und werden zusätzlich zurückgegeben
    /// (der manuelle Aufruf zeigt sie an). `Ok(None)`: nichts zu synchronisieren.
    pub async fn sync_now(&self) -> Result<Option<SyncResult>, SyncError> {
        let _turn = self.gate.lock().await;
        let last = self.status().last_sync_at;
        self.observer.started();
        self.set_status(SyncStatus {
            state: SyncState::Syncing,
            last_sync_at: last.clone(),
            error_message: None,
        });
        match self.runner.run().await {
            Ok(Some((result, finished_at))) => {
                self.set_status(SyncStatus::idle(Some(finished_at)));
                self.observer.completed(&result);
                Ok(Some(result))
            }
            Ok(None) => {
                self.set_status(SyncStatus::idle(last));
                Ok(None)
            }
            Err(error) => {
                self.set_status(SyncStatus {
                    state: SyncState::Error,
                    last_sync_at: last,
                    error_message: Some(error.to_string()),
                });
                Err(error)
            }
        }
    }

    /// Meldet einen Anlass. Kehrt sofort zurück; der Lauf geschieht im Hintergrund. Fehler stehen im Status.
    pub fn trigger(self: &Arc<Self>, trigger: Trigger) {
        if !trigger.applies(self.mobile) {
            return;
        }
        let this = self.clone();
        // Eine neue Todo-Änderung ersetzt die wartende (ein bereits laufender Sync wird nie abgebrochen).
        let wait = (trigger == Trigger::TodoChanged).then(|| {
            let generation = self.debounce_generation.fetch_add(1, Ordering::SeqCst) + 1;
            (generation, self.timing.debounce)
        });
        self.runtime.spawn(async move {
            if let Some((generation, wait)) = wait {
                tokio::time::sleep(wait).await;
                if this.debounce_generation.load(Ordering::SeqCst) != generation {
                    return;
                }
            }
            // der Fehler steht im Status
            let _ = this.sync_now().await;
        });
    }

    /// Letzter Lauf beim Beenden (Desktop): wartet höchstens [`Timing::quit_limit`]. Eine wartende, noch nicht
    /// gestartete Todo-Änderung ist darin enthalten.
    pub async fn sync_on_quit(&self) {
        // eine wartende Todo-Änderung ist im Lauf unten enthalten
        self.debounce_generation.fetch_add(1, Ordering::SeqCst);
        match tokio::time::timeout(self.timing.quit_limit, self.sync_now()).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => log::warn!("Sync beim Beenden: {error}"),
            Err(_) => log::warn!(
                "Sync beim Beenden nach {:?} abgebrochen",
                self.timing.quit_limit
            ),
        }
    }
}
