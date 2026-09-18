use std::time::{Duration, Instant};

use crate::core::config::{Limit, TestConfig};
use crate::core::stats::{CharCounts, Key, Keystroke, Report};
use crate::core::word_pool::WordList;
use crate::error::Result;

const LOOKAHEAD: usize = 50;

/// So a held key doesnt go off screen
const MAX_EXTRA: usize = 10;

/// How one character position of a word compares against what was typed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Correct(char),
    /// Holds expected
    Incorrect(char),
    /// Typed past the end of the word
    Extra(char),
    /// Not typed (yet)
    Untyped(char),
}

#[derive(Clone, Debug)]
pub struct Word {
    pub target: String,
    pub typed: String,
}

impl Word {
    fn new(target: String) -> Self {
        Self {
            target,
            typed: String::new(),
        }
    }

    pub fn is_correct(&self) -> bool {
        self.typed == self.target
    }

    /// Width on screen, which grows when characters are typed past the end.
    pub fn width(&self) -> usize {
        self.glyphs().count()
    }

    pub fn glyphs(&self) -> impl Iterator<Item = Glyph> + '_ {
        let mut target = self.target.chars();
        let mut typed = self.typed.chars();

        std::iter::from_fn(move || match (target.next(), typed.next()) {
            (Some(t), Some(c)) if t == c => Some(Glyph::Correct(t)),
            (Some(t), Some(_)) => Some(Glyph::Incorrect(t)),
            (None, Some(c)) => Some(Glyph::Extra(c)),
            (Some(t), None) => Some(Glyph::Untyped(t)),
            (None, None) => None,
        })
    }
}

pub struct TypingTest {
    config: TestConfig,
    source: WordList,
    words: Vec<Word>,
    /// Index of the word under the caret
    current: usize,
    /// Set by the first keystroke
    started: Option<Instant>,
    /// Set when the test is ended early; freezes the clock
    stopped: Option<Duration>,
    log: Vec<Keystroke>,
    /// Characters in correctly typed words so far, counting the space after each
    score: usize,
}

impl TypingTest {
    pub fn new(config: TestConfig) -> Result<Self> {
        let source = WordList::new(&config.wordlist)?;
        Ok(Self::with_source(config, source))
    }

    fn with_source(config: TestConfig, mut source: WordList) -> Self {
        let count = match config.limit {
            Limit::Words(n) => (n as usize).max(1),
            Limit::Time(_) | Limit::None => LOOKAHEAD,
        };
        let words = (0..count)
            .map(|_| Word::new(source.next_word().to_owned()))
            .collect();

        Self {
            config,
            source,
            words,
            current: 0,
            started: None,
            stopped: None,
            log: Vec::new(),
            score: 0,
        }
    }

    pub fn config(&self) -> &TestConfig {
        &self.config
    }

    pub fn words(&self) -> &[Word] {
        &self.words
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn has_started(&self) -> bool {
        self.started.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        match (self.started, self.stopped) {
            (_, Some(stopped)) => stopped,
            (Some(started), None) => started.elapsed(),
            (None, None) => Duration::ZERO,
        }
    }

    pub fn type_char(&mut self, c: char) {
        if self.is_finished() {
            return;
        }

        let at = self.clock();
        let word = &mut self.words[self.current];
        let pos = word.typed.chars().count();
        if pos >= word.target.chars().count() + MAX_EXTRA {
            return;
        }

        let hit = word.target.chars().nth(pos) == Some(c);
        word.typed.push(c);
        self.record(at, if hit { Key::Hit } else { Key::Miss });
    }

    pub fn space(&mut self) {
        // Space on an empty word shouldnt skip
        if self.is_finished() || self.words[self.current].typed.is_empty() {
            return;
        }

        let at = self.clock();
        let word = &self.words[self.current];
        let len = word.target.chars().count();
        let hit = word.typed.chars().count() == len;
        if word.is_correct() {
            self.score += len + 1;
        }

        self.current += 1;
        self.refill();
        self.record(at, if hit { Key::Hit } else { Key::Miss });
    }

    pub fn backspace(&mut self) {
        if self.is_finished() || self.words[self.current].typed.pop().is_some() {
            return;
        }
        if !self.config.freedom || self.current == 0 {
            return;
        }

        self.current -= 1;
        let word = &self.words[self.current];
        if word.is_correct() {
            self.score -= word.target.chars().count() + 1;
            let at = self.clock();
            self.record(at, Key::Erase);
        }
    }

    /// earlyt stop
    pub fn stop(&mut self) {
        self.stopped = Some(self.elapsed());
    }

    pub fn is_finished(&self) -> bool {
        if self.stopped.is_some() {
            return true;
        }

        match self.config.limit {
            Limit::Time(secs) => self.elapsed() >= Duration::from_secs(secs),
            Limit::Words(_) => {
                let last = self.words.len() - 1;
                self.current > last || self.current == last && self.words[last].is_correct()
            }
            Limit::None => false,
        }
    }

    pub fn report(&self) -> Report {
        Report::new(
            &self.log,
            self.final_score(),
            self.duration(),
            self.char_counts(),
        )
    }

    fn clock(&mut self) -> Duration {
        self.started.get_or_insert_with(Instant::now).elapsed()
    }

    fn record(&mut self, at: Duration, key: Key) {
        self.log.push(Keystroke {
            at,
            key,
            score: self.score,
        });
    }

    fn refill(&mut self) {
        if matches!(self.config.limit, Limit::Words(_)) {
            return;
        }
        while self.words.len() < self.current + LOOKAHEAD {
            let word = Word::new(self.source.next_word().to_owned());
            self.words.push(word);
        }
    }

    fn duration(&self) -> Duration {
        match self.config.limit {
            Limit::Time(secs) => self.elapsed().min(Duration::from_secs(secs)),
            // Idle time after the last keystroke shouldn't count against you.
            Limit::Words(_) | Limit::None => self.log.last().map_or(Duration::ZERO, |k| k.at),
        }
    }

    /// The score plus credit for the correctly typed part of the word under the caret.
    fn final_score(&self) -> usize {
        let partial = self
            .words
            .get(self.current)
            .filter(|w| w.target.starts_with(&w.typed))
            .map_or(0, |w| w.typed.chars().count());

        self.score + partial
    }

    fn char_counts(&self) -> CharCounts {
        let mut counts = CharCounts::default();

        for (i, word) in self.words.iter().enumerate().take(self.current + 1) {
            for glyph in word.glyphs() {
                match glyph {
                    Glyph::Correct(_) => counts.correct += 1,
                    Glyph::Incorrect(_) => counts.incorrect += 1,
                    Glyph::Extra(_) => counts.extra += 1,
                    Glyph::Untyped(_) if i < self.current => counts.missed += 1,
                    Glyph::Untyped(_) => {}
                }
            }
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(limit: Limit, words: &[&str]) -> TypingTest {
        let config = TestConfig {
            limit,
            wordlist: String::new(),
            freedom: true,
        };
        let words: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        let mut test = TypingTest::with_source(config, WordList::from_words(words.clone()));
        test.words = words.into_iter().map(Word::new).collect();
        test
    }

    fn type_str(test: &mut TypingTest, text: &str) {
        for c in text.chars() {
            match c {
                ' ' => test.space(),
                '<' => test.backspace(),
                c => test.type_char(c),
            }
        }
    }

    #[test]
    fn correct_words_score_their_length_plus_a_space() {
        let mut t = test(Limit::None, &["the", "fox", "ran"]);
        type_str(&mut t, "the fxo ");

        assert_eq!(t.score, 4);
        assert_eq!(t.current(), 2);
    }

    #[test]
    fn backspacing_into_a_correct_word_takes_it_off_the_score() {
        let mut t = test(Limit::None, &["the", "fox"]);
        type_str(&mut t, "the <");

        assert_eq!(t.current(), 0);
        assert_eq!(t.score, 0);
        assert_eq!(t.words()[0].typed, "the");
    }

    #[test]
    fn backspace_stays_put_without_freedom() {
        let mut t = test(Limit::None, &["the", "fox"]);
        t.config.freedom = false;
        type_str(&mut t, "the <");

        assert_eq!(t.current(), 1);
    }

    #[test]
    fn space_on_an_empty_word_does_nothing() {
        let mut t = test(Limit::None, &["the", "fox"]);
        type_str(&mut t, " ");

        assert_eq!(t.current(), 0);
        assert!(!t.has_started());
    }

    #[test]
    fn word_tests_end_once_the_last_word_is_right() {
        let mut t = test(Limit::Words(2), &["a", "bc"]);
        type_str(&mut t, "a b");
        assert!(!t.is_finished());

        type_str(&mut t, "c");
        assert!(t.is_finished());
        assert_eq!(t.final_score(), 4);
    }

    #[test]
    fn input_after_the_end_is_ignored() {
        let mut t = test(Limit::Words(1), &["a"]);
        type_str(&mut t, "abc <");

        assert_eq!(t.words()[0].typed, "a");
    }

    #[test]
    fn char_counts_match_monkeytype() {
        let mut t = test(Limit::None, &["abc", "de", "fg"]);
        type_str(&mut t, "axcz d f");

        let counts = t.char_counts();
        assert_eq!(
            counts,
            CharCounts {
                correct: 4,
                incorrect: 1,
                extra: 1,
                missed: 1,
            }
        );
    }

    #[test]
    fn glyphs_show_the_expected_char_for_typos() {
        let word = Word {
            target: "ab".into(),
            typed: "xbz".into(),
        };
        let glyphs: Vec<_> = word.glyphs().collect();

        assert_eq!(
            glyphs,
            [
                Glyph::Incorrect('a'),
                Glyph::Correct('b'),
                Glyph::Extra('z')
            ]
        );
    }
}
