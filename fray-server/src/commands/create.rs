use anyhow::Context;
use tracing::info;
use crate::cli::CreateArgs;
use crate::db::setup_database;
use crate::{entity, utils};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};

pub async fn create(args: CreateArgs) -> anyhow::Result<()>{
  let db = setup_database(None).await.with_context(|| "Failed to setup database")?;
  let name = args.name;
  let client_secret = utils::generate_random_string(32);
  let hostname = if let Some(hostname) = args.hostname {
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
  Ok(())
}