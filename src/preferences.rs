use crate::core::config::{Gamemode, DEFAULT_TIMED_SECS, DEFAULT_WORDSET};

pub struct Preferences {
    pub wordset: String,
    pub mode: Gamemode,
    pub theme: String,
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
