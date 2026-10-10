mod config;
mod client;
mod request;

use config::Config;
use client::Client;
use tracing::info;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  let config = Config::parse().unwrap();
  info!("Building client");
  let mut client = Client::new(config);
  info!("Running client");
  client.run().await.map_err(|e| format!("Error: {}", e)).unwrap();
}
