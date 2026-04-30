pub const DEFAULT_WORDSET: &str = "english_200";
pub const DEFAULT_TIMED_SECS: u64 = 5;

#[derive(Clone, Debug)]
pub enum Gamemode {
    Timed(u64),
    Words(u64),
    Zen,
}

#[derive(Clone, Debug)]
pub struct TestConfig {
    pub wordset: String,
    pub has_punctuation: bool,
    pub has_numbers: bool,
    pub freedom_mode: bool,
    pub mode: Gamemode,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            wordset: DEFAULT_WORDSET.to_string(),
            has_punctuation: false,
            has_numbers: false,
            freedom_mode: true,
            mode: Gamemode::Timed(DEFAULT_TIMED_SECS),
        }
    }
}
