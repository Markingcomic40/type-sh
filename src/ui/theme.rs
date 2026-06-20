use crossterm::style::Color;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Deserialize, Serialize)]
struct ThemeFile {
    background: Option<[u8; 3]>,
    primary: [u8; 3],
    secondary: [u8; 3],
    correct: [u8; 3],
    incorrect: [u8; 3],
    incorrect_subtle: [u8; 3],
}

fn rgb_to_color(c: [u8; 3]) -> Color {
    Color::Rgb {
        r: c[0],
        g: c[1],
        b: c[2],
    }
}

impl From<ThemeFile> for Theme {
    fn from(tf: ThemeFile) -> Self {
        Self {
            background: tf.background.map(rgb_to_color),
            primary: rgb_to_color(tf.primary),
            secondary: rgb_to_color(tf.secondary),
            correct: rgb_to_color(tf.correct),
            incorrect: rgb_to_color(tf.incorrect),
            incorrect_subtle: rgb_to_color(tf.incorrect_subtle),
        }
    }
}

pub const BUILTIN_THEMES: &[&str] = &["gruvbox", "ayu-mirage"];

const GRUVBOX_JSON: &str = include_str!("../assets/themes/gruvbox.json");
const AYU_MIRAGE_JSON: &str = include_str!("../assets/themes/ayu-mirage.json");

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Option<Color>,
    pub primary: Color,
    pub secondary: Color,
    pub correct: Color,
    pub incorrect: Color,
    pub incorrect_subtle: Color,
}

impl Theme {
    pub fn gruvbox() -> Self {
        let tf: ThemeFile =
            serde_json::from_str(GRUVBOX_JSON).expect("embedded gruvbox.json is valid");
        tf.into()
    }

    pub fn ayu_mirage() -> Self {
        let tf: ThemeFile =
            serde_json::from_str(AYU_MIRAGE_JSON).expect("embedded ayu-mirage.json is valid");
        tf.into()
    }

    /// Load a theme by name. Try built-in themes first, then fall back to file path.
    pub fn load(name: &str) -> crate::error::Result<Self> {
        match name {
            "gruvbox" => Ok(Self::gruvbox()),
            "ayu-mirage" => Ok(Self::ayu_mirage()),
            _ => Self::from_file(name),
        }
    }

    fn from_file(path: &str) -> crate::error::Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| AppError::ThemeLoad {
            name: path.to_string(),
            source: Box::new(e),
        })?;

        let tf: ThemeFile = serde_json::from_str(&contents).map_err(|e| AppError::ThemeLoad {
            name: path.to_string(),
            source: Box::new(e),
        })?;

        Ok(tf.into())
    }
}
