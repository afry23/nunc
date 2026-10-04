//! Die eine Datenbankverbindung der App, hinter einem Mutex.
//!
//! Sperrreihenfolge: erst der Timer (`TimerService`), dann die Datenbank. Commands, die nur Daten lesen oder
//! schreiben, sperren nur die Datenbank.

use std::sync::Mutex;

use nunc_store::Connection;

use crate::error::AppError;

pub struct Db(Mutex<Connection>);

impl Db {
    pub fn new(conn: Connection) -> Self {
        Self(Mutex::new(conn))
    }

    /// Wie [`Db::with`] für Aufrufer, die [`StoreError`](nunc_store::StoreError) selbst weitergeben (der Sync).
    pub fn with_store<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, nunc_store::StoreError>,
    ) -> Result<T, nunc_store::StoreError> {
        let mut conn = self.0.lock().map_err(|_| {
            nunc_store::StoreError::BackupDir("Datenbank gesperrt (vergiftete Sperre)".to_owned())
        })?;
        f(&mut conn)
    }

    /// Führt `f` mit exklusivem Zugriff auf die Verbindung aus.
    pub fn with<T, E>(&self, f: impl FnOnce(&mut Connection) -> Result<T, E>) -> Result<T, AppError>
    where
        AppError: From<E>,
    {
        let mut conn = self.0.lock()?;
        Ok(f(&mut conn)?)
    }
}
