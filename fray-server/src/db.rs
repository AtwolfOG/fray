use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

pub async fn connect() -> Result<DatabaseConnection, DbErr> {
  let mut opt = ConnectOptions::new("sqlite://fray-server.db?mode=rwc");
  opt.max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(false);

  Database::connect(opt).await
}
