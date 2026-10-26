use anyhow::Context;
use tracing::info;
use crate::cli::DeleteArgs;
use crate::db::setup_database;
use crate::entity;
use sea_orm::{EntityTrait, ModelTrait};

pub async fn delete(args: DeleteArgs) -> anyhow::Result<()>{
  let db = setup_database(None).await.with_context(|| "Failed to setup database")?;
  let client = entity::client::Entity::find_by_id(args.id).one(&db).await.expect("Failed to find client");
  let client = client.expect("Client not found");
  let res = client.delete(&db).await.expect("Failed to delete client");
  info!("Client deleted: {:#?}", res);
  Ok(())
}