use crate::{
    core::{
        config::{Gamemode, TestConfig},
        word_pool::WordSet,
    },
    preferences::Preferences,
    ui::theme::{Theme, BUILTIN_THEMES},
};

pub const TIMED_PRESETS: &[u64] = &[5, 15, 30, 60];
pub const WORDS_PRESETS: &[u64] = &[5, 15, 30, 60];
pub const MODE_OPTIONS: &[&str] = &["timed", "words", "zen"];
pub const BUILTIN_WORDLISTS: &[&str] = &["english_200"];
pub const BOOLEAN_OPTIONS: &[&str] = &["off", "on"];

enum InputKind {
    Number,
    FilePath,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuSection {
    Mode,
    Duration,
    Wordlist,
    Theme,
    Freedom,
}

const SECTIONS: &[MenuSection] = &[
    MenuSection::Mode,
    MenuSection::Duration,
    MenuSection::Wordlist,
    MenuSection::Theme,
    MenuSection::Freedom,
];

#[derive(Clone, Debug)]
pub struct TextInput {
    pub buffer: String,
    pub digits_only: bool, // Probs nicer ways to do this
}

impl TextInput {
    fn new(digits_only: bool) -> Self {
        Self {
            buffer: String::new(),
            digits_only,
        }
    }

    pub fn push(&mut self, c: char) {
        if self.digits_only && !c.is_ascii_digit() {
            return;
        }

        self.buffer.push(c);
    }

    pub fn pop(&mut self) {
        self.buffer.pop(); // INteresting that we can pop an empty string
    }
}

#[derive(Clone, Debug)]
pub struct MenuState {
    pub focused_section: MenuSection,

    /// 0 = Timed, 1 = Words, 2 = Zen; TODO: enum could be better but its literally only ever 3
    pub selected_mode: usize,

    /// Index into the active duration/word count preset array (+ custom slot) Maybe bad design to put it together but whatever
    pub selected_duration: usize,

    /// Index into wordlist options (builtins + custom slot=path)
    pub selected_wordlist: usize,

    /// Index into theme options (builtins + custom slot=path)
    pub selected_theme: usize,

    /// 0 = off, 1 = on
    pub selected_freedom: usize,

    /// Active text input when editing a custom value
    custom_input: Option<TextInput>,

    /// Validation error message shown on stupid user inputs (jk love yall)
    error_message: Option<&'static str>,

    /// Custom values that have been confirmed by the user
    pub custom_duration: Option<u64>,
    pub custom_wordlist: Option<String>,
    pub custom_theme: Option<String>,
}

impl MenuState {
    pub fn new(prefs: &Preferences) -> Self {
        let selected_mode = match &prefs.mode {
            Gamemode::Timed(_) => 0,
            Gamemode::Words(_) => 1,
            Gamemode::Zen => 2,
        };

        let selected_duration = match &prefs.mode {
            Gamemode::Timed(secs) => TIMED_PRESETS
                .iter()
                .position(|&p| p == *secs)
                .unwrap_or(TIMED_PRESETS.len()), // custom slot, pretty neat right
            Gamemode::Words(count) => WORDS_PRESETS
                .iter()
                .position(|&p| p == *count)
                .unwrap_or(WORDS_PRESETS.len()),
            Gamemode::Zen => 0,
        };

        let custom_duration = match &prefs.mode {
            Gamemode::Timed(secs) if !TIMED_PRESETS.contains(secs) => Some(*secs),
            Gamemode::Words(count) if !WORDS_PRESETS.contains(count) => Some(*count),
            _ => None,
        };

        let selected_wordlist = BUILTIN_WORDLISTS
            .iter()
            .position(|&w| w == prefs.wordset)
            .unwrap_or(BUILTIN_WORDLISTS.len());

        let custom_wordlist = if selected_wordlist == BUILTIN_WORDLISTS.len() {
            Some(prefs.wordset.clone())
        } else {
            None
        };

        let selected_theme = BUILTIN_THEMES
            .iter()
            .position(|&t| t == prefs.theme)
            .unwrap_or(BUILTIN_THEMES.len());

        let custom_theme = if selected_theme == BUILTIN_THEMES.len() {
            Some(prefs.theme.clone())
        } else {
            None
        };

        let selected_freedom = if prefs.freedom_mode { 1 } else { 0 };

        Self {
            focused_section: MenuSection::Mode,
            selected_mode,
            selected_duration,
            selected_wordlist,
            selected_theme,
            selected_freedom,
            custom_input: None,
            error_message: None,
            custom_duration,
            custom_wordlist,
            custom_theme,
        }
    }

    pub fn is_inputting(&self) -> bool {
        self.custom_input.is_some()
    }

    pub fn input_buffer(&self) -> &str {
        self.custom_input
            .as_ref()
            .map(|ti| ti.buffer.as_str())
            .unwrap_or("")
    }

    // --

    pub fn error_message(&self) -> Option<&'static str> {
        self.error_message
    }

    pub fn set_error(&mut self, msg: &'static str) {
        self.error_message = Some(msg);
    }

    pub fn confirm_input(&mut self) {
        let input = match self.custom_input.take() {
            Some(input) if !input.buffer.is_empty() => input,
            _ => return,
        };

        match self.focused_section {
            MenuSection::Duration => match input.buffer.parse::<u64>() {
                Ok(val) if val > 0 => {
                    self.custom_duration = Some(val);
                }
                _ => self.set_error("invalid number"),
            },
            MenuSection::Wordlist => match WordSet::load(&input.buffer) {
                Ok(_) => {
                    self.custom_wordlist = Some(input.buffer);
                }
                Err(_) => self.set_error("could not load wordlist"),
            },
            MenuSection::Theme => match Theme::load(&input.buffer) {
                Ok(_) => {
                    self.custom_theme = Some(input.buffer);
                }
                Err(_) => self.set_error("could not load theme"),
            },
            MenuSection::Mode | MenuSection::Freedom => {}
        }
    }

    // --

    pub fn cancel_input(&mut self) {
        self.custom_input = None;
    }

    pub fn input_char(&mut self, c: char) {
        if let Some(input) = &mut self.custom_input {
            input.push(c);
        }
    }

    pub fn input_backspace(&mut self) {
        if let Some(input) = &mut self.custom_input {
            input.pop();
        }
    }

    pub fn is_custom_selected(&self) -> bool {
        match self.focused_section {
            MenuSection::Mode | MenuSection::Freedom => false,
            // TODO: Actually i kinda hate this variable name but ill refacotr later
            MenuSection::Duration => match self.selected_mode {
                0 => self.selected_duration == TIMED_PRESETS.len(),
                1 => self.selected_duration == WORDS_PRESETS.len(),
                _ => false,
            },
            MenuSection::Wordlist => self.selected_wordlist == BUILTIN_WORDLISTS.len(),
            MenuSection::Theme => self.selected_theme == BUILTIN_THEMES.len(),
        }
    }

    pub fn enter_input_mode(&mut self) {
        // TODO: I dont like this
        let digits_only = matches!(self.focused_section, MenuSection::Duration);
        self.custom_input = Some(TextInput::new(digits_only));
    }

    pub fn select_next(&mut self) {
        // + 1 are for custom
        match self.focused_section {
            MenuSection::Mode => {
                self.selected_mode = (self.selected_mode + 1) % MODE_OPTIONS.len();
            }
            MenuSection::Duration => {
                let max = self.duration_option_count(); // The +1 is in this already
                self.selected_duration = (self.selected_duration + 1) % max;
            }
            MenuSection::Wordlist => {
                self.selected_wordlist = (self.selected_wordlist + 1) % BUILTIN_WORDLISTS.len() + 1;
            }
            MenuSection::Theme => {
                self.selected_theme = (self.selected_theme + 1) % BUILTIN_THEMES.len() + 1;
            }
            MenuSection::Freedom => {
                // It doesnt look so coherent/pretty but its cleaner than a modulus....
                self.selected_freedom = 1 - self.selected_freedom;
            }
        }
    }

    pub fn shift_next(&mut self) {
        let current = SECTIONS
            .iter()
            .position(|&s| s == self.focused_section)
            .unwrap_or(0);

        for i in 1..SECTIONS.len() {
            let next = (current + i) % SECTIONS.len();

            if !self.should_skip_section(SECTIONS[next]) {
                self.focused_section = SECTIONS[next];
                return;
            }
        }
    }

    pub fn shift_prev(&mut self) {
        let current = SECTIONS
            .iter()
            .position(|&s| s == self.focused_section)
            .unwrap_or(0);

        for i in 1..SECTIONS.len() {
            let next = (current + SECTIONS.len() - i) % SECTIONS.len();

            if !self.should_skip_section(SECTIONS[next]) {
                self.focused_section = SECTIONS[next];
                return;
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.focused_section {
            MenuSection::Mode => {
                let max = MODE_OPTIONS.len();
                self.selected_mode = (self.selected_mode + max - 1) % max;
            }
            MenuSection::Duration => {
                let max = self.duration_option_count();
                self.selected_duration = (self.selected_duration + max - 1) % max;
            }
            MenuSection::Wordlist => {
                let max = BUILTIN_WORDLISTS.len() + 1;
                self.selected_wordlist = (self.selected_wordlist + 1) % max;
            }
            MenuSection::Theme => {
                let max = BUILTIN_THEMES.len() + 1;
                self.selected_theme = (self.selected_theme + 1) % max;
            }
            MenuSection::Freedom => {
                // It doesnt look so coherent/pretty but its cleaner than a modulus....
                self.selected_freedom = 1 - self.selected_freedom;
            }
        }
    }

    pub fn to_test_config(&self) -> TestConfig {
        let wordset = if self.selected_wordlist < BUILTIN_WORDLISTS.len() {
            BUILTIN_WORDLISTS[self.selected_wordlist].to_string()
        } else {
            self.custom_wordlist
                .clone()
                .unwrap_or_else(|| "english_200".to_string()) // TODO: Should not hardcode default here
        };

        let mode = match self.selected_mode {
            0 => {
                let secs = if self.selected_mode < TIMED_PRESETS.len() {
                    TIMED_PRESETS[self.selected_mode]
                } else {
                    // TODO: THE DEFAULT SHOULD BE SET ESLEWHERE
                    self.custom_duration.unwrap_or(30)
                };

                Gamemode::Timed(secs)
            }

            1 => {
                let count = if self.selected_mode < TIMED_PRESETS.len() {
                    TIMED_PRESETS[self.selected_mode]
                } else {
                    self.custom_duration.unwrap_or(50)
                };

                Gamemode::Words(count)
            }
            _ => Gamemode::Zen,
        };

        TestConfig {
            wordset,
            has_punctuation: false, // TODO
            has_numbers: false,     // TODO
            freedom_mode: self.selected_freedom == 1,
            mode,
        }
    }

    // --

    // TODO: What was I smoking when i set this name.... ig i cant really think of a better one rnow tbh but
    fn duration_option_count(&self) -> usize {
        match self.selected_mode {
            0 => TIMED_PRESETS.len() + 1,
            1 => WORDS_PRESETS.len() + 1,
            _ => 1, // Shouldnt even be reachable but
        }
    }

    fn should_skip_section(&self, section: MenuSection) -> bool {
        match section {
            MenuSection::Duration => self.selected_mode == 2,
            _ => false,
        }
    }
}
