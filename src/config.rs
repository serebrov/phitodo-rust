use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub shortcut_modifier: String,
    pub github_token: Option<String>,
    pub github_repos: Vec<String>,
    pub toggl_token: Option<String>,
    pub toggl_hidden_projects: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shortcut_modifier: "alt".to_string(),
            github_token: None,
            github_repos: Vec::new(),
            toggl_token: None,
            toggl_hidden_projects: Vec::new(),
        }
    }
}

impl Config {
    /// Returns the config directory path (~/.config/phitodo-tui/)
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::Config("Could not find config directory".to_string()))?
            .join("phitodo-tui");
        Ok(config_dir)
    }

    /// Returns the config file path (~/.config/phitodo-tui/config.toml)
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Returns the data directory path (~/.local/share/phitodo-tui/)
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| AppError::Config("Could not find data directory".to_string()))?
            .join("phitodo-tui");
        Ok(data_dir)
    }

    /// Returns the database file path (~/.local/share/phitodo-tui/phitodo.db)
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("phitodo.db"))
    }

    /// Load config from file, or create default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        fs::create_dir_all(&config_dir)?;

        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(config_path, contents)?;

        Ok(())
    }

    /// Ensure all required directories exist
    pub fn ensure_dirs() -> Result<()> {
        fs::create_dir_all(Self::config_dir()?)?;
        fs::create_dir_all(Self::data_dir()?)?;
        Ok(())
    }

    /// Check if GitHub is configured
    pub fn has_github(&self) -> bool {
        self.github_token.as_ref().is_some_and(|t| !t.is_empty())
    }

    /// Check if Toggl is configured
    pub fn has_toggl(&self) -> bool {
        self.toggl_token.as_ref().is_some_and(|t| !t.is_empty())
    }
}
