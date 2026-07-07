mod adapter;
mod hint;
mod label;
mod probe;
mod project;
mod snapshot;

use std::net::SocketAddr;

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use clap::Parser;
use rust_embed::RustEmbed;
use serde_json::{Value, json};

#[derive(Parser)]
#[command(name = "portdoc", version, about = "Local dev server control panel")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Port for the local UI server
    #[arg(long, global = true, default_value_t = 7788)]
    port: u16,

    /// Don't open the browser after the server starts
    #[arg(long, global = true)]
    no_open: bool,

    /// Print the dev snapshot as JSON and exit
    #[arg(long, global = true)]
    json: bool,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Launch the local dashboard (same as the default command)
    Ui,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.json {
        let snapshot = adapter::live_snapshot().unwrap_or_else(|err| {
            eprintln!("probe failed: {err}");
            std::process::exit(1);
        });
        match serde_json::to_string_pretty(&snapshot) {
            Ok(out) => println!("{out}"),
            Err(err) => {
                eprintln!("failed to serialize snapshot: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    // `portdoc ui` is an explicit alias of the default launch behavior
    match cli.command {
        Some(Command::Ui) | None => {}
    }

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/snapshot", get(api_snapshot))
        .fallback(static_handler);

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    let url = format!("http://{addr}");
    println!("PortDoc listening on {url}");

    if !cli.no_open
        && let Err(err) = open::that_detached(&url)
    {
        eprintln!("warning: could not open browser: {err}");
    }

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("server error: {err}");
        std::process::exit(1);
    }
}

/// Embedded production build of the web UI. Debug builds read `web/dist`
/// from disk at runtime; release builds embed the files in the binary.
#[derive(RustEmbed)]
#[folder = "web/dist"]
struct Assets;

async fn static_handler(uri: Uri) -> Response {
    if uri.path() == "/api" || uri.path().starts_with("/api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = uri.path().trim_start_matches('/');
    if !path.is_empty()
        && let Some(file) = Assets::get(path)
    {
        return asset_response(path, file);
    }

    // SPA fallback: any non-API, non-asset path gets the app shell
    match Assets::get("index.html") {
        Some(index) => asset_response("index.html", index),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "UI not built - run `npm run build` in web/ first",
        )
            .into_response(),
    }
}

fn asset_response(path: &str, file: rust_embed::EmbeddedFile) -> Response {
    ([(header::CONTENT_TYPE, content_type(path))], file.data).into_response()
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, ext)| ext) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("json") | Some("map") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

/// Every request re-probes; the blocking /proc walk stays off the async
/// runtime so concurrent refreshes don't stall other requests.
async fn api_snapshot() -> Response {
    match tokio::task::spawn_blocking(adapter::live_snapshot).await {
        Ok(Ok(snapshot)) => Json(snapshot).into_response(),
        Ok(Err(err)) => snapshot_error(format!("probe failed: {err}")),
        Err(err) => snapshot_error(format!("probe task failed: {err}")),
    }
}

fn snapshot_error(message: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": message })),
    )
        .into_response()
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
}
