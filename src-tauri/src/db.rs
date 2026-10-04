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

    /// Führt `f` mit exklusivem Zugriff auf die Verbindung aus.
    pub fn with<T, E>(&self, f: impl FnOnce(&mut Connection) -> Result<T, E>) -> Result<T, AppError>
    where
        AppError: From<E>,
    {
        let mut conn = self.0.lock()?;
        Ok(f(&mut conn)?)
    }
}
