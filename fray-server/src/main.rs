mod server;
mod cli;
mod client;
mod public;
mod utils;
mod db;
mod entity;
mod commands;
mod test;
mod redis;

use tracing::info;


#[tokio::main]
async fn main() {
  // load env 
  dotenvy::dotenv().ok();
  // setting up logging
  tracing_subscriber::fmt().init();
  // parsing the cli arguments
  let cli = cli::Cli::parse().unwrap();
  info!("building server...");
  match cli.cmd{
    cli::Commands::Run(args) => {
      commands::run(args).await.expect("Failed to run server");
    }
    cli::Commands::Create(args) => {
      commands::create(args).await.expect("Failed to create client");
    }
    cli::Commands::List => {
      commands::list().await.expect("Failed to list clients");
    }
    cli::Commands::Delete(args) => {
      commands::delete(args).await.expect("Failed to delete client");
    }
  }
}
