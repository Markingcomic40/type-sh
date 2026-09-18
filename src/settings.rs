use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::core::config::{Limit, TestConfig};
use crate::core::word_pool::{self, BUILTIN_WORDLISTS};
use crate::error::{AppError, Result};
use crate::ui::theme::{Theme, BUILTIN_THEMES};

pub const MAX_AMOUNT: u64 = 4269;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Time,
    Words,
    Zen,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Time, Mode::Words, Mode::Zen];

    pub fn name(self) -> &'static str {
        match self {
            Mode::Time => "time",
            Mode::Words => "words",
            Mode::Zen => "zen",
        }
    }

    pub fn presets(self) -> &'static [u64] {
        match self {
            Mode::Time => &[15, 30, 60, 120],
            Mode::Words => &[10, 25, 50, 100],
            Mode::Zen => &[],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    #[serde(deserialize_with = "or_default")]
    pub mode: Mode,
    pub time: u64,
    pub words: u64,
    #[serde(alias = "wordset")]
    pub wordlist: String,
    pub custom_wordlist: Option<String>,
    pub theme: String,
    pub custom_theme: Option<String>,
    #[serde(alias = "freedom_mode")]
    pub freedom: bool,
}

fn or_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Time,
            time: 30,
            words: 25,
            wordlist: BUILTIN_WORDLISTS[0].to_owned(),
            custom_wordlist: None,
            theme: BUILTIN_THEMES[0].to_owned(),
            custom_theme: None,
            freedom: true,
        }
    }
}

impl Settings {
    /// Loads saved settings, falling back to defaults if there are none or they don't parse
    pub fn load() -> Self {
        Self::path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|json| serde_json::from_str::<Self>(&json).ok())
            .unwrap_or_default()
            .sanitized()
    }

    pub fn save(&self) -> Result<()> {
        let path =
            Self::path().ok_or_else(|| AppError::Settings("no config directory".to_owned()))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        let json =
            serde_json::to_string_pretty(self).map_err(|e| AppError::Settings(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("type-sh").join("config.json"))
    }

    /// Attempt repair for ppl that f up
    fn sanitized(mut self) -> Self {
        let defaults = Self::default();
        // The builtin list's old name
        if self.wordlist == "english_200" {
            self.wordlist.clone_from(&defaults.wordlist);
        }
        if !(1..=MAX_AMOUNT).contains(&self.time) {
            self.time = defaults.time;
        }
        if !(1..=MAX_AMOUNT).contains(&self.words) {
            self.words = defaults.words;
        }
        if !self.wordlists().contains(&self.wordlist.as_str()) {
            self.custom_wordlist = Some(self.wordlist.clone());
        }
        if !self.themes().contains(&self.theme.as_str()) {
            self.custom_theme = Some(self.theme.clone());
        }
        self
    }

    pub fn test_config(&self) -> TestConfig {
        TestConfig {
            limit: match self.mode {
                Mode::Time => Limit::Time(self.time),
                Mode::Words => Limit::Words(self.words),
                Mode::Zen => Limit::None,
            },
            wordlist: self.wordlist.clone(),
            freedom: self.freedom,
        }
    }

    /// The time or word count for the current mode.
    pub fn amount(&self) -> Option<u64> {
        match self.mode {
            Mode::Time => Some(self.time),
            Mode::Words => Some(self.words),
            Mode::Zen => None,
        }
    }

    pub fn set_amount(&mut self, amount: u64) {
        match self.mode {
            Mode::Time => self.time = amount,
            Mode::Words => self.words = amount,
            Mode::Zen => {}
        }
    }

    pub fn wordlists(&self) -> Vec<&str> {
        with_custom(BUILTIN_WORDLISTS, &self.custom_wordlist)
    }

    pub fn themes(&self) -> Vec<&str> {
        with_custom(BUILTIN_THEMES, &self.custom_theme)
    }

    /// Load n switch to path
    pub fn set_custom_wordlist(&mut self, path: &str) -> Result<()> {
        let path = expand_home(path);
        word_pool::load(&path)?;
        self.wordlist.clone_from(&path);
        self.custom_wordlist = Some(path);
        Ok(())
    }

    pub fn set_custom_theme(&mut self, path: &str) -> Result<()> {
        let path = expand_home(path);
        Theme::load(&path)?;
        self.theme.clone_from(&path);
        self.custom_theme = Some(path);
        Ok(())
    }
}

fn with_custom<'a>(builtins: &[&'a str], custom: &'a Option<String>) -> Vec<&'a str> {
    builtins.iter().copied().chain(custom.as_deref()).collect()
}

pub fn display_name(value: &str) -> &str {
    Path::new(value)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(value)
}

fn expand_home(path: &str) -> String {
    let path = path.trim();
    match (path.strip_prefix("~/"), dirs::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest).to_string_lossy().into_owned(),
        _ => path.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_from_older_versions_carry_over() {
        let json = r#"{
            "wordset": "/words.txt",
            "mode": { "Timed": 5 },
            "theme": "/theme.json",
            "freedom_mode": false
        }"#;
        let s = serde_json::from_str::<Settings>(json).unwrap().sanitized();

        assert_eq!(s.mode, Mode::Time);
        assert_eq!(s.wordlist, "/words.txt");
        assert_eq!(s.custom_wordlist.as_deref(), Some("/words.txt"));
        assert_eq!(s.theme, "/theme.json");
        assert_eq!(s.custom_theme.as_deref(), Some("/theme.json"));
        assert!(!s.freedom);
    }

    #[test]
    fn old_builtin_wordlist_name_maps_to_the_new_one() {
        let s = serde_json::from_str::<Settings>(r#"{ "wordset": "english_200" }"#)
            .unwrap()
            .sanitized();

        assert_eq!(s.wordlist, "english");
        assert_eq!(s.custom_wordlist, None);
    }
}
