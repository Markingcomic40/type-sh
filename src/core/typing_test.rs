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
            .gen_words(100)
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
        self.current_input.push(c);
    }

    pub fn append_words(&mut self) {}
}
