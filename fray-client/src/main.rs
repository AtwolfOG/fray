mod cli;
mod client;
mod request;
mod db;
mod entity;


use client::Client;
use tracing::info;

#[tokio::main]
async fn main() {
  // setting up logs
  tracing_subscriber::fmt().init();
  // parsing the cli arguments
  let cli = cli::Cli::parse().unwrap();
  info!("Parsed CLI: {:?}", cli);
  println!("Parsed CLI: {:?}", cli);
  // matching the cli arguments
  match cli.cmd {
    cli::Commands::Run(config) => {
      info!("Building client");
      let mut client = Client::new(config);
      info!("Running client");
      client.run().await.map_err(|e| format!("Error: {}", e)).unwrap();
    }
    cli::Commands::Auth(conf) => {
      let config = cli::AppConfig::from(conf);
      cli::save_app_config(config).unwrap();
    }
  }
}
