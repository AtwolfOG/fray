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
  Run(Config),
  Create(CreateClient),
  List,
  Delete(DeleteClient),
}

#[derive(Parser, Debug, Clone)]
pub struct CreateClient {
  /// Name that can be used to identify the client
  pub name: String,
  /// Optional Hostname, that will be used as the subdomain to access your tunnel
  #[arg(short = 'H', long)]
  pub hostname: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct Config {
  #[arg( default_value = "0.0.0.0")]
  pub addr: String,
  #[arg(short, long, default_value = "3000")]
  pub client_port: String,
  #[arg(short, long, default_value = "3001")]
  pub admin_port: String,
}



#[derive(Parser, Debug, Clone)]
pub struct DeleteClient {
  pub id: i64,
}

impl Cli {
  pub fn parse() -> Result<Cli>{
    let cli = <Self as Parser>::parse();
    Ok(cli)
  }
}