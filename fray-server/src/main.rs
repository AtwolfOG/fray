mod server;
mod config;
mod client;
mod public;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let config = config::Config::parse().unwrap();
    info!("building server...");
    let server = server::Server::new(config);
    info!("starting server...");
    let _ = server.run().await;
    info!("server exited");
}
