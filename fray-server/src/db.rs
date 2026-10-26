use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::entity::{api_log, client};

/// Connects to the database
async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
  let mut opt = ConnectOptions::new(url);
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
    .register(client::Entity)
    .register(api_log::Entity)
    .sync(db)
    .await
}

pub async fn setup_database(db_url: Option<&str>) -> Result<DatabaseConnection, DbErr> {
  let url = db_url.unwrap_or("sqlite://fray-server.db?mode=rwc");
  let db = connect(url).await?;
  run_migration(&db).await?;
  Ok(db)
}