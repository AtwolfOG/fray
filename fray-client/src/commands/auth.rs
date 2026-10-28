use crate::{
  cli::AuthArgs,
  config::{self, AuthConfig},
};
use tracing::info;

pub async fn auth(args: AuthArgs) -> anyhow::Result<()> {
  let config = AuthConfig::from(args);
  config::save_auth_config(config)?;
  info!("Configuration saved successfully");
  println!("Configuration saved successfully");
  Ok(())
}
