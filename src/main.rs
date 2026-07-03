use std::net::SocketAddr;

use axum::Router;
use axum::routing::get;
use clap::Parser;

#[derive(Parser)]
#[command(name = "portdoc", version, about = "Local dev server control panel")]
struct Cli {
    /// Port for the local UI server
    #[arg(long, default_value_t = 7788)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app = Router::new().route("/", get(|| async { "PortDoc scaffold" }));

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

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c handler");
}
