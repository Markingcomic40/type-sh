use serde::{Deserialize, Serialize};

use crate::core::config::{Gamemode, TestConfig, DEFAULT_TIMED_SECS, DEFAULT_WORDSET};

use crate::error::{AppError, Result};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Preferences {
    pub wordset: String,
    pub mode: Gamemode,
    pub theme: String,
    #[serde(default = "Preferences::default_freedom")]
    pub freedom_mode: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            wordset: DEFAULT_WORDSET.to_string(),
            mode: Gamemode::Timed(DEFAULT_TIMED_SECS),
            theme: "gruvbox".to_string(),
            freedom_mode: true,
        }
    }
}

impl Preferences {
    fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("type-sh").join("config.json"))
    }

    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()
            .ok_or_else(|| AppError::Config("Could not determine config directory".to_string()))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        };

        let json =
            serde_json::to_string_pretty(self).map_err(|e| AppError::Config(e.to_string()))?;

        std::fs::write(&path, json)?;

        Ok(())
    }

    fn default_freedom() -> bool {
        true
    }
}
