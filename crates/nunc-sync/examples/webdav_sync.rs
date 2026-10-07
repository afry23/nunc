//! Ein Sync-Lauf von nunc gegen einen echten WebDAV-Server, für den Interop-Test mit der Go-App
//! (`tests/interop/webdav-go-nunc.sh`).
//!
//! `cargo run -p nunc-sync --example webdav_sync -- <datenbank> <url> <benutzer> <passwort> <remote-pfad> [todo]`
//! Mit `todo` wird vor dem Sync ein Todo mit diesem Text angelegt. Ausgabe: JSON mit dem Ergebnis.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use nunc_core::clock::Clock;
use nunc_core::model::{SyncProvider, WebDavAuth, WebDavConfig};
use nunc_store::{Applied, Connection, StoreError};
use nunc_sync::{SyncSetup, SyncStore, WebDavClient, sync_webdav};

struct Store {
    conn: Mutex<Connection>,
}

fn random_id() -> String {
    format!("{:016x}", RandomState::new().build_hasher().finish())
}

impl SyncStore for Store {
    fn local_state(&self) -> Result<nunc_core::crdt::SyncState, StoreError> {
        let conn = self.conn.lock().unwrap();
        nunc_store::local_state(&conn, random_id)
    }

    fn setup(&self) -> Result<SyncSetup, StoreError> {
        Ok(SyncSetup {
            provider: SyncProvider::Webdav,
            enabled: true,
            webdav: WebDavConfig::default(),
        })
    }

    fn record_sync(&self, _: SyncProvider, _: DateTime<Utc>) -> Result<(), StoreError> {
        Ok(())
    }

    fn record_drive_folder(&self, _: &str) -> Result<(), StoreError> {
        Ok(())
    }

    fn apply_state(&self, merged: &nunc_core::crdt::SyncState) -> Result<Applied, StoreError> {
        nunc_store::apply_state(&mut self.conn.lock().unwrap(), merged)
    }
}

struct Now;

impl Clock for Now {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [db, url, user, password, remote_path, rest @ ..] = args.as_slice() else {
        eprintln!(
            "Aufruf: webdav_sync <datenbank> <url> <benutzer> <passwort> <remote-pfad> [todo]"
        );
        std::process::exit(2);
    };
    let conn = nunc_store::open(std::path::Path::new(db)).expect("Datenbank öffnen");
    nunc_store::device_id(&conn, random_id).expect("Geräte-ID");
    if let Some(text) = rest.first() {
        nunc_store::add_todo(&conn, text, &random_id(), Utc::now()).expect("Todo anlegen");
    }
    let store = Store {
        conn: Mutex::new(conn),
    };
    let client = WebDavClient::new(
        WebDavConfig {
            url: url.clone(),
            auth_method: WebDavAuth::Basic,
            username: user.clone(),
            remote_path: remote_path.clone(),
        },
        password.clone(),
    )
    .expect("Client");
    match sync_webdav(&client, &store, &Now).await {
        Ok(result) => println!("{}", serde_json::to_string(&result).expect("JSON")),
        Err(error) => {
            eprintln!("FEHLER: {error}");
            std::process::exit(1);
        }
    }
}
