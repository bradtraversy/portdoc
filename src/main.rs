mod action;
mod adapter;
mod advanced;
mod config;
mod docker;
mod exec;
mod facts;
mod hint;
mod label;
mod probe;
mod project;
mod snapshot;

use std::net::SocketAddr;
use std::time::Duration;

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
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
        .route("/api/sockets", get(api_sockets))
        .route("/api/config", get(api_config))
        .route("/api/ignore", post(api_ignore))
        .route("/api/stop", post(api_stop))
        .route("/api/reveal", post(api_reveal))
        .route("/api/open", post(api_open))
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
    api_error(StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// Raw pre-merge sockets and unknown-owner diagnostics for the Advanced tab;
/// same blocking-probe treatment as /api/snapshot.
async fn api_sockets() -> Response {
    match tokio::task::spawn_blocking(advanced::live_sockets).await {
        Ok(Ok(sockets)) => Json(sockets).into_response(),
        Ok(Err(err)) => snapshot_error(format!("probe failed: {err}")),
        Err(err) => snapshot_error(format!("probe task failed: {err}")),
    }
}

fn api_error(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

async fn api_config() -> Response {
    match config::config_path() {
        Some(path) => Json(config::load(&path)).into_response(),
        None => config_dir_error(),
    }
}

#[derive(serde::Deserialize)]
struct IgnoreRequest {
    service_id: String,
    ignored: bool,
}

/// Read-modify-write under a lock so concurrent toggles cannot drop each other.
static CONFIG_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn api_ignore(Json(request): Json<IgnoreRequest>) -> Response {
    if request.service_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "no service_id provided".into());
    }
    let Some(path) = config::config_path() else {
        return config_dir_error();
    };
    let _guard = CONFIG_LOCK.lock().await;
    let mut cfg = config::load(&path);
    if cfg.set_ignored(&request.service_id, request.ignored)
        && let Err(err) = config::save(&path, &cfg)
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not save config: {err}"),
        );
    }
    Json(cfg).into_response()
}

fn config_dir_error() -> Response {
    api_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "no config directory on this platform".into(),
    )
}

#[derive(serde::Deserialize)]
struct StopRequest {
    service_id: String,
    pid: u32,
    #[serde(default)]
    force: bool,
}

async fn api_stop(Json(request): Json<StopRequest>) -> Response {
    match tokio::task::spawn_blocking(move || stop_service(request)).await {
        Ok(response) => response,
        Err(err) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("stop task failed: {err}"),
        ),
    }
}

/// The safety contract: no signal is ever sent to a pid that does not,
/// right now, own the claimed service.
fn stop_service(request: StopRequest) -> Response {
    if request.pid == std::process::id() {
        return api_error(StatusCode::FORBIDDEN, "PortDoc will not stop itself".into());
    }

    let snapshot = match adapter::live_snapshot() {
        Ok(snapshot) => snapshot,
        Err(err) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("probe failed: {err}"),
            );
        }
    };
    let Some(service) = snapshot
        .services
        .iter()
        .find(|s| s.id == request.service_id)
    else {
        return api_error(
            StatusCode::CONFLICT,
            "service not found - refresh and retry".into(),
        );
    };
    match service.pid {
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "service has no known owner pid".into(),
            );
        }
        Some(pid) if pid != request.pid => {
            return api_error(
                StatusCode::CONFLICT,
                "service changed - refresh and retry".into(),
            );
        }
        Some(_) => {}
    }

    let port = service.port;
    match action::terminate(request.pid, request.force) {
        Ok(()) => {}
        // died between the probe and the signal: that is a release
        Err(action::StopError::NoSuchProcess) => return stop_outcome(true),
        Err(err @ action::StopError::NotPermitted) => {
            return api_error(StatusCode::FORBIDDEN, err.to_string());
        }
        Err(err) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }

    let released = action::wait_released(
        || still_listening(port, request.pid),
        6,
        Duration::from_millis(500),
    );
    stop_outcome(released)
}

fn stop_outcome(released: bool) -> Response {
    let outcome = if released {
        "released"
    } else {
        "still_listening"
    };
    Json(json!({ "outcome": outcome })).into_response()
}

#[derive(serde::Deserialize)]
struct RevealRequest {
    path: String,
}

/// Snapshot paths render with a shortened home ("~/Code/x"); expand the
/// leading tilde back before touching the filesystem.
fn expand_home(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    std::path::PathBuf::from(path)
}

/// Opens a folder in the OS file manager. A browser cannot, so the server
/// does it - but only for a path that resolves to an existing directory.
async fn api_reveal(Json(request): Json<RevealRequest>) -> Response {
    let path = expand_home(&request.path);
    let is_dir = std::fs::metadata(&path).ok().map(|m| m.is_dir());
    if let Err(message) = validate_reveal_path(&path, is_dir) {
        return api_error(StatusCode::BAD_REQUEST, message);
    }
    match open::that_detached(&path) {
        Ok(()) => Json(json!({})).into_response(),
        Err(err) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not open folder: {err}"),
        ),
    }
}

#[derive(serde::Deserialize)]
struct OpenRequest {
    path: String,
}

/// Opens a project root in the configured editor (config `editor`, default
/// `code`). Same validation as reveal: only existing directories.
async fn api_open(Json(request): Json<OpenRequest>) -> Response {
    let path = expand_home(&request.path);
    let is_dir = std::fs::metadata(&path).ok().map(|m| m.is_dir());
    if let Err(message) = validate_reveal_path(&path, is_dir) {
        return api_error(StatusCode::BAD_REQUEST, message);
    }
    let cfg = match config::config_path() {
        Some(config_file) => config::load(&config_file),
        None => config::Config::default(),
    };
    let argv = cfg.editor_argv(&path);
    match spawn_detached(&argv) {
        Ok(()) => Json(json!({})).into_response(),
        Err(err) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not launch editor '{}': {err}", argv[0]),
        ),
    }
}

/// Spawn without inheriting the server's stdio, and reap the child from a
/// thread so exited editors never linger as zombies.
fn spawn_detached(argv: &[String]) -> std::io::Result<()> {
    let mut child = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Pure: `is_dir` is None when the path is missing, Some(false) for a file.
/// Kept separate from the fs lookup so it unit-tests without touching the
/// real file manager.
fn validate_reveal_path(path: &std::path::Path, is_dir: Option<bool>) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("no path provided".into());
    }
    match is_dir {
        Some(true) => Ok(()),
        Some(false) => Err("path is not a directory".into()),
        None => Err("path does not exist".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_reveal_path;
    use std::path::Path;

    #[test]
    fn reveal_path_accepts_only_existing_directories() {
        assert!(validate_reveal_path(Path::new("/home/brad/Code"), Some(true)).is_ok());
        assert!(validate_reveal_path(Path::new("/home/brad/f.txt"), Some(false)).is_err());
        assert!(validate_reveal_path(Path::new("/nope"), None).is_err());
        assert!(validate_reveal_path(Path::new(""), Some(true)).is_err());
    }

    #[test]
    fn expand_home_only_touches_a_leading_tilde() {
        let home = dirs::home_dir().expect("home dir in tests");
        assert_eq!(super::expand_home("~/Code/x"), home.join("Code/x"));
        assert_eq!(super::expand_home("/abs/path"), Path::new("/abs/path"));
        assert_eq!(super::expand_home("rel/~/odd"), Path::new("rel/~/odd"));
        assert_eq!(
            super::expand_home("~"),
            Path::new("~"),
            "bare tilde is not a project root"
        );
    }
}

fn still_listening(port: u16, pid: u32) -> bool {
    probe::platform_probe()
        .and_then(|probe| probe.probe().ok())
        .is_some_and(|output| {
            output
                .sockets
                .iter()
                .any(|s| s.port == port && s.pid == Some(pid))
        })
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
}
