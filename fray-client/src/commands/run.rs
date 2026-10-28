use crate::{cli::RunArgs, client::Client, config};
use tracing::info;

pub async fn run(args: RunArgs) -> anyhow::Result<()> {
  let app_config = config::load_app_config().unwrap_or_default();
  let run_config = app_config.to_run_config(args)?;
  info!("Building client");
  let mut client = Client::new(run_config);
  info!("Running client");
  client.run().await?;
  info!("client exited");
  Ok(())
}
