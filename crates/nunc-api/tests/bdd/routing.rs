//! Steps für die `@domain`-Szenarien: Anfrage zusammenstellen, `route` aufrufen, Antwort prüfen.

use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use nunc_api::{ApiConfig, Method, Request, route};
use serde_json::Value;

use crate::{ApiWorld, make_todo};

fn docstring(step: &Step) -> String {
    step.docstring
        .as_deref()
        .expect("Docstring fehlt")
        .trim_matches('\n')
        .to_owned()
}

#[given(expr = "the API token is {string} and the port is {int}")]
fn configure(world: &mut ApiWorld, token: String, port: u16) {
    world.token = token;
    world.port = port;
}

#[given("the app has the todos:")]
fn todos(world: &mut ApiWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    let todos = table.rows[1..]
        .iter()
        .map(|row| {
            make_todo(
                row[0].parse().expect("id"),
                &row[1],
                row[2].parse().expect("pomodoros"),
                row[3].parse().expect("position"),
            )
        })
        .collect();
    *world.backend.todos.lock().unwrap() = todos;
}

#[given("the request has no Authorization header")]
fn no_authorization(world: &mut ApiWorld) {
    world.headers.insert("authorization".into(), None);
}

#[given("the request has no Host header")]
fn no_host(world: &mut ApiWorld) {
    world.headers.insert("host".into(), None);
}

#[given(expr = "the request header {string} is {string}")]
fn header(world: &mut ApiWorld, name: String, value: String) {
    world.headers.insert(name.to_ascii_lowercase(), Some(value));
}

#[given(expr = "the app fails with {string}")]
fn app_fails(world: &mut ApiWorld, message: String) {
    *world.backend.fail.lock().unwrap() = Some(message);
}

fn send(world: &mut ApiWorld, line: &str, body: Vec<u8>) {
    let (method, target) = line.split_once(' ').expect("Methode und Pfad");
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut headers = vec![
        ("host".to_owned(), format!("127.0.0.1:{}", world.port)),
        (
            "authorization".to_owned(),
            format!("Bearer {}", world.token),
        ),
    ];
    for (name, value) in &world.headers {
        headers.retain(|(key, _)| key != name);
        if let Some(value) = value {
            headers.push((name.clone(), value.clone()));
        }
    }
    let request = Request {
        method: Method::parse(method),
        path: path.to_owned(),
        query: query.to_owned(),
        headers,
        body,
    };
    let config = ApiConfig {
        token: world.token.clone(),
        port: world.port,
    };
    world.response = Some(route(&request, &config, &*world.backend));
}

#[when(expr = "I send {string}")]
fn send_plain(world: &mut ApiWorld, line: String) {
    send(world, &line, Vec::new());
}

#[when(expr = "I send {string} with the body:")]
fn send_body(world: &mut ApiWorld, line: String, step: &Step) {
    send(world, &line, docstring(step).into_bytes());
}

#[when(expr = "I send {string} with the invalid UTF-8 bytes")]
fn send_invalid(world: &mut ApiWorld, line: String) {
    send(world, &line, vec![0xff, 0xfe, 0x41]);
}

fn response(world: &ApiWorld) -> &nunc_api::Response {
    world.response.as_ref().expect("keine Antwort")
}

fn json(world: &ApiWorld) -> Value {
    serde_json::from_slice(&response(world).body).expect("Antwort ist kein JSON")
}

#[then(expr = "the response status is {int}")]
fn status(world: &mut ApiWorld, expected: u16) {
    assert_eq!(response(world).status, expected);
}

#[then(expr = "the error code is {string}")]
fn error_code(world: &mut ApiWorld, expected: String) {
    assert_eq!(json(world)["code"], expected);
}

#[then(expr = "the response has the header {string} with {string}")]
fn has_header(world: &mut ApiWorld, name: String, value: String) {
    assert_eq!(response(world).header(&name), Some(value.as_str()));
}

#[then(expr = "the response content type is {string}")]
fn content_type(world: &mut ApiWorld, expected: String) {
    assert_eq!(
        response(world).header("content-type"),
        Some(expected.as_str())
    );
}

#[then(expr = "the response body does not contain {string}")]
fn body_lacks(world: &mut ApiWorld, text: String) {
    let body = String::from_utf8_lossy(&response(world).body).into_owned();
    assert!(!body.contains(&text), "{body}");
}

#[then("the response body is:")]
fn body_is(world: &mut ApiWorld, step: &Step) {
    let body = String::from_utf8(response(world).body.clone()).expect("UTF-8");
    assert_eq!(body, format!("{}\n", docstring(step)));
}

#[then(regex = r#"^the response JSON has "([^"]*)" equal to (.+)$"#)]
fn json_field(world: &mut ApiWorld, field: String, expected: String) {
    let expected: Value = serde_json::from_str(&expected).expect("Erwartung ist kein JSON");
    assert_eq!(json(world)[field.as_str()], expected);
}

#[then(expr = "the response JSON has a non-empty {string}")]
fn json_non_empty(world: &mut ApiWorld, field: String) {
    assert!(
        json(world)[field.as_str()]
            .as_str()
            .is_some_and(|text| !text.is_empty())
    );
}

fn as_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Die Todos der Antwort: eine Liste (`GET`) oder das Feld `todos` (`POST`).
fn todos_of(value: Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items,
        Value::Object(mut object) => match object.remove("todos") {
            Some(Value::Array(items)) => items,
            _ => panic!("keine Todo-Liste in der Antwort"),
        },
        other => panic!("unerwartete Antwort {other}"),
    }
}

#[then("the response todos are:")]
fn response_todos(world: &mut ApiWorld, step: &Step) {
    let table = &step.table.as_ref().expect("Tabelle fehlt").rows;
    let items = todos_of(json(world));
    assert_eq!(items.len(), table.len() - 1, "Anzahl Todos");
    for (row, item) in table[1..].iter().zip(&items) {
        for (field, expected) in table[0].iter().zip(row) {
            assert_eq!(&as_text(&item[field.as_str()]), expected, "{field}");
        }
    }
}

#[then(expr = "the response todo ids are {string}")]
fn response_ids(world: &mut ApiWorld, expected: String) {
    let ids: Vec<String> = todos_of(json(world))
        .iter()
        .map(|item| as_text(&item["id"]))
        .collect();
    assert_eq!(ids.join(" "), expected);
}

#[then("the app's todo lines are:")]
fn app_lines(world: &mut ApiWorld, step: &Step) {
    assert_eq!(world.backend.lines().join("\n"), docstring(step));
}
