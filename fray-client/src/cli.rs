use std::{fs, io, path::PathBuf};

use anyhow::{bail, Ok, Result};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};


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
  #[arg(short, long, env, default_value_t = get_remote_url_from_config())]
  pub remote_addr: String,
  #[arg(short, long, env, default_value_t = get_key_from_config())]
  pub key: String,
  #[arg(short, long, env, default_value_t = get_id_from_config())]
  pub id: i64,
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

#[derive(Deserialize, Serialize, Default)]
pub struct AppConfig {
  pub remote_url: String,
  pub key: String,
  pub id: i64,
}

impl From<AuthArgs> for AppConfig {
  fn from(conf: AuthArgs) -> Self {
    AppConfig { remote_url: conf.remote_url, key: conf.key, id: conf.id }
  }
}


fn config_file_dir() -> Option<ProjectDirs> {
  let dirs = ProjectDirs::from("xyz", "atwolf", "fray")?;
  Some(dirs)
}

fn config_file_path() -> Option<PathBuf> {
  let dirs = config_file_dir()?;
  Some(dirs.config_dir().join("config.toml"))
}

fn load_app_config() -> Option<AppConfig> {
  let config_path = config_file_path()?;
  if !config_path.exists() {
    return None;
  }
  let contents = fs::read_to_string(config_path).ok()?;

  toml::from_str(&contents).ok()
}

fn get_remote_url_from_config() -> String {
  let Some(config) = load_app_config() else {
    return "".to_owned();
  };
  config.remote_url
}

fn get_id_from_config() -> i64 {
  let Some(config) = load_app_config() else {
    return 0;
  };
  config.id
}

fn get_key_from_config() -> String {
  let Some(config) = load_app_config() else {
    return "".to_owned();
  };
  config.key
}

pub fn save_app_config(config: AppConfig) -> Result<()>{
  let contents = toml::to_string_pretty(&config)?;
  let config_dir = config_file_dir().ok_or_else(|| io::Error::other("Can not determine config directory"))?;
  fs::create_dir_all(config_dir.config_dir())?;
  let config_path = config_file_path().ok_or_else(|| io::Error::other("Can not determine config file path"))?;
  fs::write(config_path, contents)?;
  Ok(())
}

impl Cli {
  pub fn parse() -> Result<Self>{
    let cli = <Self as Parser>::parse();
    if let Commands::Run(ref config) = cli.cmd {
      if config.remote_addr.is_empty() {
        bail!("remote url is empty")
      }
      if config.key.is_empty() {
        bail!("auth key is empty")
      }
    }
    Ok(cli)
  }
}