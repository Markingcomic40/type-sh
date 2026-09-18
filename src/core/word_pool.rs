use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;

use crate::error::{AppError, Result};

pub const BUILTIN_WORDLISTS: &[&str] = &["english", "rust"];

fn builtin(name: &str) -> Option<&'static str> {
    match name {
        "english" | "english_200" => Some(include_str!("../assets/english_200.txt")),
        "rust" => Some(include_str!("../assets/rust.txt")),
        _ => None,
    }
}

/// Loads a builtin word list by name, or else a whitespace separated word file by path.
pub fn load(name: &str) -> Result<Vec<String>> {
    let words: Vec<String> = match builtin(name) {
        Some(text) => split(text),
        None => {
            let text = std::fs::read_to_string(name).map_err(|source| AppError::WordList {
                name: name.to_owned(),
                source,
            })?;
            split(&text)
        }
    };

    if words.is_empty() {
        return Err(AppError::EmptyWordList(name.to_owned()));
    }

    Ok(words)
}

// A word can't contain whitespace for now sadly ill think of sth to maybe be able to load like sentences but idk how thatd work
fn split(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

pub struct WordList {
    words: Vec<String>,
    rng: ThreadRng,
}

impl WordList {
    pub fn new(name: &str) -> Result<Self> {
        Ok(Self::from_words(load(name)?))
    }

    pub fn from_words(words: Vec<String>) -> Self {
        Self {
            words,
            rng: rand::rng(),
        }
    }

    pub fn next_word(&mut self) -> &str {
        self.words
            .choose(&mut self.rng)
            .expect("word lists are never empty")
    }
}
