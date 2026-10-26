use crate::{cli::RunArgs, db::setup_database, redis, server};
use tracing::info;


pub async fn run(args: RunArgs) -> anyhow::Result<()> {
  let db = setup_database(None).await?;
  let redis_client = redis::connect(Some(args.redis_url.clone())).await?;
  let server = server::Server::new(args, db, redis_client);
  info!("starting server...");
  server.run().await?;
  info!("server exited");
  Ok(())
}