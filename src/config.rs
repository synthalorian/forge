use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub archive_dir: PathBuf,
    pub db_path: PathBuf,
    pub default_compression: u32,
    pub repo_paths: Vec<String>,
    pub retention: RetentionConfig,
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp/forge"))
            .join("forge");

        Self {
            archive_dir: data_dir.join("archives"),
            db_path: data_dir.join("forge.db"),
            default_compression: 3,
            repo_paths: Vec::new(),
            retention: RetentionConfig {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 12,
            },
            theme: "synthwave84".to_string(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp/forge"))
            .join("forge")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            anyhow::bail!(
                "forge config not found at {}. Run 'forge init' first.",
                path.display()
            );
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        fs::create_dir_all(&self.archive_dir).with_context(|| {
            format!(
                "Failed to create archive directory {}",
                self.archive_dir.display()
            )
        })?;

        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create database directory {}", parent.display())
            })?;
        }

        Ok(())
    }
}
