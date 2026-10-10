use anyhow::{Ok, Result};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Config {
  #[arg( default_value = "0.0.0.0")]
  pub addr: String,
  #[arg(short, long, default_value = "3000")]
  pub client_port: String,
  #[arg(short, long, default_value = "3001")]
  pub admin_port: String,
}

impl Config {
  pub fn parse() -> Result<Self>{
    let cfg = <Self as Parser>::parse();
    Ok(cfg)
  }
}