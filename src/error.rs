// This blog was an interesting read but maybe a bit too overcomplciated for this proejct atm but Ill try something similar and kinda against what it says but cause it makes sense here haha
// https://gist.github.com/quad/a8a7cc87d1401004c6a8973947f20365
// https://crates.io/crates/thiserror

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not load word set '{name}'")]
    WordSetLoad {
        name: String,
        source: std::io::Error,
    },

    #[error("Word set '{0}' is empty")]
    EmptyWordSet(String),

    #[error("Failed to load theme '{name}': {source}")]
    ThemeLoad {
        name: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub type Result<T> = std::result::Result<T, AppError>;
