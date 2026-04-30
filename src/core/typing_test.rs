use std::time::Instant;

use crate::core::config::{Gamemode, TestConfig};
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
    start_time: Option<Instant>,
}

impl TypingTest {
    pub fn new(config: TestConfig) -> Result<Self> {
        let mut wordlist = WordList::new(&config.wordset)?;

        let n_words = match config.mode {
            Gamemode::Timed(_) => 20,
            Gamemode::Words(n) => n,
            Gamemode::Zen => 20,
        };

        let words = wordlist
            .gen_words(n_words as usize)
            .into_iter()
            .map(WordState::new)
            .collect();

        Ok(Self {
            config,
            wordlist,
            words,
            current_word_index: 0,
            current_input: String::new(),
            start_time: None,
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

    pub fn remaining_secs(&self) -> Option<u64> {
        if let Gamemode::Timed(t) = self.config.mode {
            let elapsed = self.elapsed_secs_u64();
            Some(t.saturating_sub(elapsed))
        } else {
            None
        }
    }

    pub fn elapsed_secs_u64(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
    }

    pub fn has_started(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn handle_space(&mut self) {
        if self.current_word_index > self.words.len() {
            return;
        }

        self.words[self.current_word_index].typed = Some(self.current_input.clone());

        self.current_word_index += 1;
        self.current_input.clear()
    }

    pub fn handle_backspace(&mut self) {
        if self.current_input.is_empty() && self.config.freedom_mode && self.current_word_index > 0
        {
            self.current_word_index -= 1;
            self.current_input = self.words[self.current_word_index]
                .typed
                .take()
                .unwrap_or_default()
        } else {
            self.current_input.pop();
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.has_started() {
            self.start_time = Some(Instant::now());
        }

        self.current_input.push(c);
    }

    pub fn append_words(&mut self) {
        if matches!(self.config.mode, Gamemode::Timed(_) | Gamemode::Zen) {
            let new_words = self.wordlist.gen_words(20);
            self.words.extend(new_words.into_iter().map(WordState::new));
        }
    }

    pub fn is_finished(&self) -> bool {
        match self.config.mode {
            Gamemode::Timed(t) => self.elapsed_secs_u64() >= t,
            Gamemode::Words(_) => self.current_word_index >= self.words.len(),
            Gamemode::Zen => false,
        }
    }
}
