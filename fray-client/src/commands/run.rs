use crate::{cli::RunArgs, client::Client};
use tracing::info;

pub async fn run(args: RunArgs) -> anyhow::Result<()> {
  info!("Building client");
  let mut client = Client::new(args);
  info!("Running client");
  client.run().await?;
  info!("client exited");
  Ok(())
}
