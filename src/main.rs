mod mock;
mod snapshot;

use std::net::SocketAddr;

use axum::routing::get;
use axum::{Json, Router};
use clap::Parser;
use serde_json::{Value, json};

#[derive(Parser)]
#[command(name = "portdoc", version, about = "Local dev server control panel")]
struct Cli {
    /// Port for the local UI server
    #[arg(long, default_value_t = 7788)]
    port: u16,

    /// Print the dev snapshot as JSON and exit
    #[arg(long)]
    json: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.json {
        match serde_json::to_string_pretty(&mock::snapshot()) {
            Ok(out) => println!("{out}"),
            Err(err) => {
                eprintln!("failed to serialize snapshot: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    let app = Router::new()
        .route("/", get(|| async { "PortDoc scaffold" }))
        .route("/api/health", get(health))
        .route("/api/snapshot", get(api_snapshot));

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    println!("PortDoc listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn api_snapshot() -> Json<snapshot::DevSnapshot> {
    Json(mock::snapshot())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
}
