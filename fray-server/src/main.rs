mod server;
mod cli;
mod client;
mod public;
mod utils;
mod db;
mod entity;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, ModelTrait};
use tracing::info;

#[tokio::main]
async fn main() {
  // setting up logging
    tracing_subscriber::fmt().init();
    // parsing the cli arguments
    let cli = cli::Cli::parse().unwrap();
    info!("building server...");
    // connecting to database
    info!("connecting to database");
    let db = db::setup_database().await.expect("Failed to connect to database");
    info!("connected to database");
    match cli.cmd{
      cli::Commands::Run(config) => {
        let server = server::Server::new(config, db);
        info!("starting server...");
        let _ = server.run().await;
        info!("server exited");
      }
      cli::Commands::Create(create) => {
        let name = create.name;
        let client_secret = utils::generate_random_string(32);
        let hostname = if let Some(hostname) = create.hostname {
          hostname
        } else {
          utils::generate_host()
        };
        let client = entity::client::ActiveModel {
          name: Set(name),
          client_secret: Set(client_secret.clone()),
          host: Set(hostname.clone()),
          ..Default::default()
        };
        let result = client.insert(&db).await.expect("Failed to insert client");
        info!("Client created: {:#?}", result);
        info!("Client secret: {}", client_secret);
      }
      cli::Commands::List => {
        let clients = entity::client::Entity::find().all(&db).await.expect("Failed to find clients");
        info!("Clients: {:#?}", clients);
      }
      cli::Commands::Delete(delete) => {
        let client = entity::client::Entity::find_by_id(delete.id).one(&db).await.expect("Failed to find client");
        let client = client.expect("Client not found");
        let res = client.delete(&db).await.expect("Failed to delete client");
        info!("Client deleted: {:#?}", res);
      }
    }
}
