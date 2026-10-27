use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::entity::api_logs;

/// Connects to the database
async fn connect() -> Result<DatabaseConnection, DbErr> {
  let mut opt = ConnectOptions::new("sqlite://fray-client.db?mode=rwc");
  opt.max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(false);

  Database::connect(opt).await
}

/// Runs the database migrations
async fn run_migration(db: &DatabaseConnection) -> Result<(), DbErr> {
  db.get_schema_builder()
    .register(api_logs::Entity)
    .sync(db)
    .await
}

pub async fn setup_database() -> Result<DatabaseConnection, DbErr> {
  let db = connect().await?;
  run_migration(&db).await?;
  Ok(db)
}