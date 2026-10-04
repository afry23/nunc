//! Steps für die `@integration`-Szenarien: der Server über einen echten Socket (Rohe HTTP/1.1-Anfragen).

use std::sync::Arc;

use cucumber::{given, then, when};
use nunc_api::server::{ApiServer, start};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::ApiWorld;

#[derive(Debug, Default)]
pub struct State {
    pub running: Option<ApiServer>,
    /// Belegter Port eines „anderen Programms“.
    held: Option<TcpListener>,
    status: u16,
    port: u16,
}

#[given(expr = "the API server is running with the token {string}")]
async fn running(world: &mut ApiWorld, token: String) {
    world.token = token.clone();
    let server = start(0, token, world.backend.clone() as Arc<_>)
        .await
        .expect("Server starten");
    world.port = server.port();
    world.server.running = Some(server);
}

/// Schickt `request` und liest die Antwort bis zum Ende; Schreibfehler zählen nicht (der Server darf früh antworten).
async fn exchange(port: u16, head: &str, body_bytes: usize) -> u16 {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("verbinden");
    let _ = stream.write_all(head.as_bytes()).await;
    let chunk = vec![b'a'; 64 * 1024];
    let mut left = body_bytes;
    while left > 0 {
        let size = left.min(chunk.len());
        if stream.write_all(&chunk[..size]).await.is_err() {
            break;
        }
        left -= size;
    }
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer).await;
    let text = String::from_utf8_lossy(&answer);
    text.split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("keine HTTP-Antwort")
}

fn get(path: &str, port: u16, token: Option<&str>) -> String {
    let auth = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}\r\n")
}

#[when(expr = "I request {string} over HTTP with the token")]
async fn request_with_token(world: &mut ApiWorld, path: String) {
    let head = get(&path, world.port, Some(&world.token));
    world.server.status = exchange(world.port, &head, 0).await;
}

#[when(expr = "I request {string} over HTTP without a token")]
async fn request_without_token(world: &mut ApiWorld, path: String) {
    let head = get(&path, world.port, None);
    world.server.status = exchange(world.port, &head, 0).await;
}

#[when(expr = "I post {int} MB to {string} over HTTP with the token")]
async fn post_large(world: &mut ApiWorld, megabytes: usize, path: String) {
    let length = megabytes * 1024 * 1024;
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nContent-Length: {length}\r\n\r\n",
        world.port, world.token
    );
    world.server.status = exchange(world.port, &head, length).await;
}

#[then(expr = "the HTTP status is {int}")]
fn http_status(world: &mut ApiWorld, expected: u16) {
    assert_eq!(world.server.status, expected);
}

#[then("the server is bound to 127.0.0.1")]
fn loopback(world: &mut ApiWorld) {
    assert!(world.running().addr().ip().is_loopback());
    assert_eq!(
        world.running().addr().to_string(),
        format!("127.0.0.1:{}", world.port)
    );
}

#[given("another program holds the port")]
async fn hold_port(world: &mut ApiWorld) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("binden");
    world.server.port = listener.local_addr().expect("Adresse").port();
    world.server.held = Some(listener);
}

#[when("the API server is started on that port")]
async fn start_on_held(world: &mut ApiWorld) {
    let server = start(
        world.server.port,
        "t".into(),
        world.backend.clone() as Arc<_>,
    )
    .await
    .expect("Server starten");
    world.port = server.port();
    world.server.running = Some(server);
}

#[then("the server is running on a different port")]
async fn different_port(world: &mut ApiWorld) {
    assert_ne!(world.running().port(), world.server.port);
    assert!(
        TcpStream::connect(("127.0.0.1", world.running().port()))
            .await
            .is_ok()
    );
}

#[when("the API server is stopped")]
async fn stop(world: &mut ApiWorld) {
    world
        .server
        .running
        .take()
        .expect("Server läuft nicht")
        .stop()
        .await;
}

#[then("connecting to the port fails")]
async fn connect_fails(world: &mut ApiWorld) {
    assert!(TcpStream::connect(("127.0.0.1", world.port)).await.is_err());
}
