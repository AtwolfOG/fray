use crate::cli::{self, AppConfig, AuthArgs};
use tracing::info;

pub async fn auth(args: AuthArgs) -> anyhow::Result<()> {
  let config = AppConfig::from(args);
  cli::save_app_config(config)?;
  info!("Configuration saved successfully");
  println!("Configuration saved successfully");
  Ok(())
}
