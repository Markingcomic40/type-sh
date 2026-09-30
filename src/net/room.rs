use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::core::config::Rules;
use crate::net::protocol::{Id, Phase, Player, PlayerColor, ToClient, ToServer};
use crate::net::server::Event;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 4;
const COUNTDOWN: Duration = Duration::from_secs(3);
const RESULTS_TIMEOUT: Duration = Duration::from_secs(8);

// Everyone gets their own colour
const _: () = assert!(MAX_PLAYERS <= PlayerColor::ALL.len());

#[derive(Debug, PartialEq)]
pub enum Out {
    To(Id, ToClient),
    All(ToClient),
    Kick(Id),
}

enum Stage {
    Lobby,
    Racing { finished: HashSet<Id> },
    Results { until: Instant },
}

pub struct Room {
    rules: Rules,
    pool: Vec<String>,
    players: Vec<Player>,
    stage: Stage,
}

impl Room {
    pub fn new(rules: Rules, pool: Vec<String>) -> Self {
        Self {
            rules,
            pool,
            players: Vec::new(),
            stage: Stage::Lobby,
        }
    }

    pub fn handle(&mut self, event: Event, now: Instant) -> Vec<Out> {
        match event {
            Event::Joined(_) => Vec::new(),
            Event::Message(id, ToServer::Greetings { version, name }) => {
                self.greet(id, version, name, now)
            }
            Event::Message(id, msg) if self.is_player(id) => self.message(id, msg, now),
            Event::Message(..) => Vec::new(),
            Event::Left(id) => self.leave(id, now),
        }
    }

    pub fn tick(&mut self, now: Instant) -> Vec<Out> {
        match self.stage {
            Stage::Results { until } if now >= until => self.back_to_lobby(now),
            _ => Vec::new(),
        }
    }

    pub fn set_rules(&mut self, rules: Rules, pool: Vec<String>, now: Instant) -> Vec<Out> {
        if !matches!(self.stage, Stage::Lobby) || rules == self.rules {
            return Vec::new();
        }

        self.rules = rules;
        self.pool = pool;
        self.unready();

        vec![self.snapshot(now)]
    }

    fn greet(&mut self, id: Id, version: String, name: String, now: Instant) -> Vec<Out> {
        if self.is_player(id) {
            return Vec::new();
        }

        let refusal = if version != VERSION {
            Some(format!(
                "version mismatch: host has {VERSION}, you have {version}; whoever is older should update"
            ))
        } else if matches!(self.stage, Stage::Racing { .. }) {
            Some("a race is in progress, try again in a moment".to_owned())
        } else if self.players.len() >= MAX_PLAYERS {
            Some("the room is full type sh".to_owned())
        } else {
            None
        };

        if let Some(reason) = refusal {
            return vec![Out::To(id, ToClient::Rejected { reason }), Out::Kick(id)];
        }

        let color = PlayerColor::ALL
            .into_iter()
            .find(|c| self.players.iter().all(|p| p.color != *c))
            .expect("fewer players than colours type sh");

        self.players.push(Player {
            id,
            name,
            color,
            ready: false,
        });

        vec![Out::To(id, ToClient::Welcome { id }), self.snapshot(now)]
    }

    fn message(&mut self, id: Id, msg: ToServer, now: Instant) -> Vec<Out> {
        let racing = matches!(self.stage, Stage::Racing { .. });

        match msg {
            ToServer::Greetings { .. } => Vec::new(),
            ToServer::Ready { ready } if !racing => {
                for p in self.players.iter_mut().filter(|p| p.id == id) {
                    p.ready = ready;
                }

                if self.players.len() >= MIN_PLAYERS && self.players.iter().all(|p| p.ready) {
                    self.start(now)
                } else {
                    vec![self.snapshot(now)]
                }
            }
            ToServer::Typed { keys, wpm } if racing => {
                vec![Out::All(ToClient::Typed { id, keys, wpm })]
            }
            ToServer::Finished { report } => {
                let Stage::Racing { finished } = &mut self.stage else {
                    return Vec::new();
                };

                if !finished.insert(id) {
                    return Vec::new();
                }

                let mut out = vec![Out::All(ToClient::Finished { id, report })];

                if self.everyone_finished() {
                    out.push(self.to_results(now));
                }
                out
            }
            ToServer::Cancel if !matches!(self.stage, Stage::Lobby) => self.back_to_lobby(now),
            _ => Vec::new(),
        }
    }

    fn leave(&mut self, id: Id, now: Instant) -> Vec<Out> {
        if !self.is_player(id) {
            return Vec::new();
        }

        self.players.retain(|p| p.id != id);

        let mut out = vec![Out::All(ToClient::Left { id })];

        // The last one still racing leaving means everyone left is done
        if matches!(self.stage, Stage::Racing { .. }) && self.everyone_finished() {
            out.push(self.to_results(now));
        } else {
            out.push(self.snapshot(now));
        }

        out
    }

    fn start(&mut self, now: Instant) -> Vec<Out> {
        self.unready();

        self.stage = Stage::Racing {
            finished: HashSet::new(),
        };

        vec![
            Out::All(ToClient::Start {
                rules: self.rules.clone(),
                words: self.pool.clone(),
                seed: rand::random(),
                countdown_ms: COUNTDOWN.as_millis() as u64,
            }),
            self.snapshot(now),
        ]
    }

    fn to_results(&mut self, now: Instant) -> Out {
        self.stage = Stage::Results {
            until: now + RESULTS_TIMEOUT,
        };

        self.snapshot(now)
    }

    fn back_to_lobby(&mut self, now: Instant) -> Vec<Out> {
        self.unready();
        self.stage = Stage::Lobby;

        vec![self.snapshot(now)]
    }

    fn everyone_finished(&self) -> bool {
        match &self.stage {
            Stage::Racing { finished } => self.players.iter().all(|p| finished.contains(&p.id)),
            _ => false,
        }
    }

    fn is_player(&self, id: Id) -> bool {
        self.players.iter().any(|p| p.id == id)
    }

    fn unready(&mut self) {
        for p in &mut self.players {
            p.ready = false;
        }
    }

    fn snapshot(&self, now: Instant) -> Out {
        let phase = match self.stage {
            Stage::Lobby => Phase::Lobby,
            Stage::Racing { .. } => Phase::Racing,
            Stage::Results { until } => Phase::Results {
                back_in_ms: until.saturating_duration_since(now).as_millis() as u64,
            },
        };

        Out::All(ToClient::Room {
            players: self.players.clone(),
            rules: self.rules.clone(),
            phase,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Limit;
    use crate::core::stats::{CharCounts, Report};
    use crate::net::protocol::Keypress;

    fn rules(words: u64) -> Rules {
        Rules {
            limit: Limit::Words(words),
            wordlist: "english".into(),
            freedom: false,
        }
    }

    fn room() -> Room {
        Room::new(rules(10), vec!["a".into(), "b".into()])
    }

    fn say(room: &mut Room, id: Id, msg: ToServer, now: Instant) -> Vec<Out> {
        room.handle(Event::Message(id, msg), now)
    }

    fn join(room: &mut Room, id: Id, now: Instant) -> Vec<Out> {
        let greetings = ToServer::Greetings {
            version: VERSION.into(),
            name: format!("p{id}"),
        };
        say(room, id, greetings, now)
    }

    fn ready(room: &mut Room, id: Id, now: Instant) -> Vec<Out> {
        say(room, id, ToServer::Ready { ready: true }, now)
    }

    fn finish(room: &mut Room, id: Id, now: Instant) -> Vec<Out> {
        let report = Report::new(&[], 0, Duration::ZERO, CharCounts::default());
        say(room, id, ToServer::Finished { report }, now)
    }

    /// The last room snapshot in `out`, which is what clients end up showing
    fn last_room(out: &[Out]) -> (&[Player], &Phase) {
        out.iter()
            .rev()
            .find_map(|o| match o {
                Out::All(ToClient::Room { players, phase, .. }) => Some((&players[..], phase)),
                _ => None,
            })
            .expect("a room snapshot")
    }

    fn started(out: &[Out]) -> bool {
        out.iter()
            .any(|o| matches!(o, Out::All(ToClient::Start { .. })))
    }

    /// Two players in a race that just started
    fn racing(now: Instant) -> Room {
        let mut room = room();
        join(&mut room, 0, now);
        join(&mut room, 1, now);
        ready(&mut room, 0, now);
        assert!(started(&ready(&mut room, 1, now)));
        room
    }

    #[test]
    fn joining_gets_a_welcome_and_the_first_free_colour() {
        let now = Instant::now();
        let mut room = room();
        join(&mut room, 0, now);
        join(&mut room, 1, now);
        room.handle(Event::Left(0), now);

        let out = join(&mut room, 2, now);
        assert_eq!(out[0], Out::To(2, ToClient::Welcome { id: 2 }));
        let (players, phase) = last_room(&out);
        let colors: Vec<_> = players.iter().map(|p| (p.id, p.color)).collect();
        assert_eq!(colors, [(1, PlayerColor::Green), (2, PlayerColor::Cyan)]);
        assert_eq!(*phase, Phase::Lobby);
    }

    #[test]
    fn a_different_version_is_told_why_and_kicked() {
        let mut room = room();
        let greetings = ToServer::Greetings {
            version: "0.0.1".into(),
            name: "old".into(),
        };

        let out = say(&mut room, 0, greetings, Instant::now());
        assert!(
            matches!(&out[..], [Out::To(0, ToClient::Rejected { reason }), Out::Kick(0)] if reason.contains("0.0.1"))
        );
    }

    #[test]
    fn the_room_holds_four() {
        let now = Instant::now();
        let mut room = room();
        for id in 0..4 {
            join(&mut room, id, now);
        }

        let out = join(&mut room, 4, now);
        assert!(matches!(
            &out[..],
            [Out::To(4, ToClient::Rejected { .. }), Out::Kick(4)]
        ));
    }

    #[test]
    fn strangers_are_ignored_until_they_say_hi() {
        let mut room = room();
        assert!(ready(&mut room, 0, Instant::now()).is_empty());
    }

    #[test]
    fn one_ready_player_is_not_a_race() {
        let now = Instant::now();
        let mut room = room();
        join(&mut room, 0, now);

        assert!(!started(&ready(&mut room, 0, now)));
    }

    #[test]
    fn nobody_joins_mid_race() {
        let now = Instant::now();
        let mut room = racing(now);

        let out = join(&mut room, 2, now);
        assert!(matches!(
            &out[..],
            [Out::To(2, ToClient::Rejected { .. }), Out::Kick(2)]
        ));
    }

    #[test]
    fn typing_is_passed_on_with_who_sent_it() {
        let now = Instant::now();
        let mut room = racing(now);
        let keys = vec![Keypress::Char('a'), Keypress::Backspace];
        let typed = ToServer::Typed {
            keys: keys.clone(),
            wpm: 70.0,
        };

        let out = say(&mut room, 1, typed, now);
        assert_eq!(
            out,
            [Out::All(ToClient::Typed {
                id: 1,
                keys,
                wpm: 70.0
            })]
        );
    }

    #[test]
    fn everyone_finishing_shows_results_and_readying_again_rematches() {
        let now = Instant::now();
        let mut room = racing(now);

        assert!(finish(&mut room, 0, now).len() == 1);
        let out = finish(&mut room, 1, now);
        assert_eq!(
            *last_room(&out).1,
            Phase::Results {
                back_in_ms: RESULTS_TIMEOUT.as_millis() as u64
            }
        );

        ready(&mut room, 0, now);
        assert!(started(&ready(&mut room, 1, now)));
    }

    #[test]
    fn results_run_out_back_to_the_lobby() {
        let now = Instant::now();
        let mut room = racing(now);
        finish(&mut room, 0, now);
        finish(&mut room, 1, now);

        let second = Duration::from_secs(1);
        assert!(room.tick(now + RESULTS_TIMEOUT - second).is_empty());
        let out = room.tick(now + RESULTS_TIMEOUT + second);
        assert_eq!(*last_room(&out).1, Phase::Lobby);
    }

    #[test]
    fn anyone_cancelling_sends_everyone_back() {
        let now = Instant::now();
        let mut room = racing(now);
        finish(&mut room, 0, now);

        let out = say(&mut room, 0, ToServer::Cancel, now);
        let (players, phase) = last_room(&out);
        assert_eq!(*phase, Phase::Lobby);
        assert!(players.iter().all(|p| !p.ready));
    }

    #[test]
    fn the_last_racer_leaving_ends_the_race() {
        let now = Instant::now();
        let mut room = racing(now);
        finish(&mut room, 0, now);

        let out = room.handle(Event::Left(1), now);
        assert_eq!(out[0], Out::All(ToClient::Left { id: 1 }));
        assert!(matches!(last_room(&out).1, Phase::Results { .. }));
    }

    #[test]
    fn changing_the_rules_unreadies_everyone_but_only_in_the_lobby() {
        let now = Instant::now();
        let mut room = room();
        join(&mut room, 0, now);
        join(&mut room, 1, now);
        ready(&mut room, 0, now);

        assert!(room.set_rules(rules(10), Vec::new(), now).is_empty());
        let out = room.set_rules(rules(50), Vec::new(), now);
        assert!(last_room(&out).0.iter().all(|p| !p.ready));

        let mut room = racing(now);
        assert!(room.set_rules(rules(50), Vec::new(), now).is_empty());
    }
}
