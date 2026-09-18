use crossterm::style::Color;
use serde::Deserialize;

use crate::error::{AppError, Result};

pub const BUILTIN_THEMES: &[&str] = &["gruvbox", "ayu-mirage"];

/// Colours by the role they play.
#[derive(Clone, Debug)]
pub struct Theme {
    /// `Color::Reset` keeps the terminal's own background.
    pub background: Color,
    /// Main text.
    pub text: Color,
    /// Secondary text: labels, hints, words not typed yet.
    pub dim: Color,
    /// Barely there: separators and chart axes.
    pub faint: Color,
    /// Whatever should catch the eye: selections, the timer, the headline numbers.
    pub accent: Color,
    pub correct: Color,
    pub error: Color,
    /// Characters typed past the end of a word.
    pub error_extra: Color,
}

type Rgb = [u8; 3];

/// The on-disk format. Optional colours fall back to a sensible neighbour, and
/// the aliases keep themes written against the old field names loading.
#[derive(Deserialize)]
struct ThemeFile {
    background: Option<Rgb>,
    #[serde(alias = "primary")]
    text: Rgb,
    #[serde(alias = "secondary")]
    dim: Rgb,
    faint: Option<Rgb>,
    accent: Option<Rgb>,
    correct: Option<Rgb>,
    #[serde(alias = "incorrect")]
    error: Rgb,
    #[serde(alias = "incorrect_subtle")]
    error_extra: Option<Rgb>,
}

fn rgb([r, g, b]: Rgb) -> Color {
    Color::Rgb { r, g, b }
}

impl From<ThemeFile> for Theme {
    fn from(file: ThemeFile) -> Self {
        Self {
            background: file.background.map_or(Color::Reset, rgb),
            text: rgb(file.text),
            dim: rgb(file.dim),
            faint: rgb(file.faint.unwrap_or(file.dim)),
            accent: rgb(file.accent.unwrap_or(file.text)),
            correct: rgb(file.correct.unwrap_or(file.text)),
            error: rgb(file.error),
            error_extra: rgb(file.error_extra.unwrap_or(file.error)),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::load(BUILTIN_THEMES[0]).expect("builtin themes are valid")
    }
}

impl Theme {
    /// Loads a builtin theme by name, or else a theme file by path.
    pub fn load(name: &str) -> Result<Self> {
        let err = |source: Box<dyn std::error::Error + Send + Sync>| AppError::Theme {
            name: name.to_owned(),
            source,
        };

        let json = match name {
            "gruvbox" => include_str!("../assets/themes/gruvbox.json").to_owned(),
            "ayu-mirage" => include_str!("../assets/themes/ayu-mirage.json").to_owned(),
            path => std::fs::read_to_string(path).map_err(|e| err(e.into()))?,
        };

        let file: ThemeFile = serde_json::from_str(&json).map_err(|e| err(e.into()))?;
        Ok(file.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_themes_load() {
        for name in BUILTIN_THEMES {
            assert!(Theme::load(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn old_field_names_still_load() {
        let json = r#"{
            "background": null,
            "primary": [1, 1, 1],
            "secondary": [2, 2, 2],
            "correct": [3, 3, 3],
            "incorrect": [4, 4, 4],
            "incorrect_subtle": [5, 5, 5]
        }"#;
        let theme: Theme = serde_json::from_str::<ThemeFile>(json).unwrap().into();

        assert_eq!(theme.accent, rgb([1, 1, 1]));
        assert_eq!(theme.error_extra, rgb([5, 5, 5]));
    }
}
