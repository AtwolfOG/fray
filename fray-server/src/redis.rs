use anyhow::{Context, Result};
use redis::{Client, aio::MultiplexedConnection};

pub async fn connect(url: Option<String>) -> Result<MultiplexedConnection> {
  let url = url.unwrap_or_else(|| "redis://localhost:6379".to_string());
  let client = Client::open(url).with_context(|| "Failed to connect to redis")?;
  let conn = client.get_multiplexed_async_connection().await.with_context(|| "Failed to get redis connection")?;
  Ok(conn)
}