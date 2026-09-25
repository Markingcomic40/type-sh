use serde::{Deserialize, Serialize};

/// What ends a test.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Limit {
    Time(u64),
    Words(u64),
    None,
}

#[derive(Clone, Debug)]
pub struct TestConfig {
    pub limit: Limit,
    pub wordlist: String,
    pub freedom: bool,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    pub limit: Limit,
    /// Locally a builtin name or a file path; in multiplayer=display name words we send
    pub wordlist: String,
    pub freedom: bool,
}

impl Rules {
    pub fn with_seed(self, seed: u64) -> TestConfig {
        TestConfig {
            limit: self.limit,
            wordlist: self.wordlist,
            freedom: self.freedom,
            seed,
        }
    }
}
