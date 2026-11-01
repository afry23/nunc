//! Migrationsrunner, kompatibel zur Tabelle `schema_migrations` der Go-App
//! (`nunc-go/internal/db/db.go`). Migrationen 001–005 sind unverändert aus der Go-App übernommen (006 und folgende sind nunc-eigen); neue
//! Migrationen sind ausschließlich additiv (ADR 0002).

use rusqlite::{Connection, params};

use crate::StoreError;

/// Eine SQL-Migration. `version` ist das Präfix des Dateinamens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

macro_rules! migration {
    ($version:literal, $file:literal) => {
        Migration {
            version: $version,
            name: $file,
            sql: include_str!(concat!("../migrations/", $file)),
        }
    };
}

/// Alle Migrationen, aufsteigend nach Version.
pub const MIGRATIONS: &[Migration] = &[
    migration!(1, "001_init.sql"),
    migration!(2, "002_sync.sql"),
    migration!(3, "003_flowtime_ratio.sql"),
    migration!(4, "004_session_sync_device.sql"),
    migration!(5, "005_todoist_mapping.sql"),
    migration!(6, "006_active_timer.sql"),
    migration!(7, "007_todo_crdt.sql"),
    migration!(8, "008_session_crdt.sql"),
    migration!(9, "009_session_kind.sql"),
    migration!(10, "010_manual_is_stopwatch.sql"),
    migration!(11, "011_todo_parent.sql"),
    migration!(12, "012_session_tags.sql"),
    migration!(13, "013_drop_todoist_drive.sql"),
];

/// Wendet alle noch nicht in `schema_migrations` vermerkten Migrationen an.
///
/// Wie in Go zählt das Vorhandensein der Version (nicht das Maximum); unbekannte, höhere
/// Versionen (neuere App) bleiben unberührt. Jede Migration läuft samt Eintrag in einer
/// Transaktion: bei einem Fehler bleibt die Datenbank auf dem Stand davor.
pub fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    apply(conn, MIGRATIONS)
}

fn apply(conn: &mut Connection, migrations: &[Migration]) -> Result<(), StoreError> {
    // Wortgleich zur Go-App (inklusive Tabs), damit auch `sqlite_master` identisch ist.
    conn.execute_batch(concat!(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\n",
        "\t\t\tversion    INTEGER PRIMARY KEY,\n",
        "\t\t\tapplied_at DATETIME NOT NULL DEFAULT (datetime('now'))\n",
        "\t\t)"
    ))?;

    for m in migrations {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            params![m.version],
            |row| row.get(0),
        )?;
        if applied {
            continue;
        }
        let fail = |source| StoreError::Migration {
            version: m.version,
            source,
        };
        let tx = conn.transaction().map_err(fail)?;
        tx.execute_batch(m.sql).map_err(fail)?;
        tx.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            params![m.version],
        )
        .map_err(fail)?;
        tx.commit().map_err(fail)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_match_file_prefixes_and_ascend() {
        for m in MIGRATIONS {
            let prefix: u32 = m
                .name
                .split('_')
                .next()
                .and_then(|p| p.parse().ok())
                .unwrap();
            assert_eq!(prefix, m.version, "{}", m.name);
        }
        assert!(MIGRATIONS.windows(2).all(|w| w[0].version < w[1].version));
    }

    fn count(conn: &Connection, sql: &str) -> i64 {
        conn.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn failing_migration_rolls_back_and_names_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        let bad = [Migration {
            version: 99,
            name: "099_bad.sql",
            sql: "CREATE TABLE half (x); SELECT * FROM missing_table;",
        }];
        let err = apply(&mut conn, &bad).unwrap_err();
        assert!(matches!(err, StoreError::Migration { version: 99, .. }));
        assert_eq!(
            count(
                &conn,
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'half'"
            ),
            0
        );
        assert_eq!(count(&conn, "SELECT COUNT(*) FROM schema_migrations"), 0);
    }

    #[test]
    fn higher_unknown_versions_are_left_alone() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn.execute("INSERT INTO schema_migrations (version) VALUES (42)", [])
            .unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(count(&conn, "SELECT COUNT(*) FROM schema_migrations"), 14);
    }

    #[test]
    fn missing_version_below_max_is_still_applied() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn.execute("DROP TABLE active_timer", []).unwrap();
        conn.execute("DELETE FROM schema_migrations WHERE version = 6", [])
            .unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(
            count(
                &conn,
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'active_timer'"
            ),
            1
        );
    }

    #[test]
    fn removing_todoist_and_drive_cleans_up_old_data() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply(&mut conn, &MIGRATIONS[..12]).unwrap();
        assert_eq!(
            count(
                &conn,
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'todoist_mappings'"
            ),
            1
        );
        for (key, value) in [
            ("sync_provider", "google_drive"),
            ("last_synced_at", "x"),
            ("drive_folder_id", "x"),
            ("todoist.last_sync_at", "x"),
            ("sync.webdav.url", "https://dav.example"),
        ] {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .unwrap();
        }
        migrate(&mut conn).unwrap();
        assert_eq!(
            count(
                &conn,
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'todoist_mappings'"
            ),
            0
        );
        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'sync_provider'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, "none");
        assert_eq!(
            count(
                &conn,
                "SELECT COUNT(*) FROM settings WHERE key IN ('last_synced_at', 'drive_folder_id', 'todoist.last_sync_at')"
            ),
            0
        );
        assert_eq!(
            count(
                &conn,
                "SELECT COUNT(*) FROM settings WHERE key = 'sync.webdav.url'"
            ),
            1
        );
    }
}
