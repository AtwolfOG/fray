use std::{fs, io, path::PathBuf};

use anyhow::{Ok, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::cli::{AuthArgs, RunArgs};

#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
  pub remote_url: String,
  pub key: String,
  pub id: i64,
}

impl AppConfig {
  #[allow(dead_code)]
  pub fn to_auth_config(&self) -> AuthConfig {
    AuthConfig::from(self)
  }

  pub fn to_run_config(&self, args: RunArgs) -> Result<RunConfig> {
    RunConfig::from_app_config(self.clone(), args)
  }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
  pub remote_url: String,
  pub key: String,
  pub id: i64,
}

impl From<AppConfig> for AuthConfig {
  fn from(conf: AppConfig) -> Self {
    AuthConfig {
      remote_url: conf.remote_url,
      key: conf.key,
      id: conf.id,
    }
  }
}

impl From<&AppConfig> for AuthConfig {
  fn from(conf: &AppConfig) -> Self {
    AuthConfig {
      remote_url: conf.remote_url.clone(),
      key: conf.key.clone(),
      id: conf.id,
    }
  }
}

impl From<AuthConfig> for AppConfig {
  fn from(conf: AuthConfig) -> Self {
    AppConfig {
      remote_url: conf.remote_url,
      key: conf.key,
      id: conf.id,
    }
  }
}

impl From<AuthArgs> for AuthConfig {
  fn from(conf: AuthArgs) -> Self {
    AuthConfig {
      remote_url: conf.remote_url,
      key: conf.key,
      id: conf.id,
    }
  }
}

impl From<AuthArgs> for AppConfig {
  fn from(conf: AuthArgs) -> Self {
    AuthConfig::from(conf).into()
  }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct RunConfig {
  pub local_port: String,
  pub admin_port: String,
  pub remote_addr: String,
  pub key: String,
  pub id: i64,
}

impl From<AppConfig> for RunConfig {
  fn from(conf: AppConfig) -> Self {
    RunConfig {
      local_port: "3000".to_string(),
      admin_port: "3001".to_string(),
      remote_addr: conf.remote_url,
      key: conf.key,
      id: conf.id,
    }
  }
}

impl From<&AppConfig> for RunConfig {
  fn from(conf: &AppConfig) -> Self {
    RunConfig {
      local_port: "3000".to_string(),
      admin_port: "3001".to_string(),
      remote_addr: conf.remote_url.clone(),
      key: conf.key.clone(),
      id: conf.id,
    }
  }
}

impl From<AuthConfig> for RunConfig {
  fn from(conf: AuthConfig) -> Self {
    RunConfig::from(AppConfig::from(conf))
  }
}

impl From<RunConfig> for AppConfig {
  fn from(conf: RunConfig) -> Self {
    AppConfig {
      remote_url: conf.remote_addr,
      key: conf.key,
      id: conf.id,
    }
  }
}

impl From<RunConfig> for AuthConfig {
  fn from(conf: RunConfig) -> Self {
    AuthConfig {
      remote_url: conf.remote_addr,
      key: conf.key,
      id: conf.id,
    }
  }
}

impl RunConfig {
  pub fn from_app_config(app_config: AppConfig, args: RunArgs) -> Result<Self> {
    let remote_addr = args
      .remote_addr
      .filter(|s| !s.is_empty())
      .or_else(|| {
        if !app_config.remote_url.is_empty() {
          Some(app_config.remote_url)
        } else {
          None
        }
      })
      .ok_or_else(|| anyhow::anyhow!("remote url is empty"))?;

    let key = args
      .key
      .filter(|s| !s.is_empty())
      .or_else(|| {
        if !app_config.key.is_empty() {
          Some(app_config.key)
        } else {
          None
        }
      })
      .ok_or_else(|| anyhow::anyhow!("auth key is empty"))?;

    let id = args.id.unwrap_or(app_config.id);

    Ok(Self {
      local_port: args.local_port,
      admin_port: args.admin_port,
      remote_addr,
      key,
      id,
    })
  }
}

pub fn config_file_dir() -> Option<ProjectDirs> {
  let dirs = ProjectDirs::from("xyz", "atwolf", "fray")?;
  Some(dirs)
}

pub fn config_file_path() -> Option<PathBuf> {
  let dirs = config_file_dir()?;
  Some(dirs.config_dir().join("config.toml"))
}

pub fn load_app_config() -> Option<AppConfig> {
  let config_path = config_file_path()?;
  if !config_path.exists() {
    return None;
  }
  let contents = fs::read_to_string(config_path).ok()?;

  toml::from_str(&contents).ok()
}

pub fn save_app_config(config: AppConfig) -> Result<()> {
  let contents = toml::to_string_pretty(&config)?;
  let config_dir = config_file_dir().ok_or_else(|| io::Error::other("Can not determine config directory"))?;
  fs::create_dir_all(config_dir.config_dir())?;
  let config_path = config_file_path().ok_or_else(|| io::Error::other("Can not determine config file path"))?;
  fs::write(config_path, contents)?;
  Ok(())
}

pub fn save_auth_config(config: AuthConfig) -> Result<()> {
  save_app_config(AppConfig::from(config))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_config_derivation() {
    let app = AppConfig {
      remote_url: "http://example.com".to_string(),
      key: "test-key".to_string(),
      id: 42,
    };

    let auth = AuthConfig::from(&app);
    assert_eq!(auth.remote_url, "http://example.com");
    assert_eq!(auth.key, "test-key");
    assert_eq!(auth.id, 42);

    let roundtrip_app = AppConfig::from(auth);
    assert_eq!(roundtrip_app, app);
  }

  #[test]
  fn test_run_config_from_app_config_with_none_args() {
    let app = AppConfig {
      remote_url: "http://example.com".to_string(),
      key: "test-key".to_string(),
      id: 42,
    };

    let args = RunArgs {
      local_port: "3000".to_string(),
      admin_port: "3001".to_string(),
      remote_addr: None,
      key: None,
      id: None,
    };

    let run_config = RunConfig::from_app_config(app, args).unwrap();
    assert_eq!(run_config.local_port, "3000");
    assert_eq!(run_config.admin_port, "3001");
    assert_eq!(run_config.remote_addr, "http://example.com");
    assert_eq!(run_config.key, "test-key");
    assert_eq!(run_config.id, 42);
  }

  #[test]
  fn test_run_config_override_args() {
    let app = AppConfig {
      remote_url: "http://example.com".to_string(),
      key: "test-key".to_string(),
      id: 42,
    };

    let args = RunArgs {
      local_port: "8080".to_string(),
      admin_port: "8081".to_string(),
      remote_addr: Some("http://override.com".to_string()),
      key: Some("override-key".to_string()),
      id: Some(99),
    };

    let run_config = RunConfig::from_app_config(app, args).unwrap();
    assert_eq!(run_config.local_port, "8080");
    assert_eq!(run_config.admin_port, "8081");
    assert_eq!(run_config.remote_addr, "http://override.com");
    assert_eq!(run_config.key, "override-key");
    assert_eq!(run_config.id, 99);
  }

  #[test]
  fn test_run_config_missing_required_fields() {
    let app = AppConfig::default();
    let args = RunArgs {
      local_port: "3000".to_string(),
      admin_port: "3001".to_string(),
      remote_addr: None,
      key: None,
      id: None,
    };

    let result = RunConfig::from_app_config(app, args);
    assert!(result.is_err());
  }
}
