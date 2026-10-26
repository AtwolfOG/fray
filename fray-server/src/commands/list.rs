use anyhow::Context;
use tracing::info;
use crate::db::setup_database;
use crate::entity;
use sea_orm::EntityTrait;

pub async fn list() -> anyhow::Result<()>{
  let db = setup_database(None).await.with_context(|| "Failed to setup database")?;
  let clients = entity::client::Entity::find().all(&db).await.expect("Failed to find clients");
  info!("Clients: {:#?}", clients);
  Ok(())
}