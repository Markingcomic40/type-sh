use rand::seq::IndexedRandom;

use crate::error::{AppError, Result};

const ENGLISH_200: &str = include_str!("../assets/english_200.txt");
const RUST: &str = include_str!("../assets/rust.txt");
pub struct WordSet;

impl WordSet {
    pub fn load(name: &str) -> Result<Vec<String>> {
        if let Some(builtin) = Self::load_builtin(name) {
            if builtin.is_empty() {
                return Err(AppError::EmptyWordSet(name.to_string()));
            }

            return Ok(builtin);
        }

        let content = std::fs::read_to_string(name).map_err(|source| AppError::WordSetLoad {
            name: name.to_string(),
            source,
        })?;

        let words: Vec<String> = content.lines().map(String::from).collect();

        if words.is_empty() {
            return Err(AppError::EmptyWordSet(name.to_string()));
        }

        Ok(words)
    }

    fn load_builtin(name: &str) -> Option<Vec<String>> {
        let text = match name {
            "english_200" | "english" => ENGLISH_200,
            "rust" => RUST,
            _ => return None,
        };

        Some(text.lines().map(String::from).collect())
    }
}

pub struct WordList {
    source_pool: Vec<String>,
    rng: rand::rngs::ThreadRng,
}

impl WordList {
    pub fn new(name: &str) -> Result<Self> {
        let source_pool = WordSet::load(name)?;

        Ok(Self {
            source_pool,
            rng: rand::rng(),
        })
    }

    pub fn next_word(&mut self) -> &str {
        self.source_pool
            .choose(&mut self.rng)
            .expect("pool is non empty")
    }

    pub fn gen_words(&mut self, n: usize) -> Vec<String> {
        (0..n).map(|_| self.next_word().to_string()).collect()
    }
}
