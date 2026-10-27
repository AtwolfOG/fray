mod cli;
mod client;
mod request;
mod db;
mod entity;
mod commands;

use tracing::info;

#[tokio::main]
async fn main() {
  // load env 
  dotenvy::dotenv().ok();
  // setting up logging
  tracing_subscriber::fmt().init();
  // parsing the cli arguments
  let cli = cli::Cli::parse().unwrap();
  info!("building client...");
  match cli.cmd {
    cli::Commands::Run(args) => {
      commands::run(args).await.expect("Failed to run client");
    }
    cli::Commands::Auth(args) => {
      commands::auth(args).await.expect("Failed to save auth configuration");
    }
  }
}
