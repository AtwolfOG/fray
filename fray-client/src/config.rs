use anyhow::{Ok, Result};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Config {
  #[arg(long)]
  pub key: String,
  #[arg(short, long, default_value = "3000")]
  pub listen_port: String,
  #[arg(short, long, default_value = "3001")]
  pub admin_port: String,
  #[arg(short, long)]
  pub remote_addr: String,
}

impl Config {
  pub fn parse() -> Result<Self>{
    let cfg = <Self as Parser>::parse();
    Ok(cfg)
  }
}