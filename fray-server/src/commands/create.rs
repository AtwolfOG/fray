use anyhow::Context;
use tracing::info;
use crate::cli::CreateArgs;
use crate::db::setup_database;
use crate::{entity, utils};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};

pub async fn create(args: CreateArgs) -> anyhow::Result<()>{
  let db = setup_database(None).await.with_context(|| "Failed to setup database")?;
  let name = args.name;
  let key = utils::generate_random_string(32);
  let client = entity::client::ActiveModel {
    name: Set(name),
    key: Set(key.clone()),
    ..Default::default()
  };
  let result = client.insert(&db).await.expect("Failed to insert client");
  info!("Client created: {:#?}", result);
  println!("Run this in your fray client: fray auth --key {}", key);
  Ok(())
}