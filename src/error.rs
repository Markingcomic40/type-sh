// This blog was an interesting read but maybe a bit too overcomplciated for this proejct atm but Ill try something similar and kinda against what it says but cause it makes sense here haha
// https://gist.github.com/quad/a8a7cc87d1401004c6a8973947f20365
// https://crates.io/crates/thiserror

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not read word list '{name}': {source}")]
    WordList {
        name: String,
        source: std::io::Error,
    },

    #[error("word list '{0}' is empty")]
    EmptyWordList(String),

    #[error("could not load theme '{name}': {source}")]
    Theme {
        name: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("could not save settings: {0}")]
    Settings(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
