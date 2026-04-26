use crate::core::config::TestConfig;
use crate::core::word_pool::WordList;
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
    wordlist: WordList,
    words: Vec<WordState>,
    current_word_index: usize,
    current_input: String,
}

impl TypingTest {
    pub fn new(config: TestConfig) -> Result<Self> {
        let mut wordlist = WordList::new(&config.wordset)?;

        let words = wordlist
            .gen_words(10)
            .into_iter()
            .map(WordState::new)
            .collect();

        Ok(Self {
            config,
            wordlist,
            words,
            current_word_index: 0,
            current_input: String::new(),
        })
    }

    pub fn words(&self) -> &Vec<WordState> {
        &self.words
    }

    pub fn current_word_index(&self) -> usize {
        self.current_word_index
    }

    pub fn current_input(&self) -> &str {
        &self.current_input
    }

    pub fn append_words(&mut self) {}
}
