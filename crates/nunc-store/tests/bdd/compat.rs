//! Steps für „eine von nunc geschriebene Datenbank bleibt für die Go-App lesbar“ (`features/compat_go.feature`).

use std::collections::BTreeSet;

use cucumber::then;
use rusqlite::{Connection, OpenFlags};

use crate::{StoreWorld, schema};

fn fixture() -> Connection {
    Connection::open_with_flags(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/go-v5.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("Fixture öffnen")
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[then("the Go tables keep the definitions of the Go fixture")]
fn go_tables(world: &mut StoreWorld) {
    let go = schema(&fixture());
    let nunc = schema(world.conn());
    assert!(go.len() >= 6, "Fixture ohne Schema?");
    for (name, sql) in go {
        assert_eq!(nunc.get(&name), Some(&sql), "{name}");
    }
}

#[then("the settings keys are all known to the Go app")]
fn settings_keys(world: &mut StoreWorld) {
    let keys = |conn: &Connection| -> BTreeSet<String> {
        let mut stmt = conn.prepare("SELECT key FROM settings").unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    // Schlüssel, die nur nunc kennt: die Go-App liest `settings` in eine Map und ignoriert sie
    const NUNC_ONLY: [&str; 1] = ["language"];
    let unknown: Vec<_> = keys(world.conn())
        .difference(&keys(&fixture()))
        .filter(|key| !NUNC_ONLY.contains(&key.as_str()))
        .cloned()
        .collect();
    assert!(unknown.is_empty(), "unbekannte Schlüssel: {unknown:?}");
}

#[then("the rows written by nunc use the Go formats")]
fn go_formats(world: &mut StoreWorld) {
    let conn = world.conn();
    let instant = "'????-??-??T??:??:??Z'";
    let checks = [
        (
            "todos.updated_at",
            format!("SELECT COUNT(*) FROM todos WHERE updated_at NOT GLOB {instant}"),
        ),
        (
            "todos.created_at",
            "SELECT COUNT(*) FROM todos WHERE created_at IS NOT NULL AND created_at NOT GLOB '????-??-??'"
                .to_owned(),
        ),
        (
            "todos.sync_id",
            "SELECT COUNT(*) FROM todos WHERE sync_id IS NULL OR sync_id = ''".to_owned(),
        ),
        (
            "todos.project_tags",
            "SELECT COUNT(*) FROM todos WHERE json_valid(project_tags) = 0 OR json_valid(context_tags) = 0"
                .to_owned(),
        ),
        (
            "sessions.started_at/ended_at",
            format!(
                "SELECT COUNT(*) FROM sessions WHERE started_at NOT GLOB {instant} OR ended_at NOT GLOB {instant}"
            ),
        ),
        (
            "sessions.sync_device_id",
            "SELECT COUNT(*) FROM sessions WHERE sync_device_id IS NULL OR sync_device_id = ''"
                .to_owned(),
        ),
    ];
    for (what, sql) in checks {
        assert_eq!(count(conn, &sql), 0, "{what}");
    }
    // es gibt überhaupt Zeilen, die geprüft wurden
    assert!(count(conn, "SELECT COUNT(*) FROM todos") > 0);
    assert!(count(conn, "SELECT COUNT(*) FROM sessions") > 0);
}
