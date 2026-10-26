use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli{
  #[command(subcommand)]
  pub cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
  Run(RunArgs),
  Create(CreateArgs),
  List,
  Delete(DeleteArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct CreateArgs {
  /// Name that can be used to identify the client
  pub name: String,
  /// Optional Hostname, that will be used as the subdomain to access your tunnel
  #[arg(short = 'H', long)]
  pub hostname: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
  #[arg( default_value = "0.0.0.0")]
  pub addr: String,
  #[arg(short, long, default_value = "8000")]
  pub client_port: String,
  #[arg(short, long, default_value = "8001")]
  pub public_port: String,
  #[arg(short, long, default_value = "8002")]
  pub admin_port: String,
  #[arg(long, env = "REDIS_URL", default_value = "redis://localhost:6379")]
  pub redis_url: String,
}



#[derive(Parser, Debug, Clone)]
pub struct DeleteArgs {
  pub id: i64,
}

impl Cli {
  pub fn parse() -> Result<Cli>{
    let cli = <Self as Parser>::parse();
    Ok(cli)
  }
}