//! realm-icons: Realm's clay mark with any glyph on it, served by URL, plus a
//! web UI for picking one. `realm-icons` serves; `realm-icons healthcheck`
//! probes a running server and exits 0/1 (the container has no shell or curl).

mod api;
mod glyph;
mod healthcheck;
mod mark;
mod render;
mod web;

use std::net::SocketAddr;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => serve(),
        Some("healthcheck") => healthcheck::run(port()),
        Some(other) => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
}

fn port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000)
}

#[tokio::main]
async fn serve() {
    let addr = SocketAddr::from(([0, 0, 0, 0], port()));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("bind {addr}: {e}"));
    eprintln!("realm-icons listening on {addr}");
    axum::serve(listener, api::router())
        .with_graceful_shutdown(shutdown())
        .await
        .expect("server error");
}

/// Resolves on SIGINT or SIGTERM so `docker stop` ends the process promptly.
async fn shutdown() {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("install SIGTERM handler");
    tokio::select! {
        _ = ctrl_c => {}
        _ = term.recv() => {}
    }
}
