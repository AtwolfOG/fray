use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};


#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub cmd: Commands,
}


#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
  Run(RunArgs),
  Auth(AuthArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
  #[arg(short, long, default_value = "3000")]
  pub local_port: String,
  #[arg(short, long, default_value = "3001")]
  pub admin_port: String,
  #[arg(short, long, env)]
  pub remote_addr: Option<String>,
  #[arg(short, long, env)]
  pub key: Option<String>,
  #[arg(short, long, env)]
  pub id: Option<i64>,
}

#[derive(Parser, Debug, Clone)]
pub struct AuthArgs {
  #[arg(short, long, env)]
  pub key: String,
  #[arg(short, long, env)]
  pub remote_url: String,
  #[arg(short, long, env)]
  pub id: i64,
}

impl Cli {
  pub fn parse() -> Result<Self>{
    let cli = <Self as Parser>::parse();
    Ok(cli)
  }
}