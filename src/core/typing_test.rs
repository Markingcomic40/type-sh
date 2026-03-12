use crate::core::config::TestConfig;
use crate::core::word_pool::WordPool;
use crate::error::Result;

pub struct WordState {
    pub target: String, // NOTE: Could consider storing an index instead magari...
    pub typed: Option<String>,
}

impl WordState {
    pub fn new(target: String) -> Self {
        Self {
            target,
            typed: None,
        }
    }
}

pub struct TypingTest {
    config: TestConfig,
    words: Vec<WordState>,
}

impl TypingTest {
    pub fn new(config: TestConfig) -> Result<Self> {
        let mut wordlist = WordPool::new(&config.wordset)?;
        let words = wordlist
            .gen_words(10)
            .into_iter()
            .map(WordState::new)
            .collect();

        Ok(Self { config, words })
    }

    pub fn words(&self) -> &Vec<WordState> {
        &self.words
    }
}
