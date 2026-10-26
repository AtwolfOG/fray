use crate::{cli::RunArgs, db::setup_database, server};
use tracing::info;


pub async fn run(args: RunArgs) -> anyhow::Result<()> {
  let db = setup_database(None).await?;
  let server = server::Server::new(args, db);
  info!("starting server...");
  server.run().await?;
  info!("server exited");
  Ok(())
}