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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.theme, "synthwave84");
        assert_eq!(cfg.default_compression, 3);
        assert!(cfg.repo_paths.is_empty());
        assert_eq!(cfg.retention.keep_daily, 7);
        assert_eq!(cfg.retention.keep_weekly, 4);
        assert_eq!(cfg.retention.keep_monthly, 12);
    }

    #[test]
    fn toml_roundtrip() -> Result<()> {
        let tmp = TempDir::new()?;
        let original = Config {
            archive_dir: tmp.path().join("archives"),
            db_path: tmp.path().join("forge.db"),
            default_compression: 5,
            repo_paths: vec!["/home/user/projects".to_string()],
            retention: RetentionConfig {
                keep_daily: 14,
                keep_weekly: 8,
                keep_monthly: 24,
            },
            theme: "dracula".to_string(),
        };

        let toml_str = toml::to_string_pretty(&original).context("serialize")?;
        let parsed: Config = toml::from_str(&toml_str).context("deserialize")?;

        assert_eq!(parsed.archive_dir, original.archive_dir);
        assert_eq!(parsed.db_path, original.db_path);
        assert_eq!(parsed.default_compression, 5);
        assert_eq!(parsed.repo_paths, vec!["/home/user/projects"]);
        assert_eq!(parsed.retention.keep_daily, 14);
        assert_eq!(parsed.retention.keep_weekly, 8);
        assert_eq!(parsed.retention.keep_monthly, 24);
        assert_eq!(parsed.theme, "dracula");

        Ok(())
    }

    #[test]
    fn config_path_is_under_config_dir() {
        let path = Config::config_path();
        assert!(path.to_string_lossy().contains("forge"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
