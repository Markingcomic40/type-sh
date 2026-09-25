use serde::{Deserialize, Serialize};

use crate::core::config::Rules;
use crate::core::stats::Report;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerColor {
    Cyan,
    Green,
    Purple,
    Blue,
    Pink,
    Yellow,
    Orange,
}

impl PlayerColor {
    pub const ALL: [PlayerColor; 7] = [
        PlayerColor::Cyan,
        PlayerColor::Green,
        PlayerColor::Purple,
        PlayerColor::Blue,
        PlayerColor::Pink,
        PlayerColor::Yellow,
        PlayerColor::Orange,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub slot: u8,
    pub name: String,
    pub color: PlayerColor,
    pub ready: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToServer {
    /// DONT CHANGE THIS SHAPE
    Greetings {
        version: String,
        name: String,
    },

    /// Doesnt act toggle state
    Ready {
        ready: bool,
    },

    /// Where the caret is; conn tells whomai no need slot
    Progress {
        word: usize,
        char: usize,
        wpm: f64,
    },

    Finished {
        report: Report,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToClient {
    /// Which row client is
    Welcome {
        slot: u8,
    },

    /// Eg version mismatch
    Rejected {
        reason: String,
    },

    /// The whole room, resent on every change
    Room {
        players: Vec<Player>,
        rules: Rules,
    },

    Start {
        rules: Rules,
        /// The pool, not the test itself
        words: Vec<String>,
        seed: u64,
        countdown_ms: u64,
    },

    /// Clients progress tagged with whoami
    Moved {
        slot: u8,
        word: usize,
        char: usize,
        wpm: f64,
    },

    Finished {
        slot: u8,
        report: Report,
    },
    
	Left {
        slot: u8,
    },
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::time::Duration;

    use serde::de::DeserializeOwned;

    use super::*;
    use crate::core::config::Limit;
    use crate::core::stats::{CharCounts, Second};

    /// Messages are newline framed, so each has to come out as exactly one line
    fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(msg: T) {
        let line = serde_json::to_string(&msg).unwrap();
        assert!(!line.contains('\n'), "{line}");
        assert_eq!(serde_json::from_str::<T>(&line).unwrap(), msg);
    }

    fn rules() -> Rules {
        Rules {
            limit: Limit::Time(30),
            wordlist: "english".into(),
            freedom: false,
        }
    }

    // Floats kept to halves and quarters: serde_json can read some other
    // values back one bit off, which is fine to show but not to assert_eq
    fn report() -> Report {
        Report {
            wpm: 87.5,
            raw: 92.25,
            accuracy: 96.5,
            consistency: 80.0,
            duration: Duration::from_millis(30_000),
            chars: CharCounts {
                correct: 120,
                incorrect: 3,
                extra: 1,
                missed: 2,
            },
            timeline: vec![Second {
                wpm: 80.0,
                raw: 85.5,
                errors: 1,
            }],
        }
    }

    #[test]
    fn messages_to_the_server_survive_the_wire() {
        round_trip(ToServer::Greetings {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            name: "bobby".into(),
        });
        round_trip(ToServer::Ready { ready: true });
        round_trip(ToServer::Progress {
            word: 12,
            char: 3,
            wpm: 71.5,
        });
        round_trip(ToServer::Finished { report: report() });
    }

    #[test]
    fn messages_to_clients_survive_the_wire() {
        round_trip(ToClient::Welcome { slot: 2 });
        round_trip(ToClient::Room {
            players: vec![Player {
                slot: 0,
                name: "sammy".into(),
                color: PlayerColor::Cyan,
                ready: true,
            }],
            rules: rules(),
        });
        round_trip(ToClient::Start {
            rules: rules(),
            words: vec!["type".into(), "sh".into(), "bro".into()],
            seed: u64::MAX,
            countdown_ms: 3000,
        });
        round_trip(ToClient::Moved {
            slot: 1,
            word: 4,
            char: 0,
            wpm: 60.0,
        });
        round_trip(ToClient::Finished {
            slot: 1,
            report: report(),
        });
    }

    #[test]
    fn a_name_with_a_newline_still_fits_on_one_line() {
        round_trip(ToServer::Greetings {
            version: "0.2.0".into(),
            name: "vege\ntable".into(),
        });
    }
}
