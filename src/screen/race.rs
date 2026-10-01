use std::collections::HashMap;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::config::{Limit, TestConfig};
use crate::core::stats::Report;
use crate::core::typing_test::TypingTest;
use crate::net::protocol::{Id, Keypress, Player, ToServer};
use crate::screen::words::{self, Marker};
use crate::ui::frame::{Frame, Style};
use crate::ui::theme::Theme;

const MIN_BAR: u16 = 4;

#[derive(Clone, Copy, Default)]
struct Spot {
    word: usize,
    char: usize,
    wpm: f64,
}

/// Someone else's test, rebuilt by replaying the keys they send
struct Ghost {
    test: TypingTest,
    /// From their clock, since the replay's runs a little behind
    wpm: f64,
}

/// This player's test during a race, plus everyone else's replayed
pub struct Race {
    test: TypingTest,
    /// Typing unlocks once the countdown gets here
    go_at: Instant,
    /// Keys pressed since the last tick, still to be sent
    pending: Vec<Keypress>,
    sent_finished: bool,
    others: HashMap<Id, Ghost>,
    reports: HashMap<Id, Report>,
    /// Who to show once done; unset or finished means whoever's leading
    watching: Option<Id>,
}

impl Race {
    pub fn new(
        config: TestConfig,
        words: Vec<String>,
        go_at: Instant,
        others: impl IntoIterator<Item = Id>,
    ) -> Self {
        // Their keys arrive a moment late, so a replay can't be the one to
        // call time: it would cut off their last few. Their `Finished` ends it.
        let mut ghost = config.clone();
        if let Limit::Time(_) = ghost.limit {
            ghost.limit = Limit::None;
        }
        let others = others
            .into_iter()
            .map(|id| {
                let test = TypingTest::from_words(ghost.clone(), words.clone());
                (id, Ghost { test, wpm: 0.0 })
            })
            .collect();

        Self {
            test: TypingTest::from_words(config, words),
            go_at,
            pending: Vec::new(),
            sent_finished: false,
            others,
            reports: HashMap::new(),
            watching: None,
        }
    }

    pub fn is_done(&self) -> bool {
        self.sent_finished
    }

    pub fn report(&self, id: Id) -> Option<&Report> {
        self.reports.get(&id)
    }

    pub fn typed(&mut self, id: Id, keys: &[Keypress], wpm: f64) {
        if let Some(ghost) = self.others.get_mut(&id) {
            keys.iter().for_each(|&key| press(&mut ghost.test, key));
            ghost.wpm = wpm;
        }
    }

    pub fn finished(&mut self, id: Id, report: Report) {
        self.reports.insert(id, report);
    }

    pub fn handle_key(&mut self, key: KeyEvent, now: Instant) {
        if now < self.go_at || self.test.is_finished() {
            return;
        }

        let key = match key.code {
            KeyCode::Backspace => Keypress::Backspace,
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Keypress::Char(c)
            }
            _ => return,
        };
        press(&mut self.test, key);
        self.pending.push(key);
    }

    /// What to tell the others since last time. The keys that finished the
    /// test go out before the report, so everyone's replay ends where it did.
    pub fn tick(&mut self) -> Vec<ToServer> {
        let mut out = Vec::new();
        if !self.pending.is_empty() {
            out.push(ToServer::Typed {
                keys: std::mem::take(&mut self.pending),
                wpm: self.test.report().wpm,
            });
        }
        if self.test.is_finished() && !self.sent_finished {
            self.sent_finished = true;
            out.push(ToServer::Finished {
                report: self.test.report(),
            });
        }
        out
    }

    fn spot(&self, id: Id, me: Option<Id>) -> Spot {
        let (test, wpm) = if Some(id) == me {
            let wpm = if self.test.has_started() {
                self.test.report().wpm
            } else {
                0.0
            };
            (&self.test, wpm)
        } else {
            match self.others.get(&id) {
                Some(ghost) => (&ghost.test, ghost.wpm),
                None => return Spot::default(),
            }
        };

        let word = test.current();
        let char = test
            .words()
            .get(word)
            .map_or(0, |w| w.typed.chars().count());
        Spot { word, char, wpm }
    }

    /// Everyone else still typing, in room order
    fn racers<'a>(&self, players: &'a [Player], me: Option<Id>) -> Vec<&'a Player> {
        players
            .iter()
            .filter(|p| Some(p.id) != me && !self.reports.contains_key(&p.id))
            .collect()
    }

    pub fn watched<'a>(&self, players: &'a [Player], me: Option<Id>) -> Option<&'a Player> {
        let racers = self.racers(players, me);
        racers
            .iter()
            .find(|p| Some(p.id) == self.watching)
            .or_else(|| {
                racers.iter().max_by_key(|p| {
                    let spot = self.spot(p.id, me);
                    (spot.word, spot.char)
                })
            })
            .copied()
    }

    pub fn watch_next(&mut self, players: &[Player], me: Option<Id>, step: isize) {
        let racers = self.racers(players, me);
        if racers.is_empty() {
            return;
        }
        let at = self
            .watched(players, me)
            .and_then(|w| racers.iter().position(|p| p.id == w.id))
            .unwrap_or(0);
        let next = (at as isize + step).rem_euclid(racers.len() as isize) as usize;
        self.watching = Some(racers[next].id);
    }

    /// Whether there's still a race to show: your own, or someone to watch
    pub fn has_view(&self, players: &[Player], me: Option<Id>) -> bool {
        !self.sent_finished || self.watched(players, me).is_some()
    }

    /// Other racers' carets past `from`. Behind a caret the letters are
    /// showing whether they were typed right, so no markers there.
    fn markers(
        &self,
        theme: &Theme,
        players: &[Player],
        skip: Option<Id>,
        from: (usize, usize),
    ) -> Vec<Marker> {
        players
            .iter()
            .filter(|p| Some(p.id) != skip && !self.reports.contains_key(&p.id))
            .filter(|p| self.others.contains_key(&p.id))
            .filter_map(|p| {
                let spot = self.spot(p.id, None);
                ((spot.word, spot.char) > from).then(|| Marker {
                    word: spot.word,
                    char: spot.char,
                    color: theme.player(p.color),
                })
            })
            .collect()
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme, players: &[Player], me: Option<Id>) {
        if self.sent_finished {
            if let Some(watched) = self.watched(players, me) {
                self.draw_watching(f, theme, players, me, watched);
            }
            return;
        }

        let now = Instant::now();
        let area = words::area(f.area());

        let counter = if now < self.go_at {
            ((self.go_at - now).as_secs() + 1).to_string()
        } else {
            words::counter(&self.test)
        };
        f.print(area.x, area.y - 2, &counter, Style::fg(theme.accent));

        let mine = me.map_or_else(Spot::default, |me| self.spot(me, Some(me)));
        let markers = self.markers(theme, players, me, (mine.word, mine.char));
        words::draw(
            f,
            theme,
            area,
            self.test.words(),
            self.test.current(),
            now >= self.go_at,
            &markers,
        );

        let strip = area.bottom() + 2;
        self.draw_strip(f, theme, players, me, None, area.x, strip, area.width);
    }

    /// Someone else's race, exactly as their screen shows it
    fn draw_watching(
        &self,
        f: &mut Frame,
        theme: &Theme,
        players: &[Player],
        me: Option<Id>,
        watched: &Player,
    ) {
        let area = words::area(f.area());
        let spot = self.spot(watched.id, me);

        let mut x = area.x;
        if let Limit::Words(n) = self.test.config().limit {
            let counter = format!("{}/{n}", spot.word);
            x = f.print(x, area.y - 2, &counter, Style::fg(theme.accent)) + 2;
        }
        x = f.print(x, area.y - 2, "watching ", Style::fg(theme.dim));
        let color = theme.player(watched.color);
        f.print(x, area.y - 2, &watched.name, Style::fg(color).bold());

        let markers = self.markers(theme, players, Some(watched.id), (spot.word, spot.char));
        if let Some(ghost) = self.others.get(&watched.id) {
            let test = &ghost.test;
            words::draw(f, theme, area, test.words(), test.current(), true, &markers);
        }

        let strip = area.bottom() + 2;
        let watching = Some(watched.id);
        self.draw_strip(f, theme, players, me, watching, area.x, strip, area.width);
    }

    /// A progress bar per player
    #[allow(clippy::too_many_arguments)]
    fn draw_strip(
        &self,
        f: &mut Frame,
        theme: &Theme,
        players: &[Player],
        me: Option<Id>,
        watching: Option<Id>,
        x: u16,
        y: u16,
        width: u16,
    ) {
        let name_width = players
            .iter()
            .map(|p| p.name.chars().count())
            .max()
            .unwrap_or(0) as u16;
        let bar_x = x + 2 + name_width + 2;
        let bar = width
            .saturating_sub(bar_x - x + 2 + "100 wpm".len() as u16)
            .max(MIN_BAR);
        let leader = players
            .iter()
            .map(|p| self.spot(p.id, me).word)
            .max()
            .unwrap_or(0);

        for (i, player) in players.iter().enumerate() {
            let y = y + i as u16;
            let color = theme.player(player.color);
            let spot = self.spot(player.id, me);
            let report = self.reports.get(&player.id);

            let fraction = match (report, self.test.config().limit) {
                (Some(_), _) => 1.0,
                (None, Limit::Words(n)) => spot.word as f64 / n as f64,
                // Time races have no end to measure against, so the leader sets the bar
                (None, _) if leader == 0 => 0.0,
                (None, _) => spot.word as f64 / leader as f64,
            };
            let filled = (fraction.clamp(0.0, 1.0) * f64::from(bar)).round() as u16;

            if Some(player.id) == watching {
                f.put(x.saturating_sub(2), y, '›', Style::fg(theme.accent));
            }
            f.put(x, y, '●', Style::fg(color));
            let name = Style::fg(color);
            let name = if Some(player.id) == me {
                name.bold()
            } else {
                name
            };
            f.print(x + 2, y, &player.name, name);
            for j in 0..bar {
                let c = if j < filled { color } else { theme.faint };
                f.put(bar_x + j, y, '━', Style::fg(c));
            }

            let wpm = report.map_or(spot.wpm, |r| r.wpm);
            let style = if report.is_some() {
                Style::fg(theme.accent)
            } else {
                Style::fg(theme.dim)
            };
            f.print(bar_x + bar + 2, y, &format!("{wpm:.0} wpm"), style);
        }
    }
}

fn press(test: &mut TypingTest, key: Keypress) {
    match key {
        Keypress::Backspace => test.backspace(),
        Keypress::Char(' ') => test.space(),
        Keypress::Char(c) => test.type_char(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(text: &str) -> Vec<Keypress> {
        text.chars()
            .map(|c| match c {
                '<' => Keypress::Backspace,
                c => Keypress::Char(c),
            })
            .collect()
    }

    #[test]
    fn replaying_keys_rebuilds_their_screen() {
        let config = TestConfig {
            limit: Limit::Time(30),
            wordlist: "english".into(),
            freedom: true,
            seed: 7,
        };
        let pool = crate::core::word_pool::load("english").unwrap();
        let mut racer = TypingTest::from_words(config.clone(), pool.clone());

        // Typos, extras, skips, and backspacing into a finished word, over
        // enough words that the test has to draw more than its first batch
        let mut script = Vec::new();
        for i in 0..70 {
            let target = racer.words()[racer.current()].target.clone();
            let typed = match i % 4 {
                0 => format!("x{}", &target[1..]),
                1 => format!("{target}zz"),
                2 => target.chars().take(1).collect(),
                // back into the finished word to change its last letter
                _ => format!("{target} <<x"),
            };
            let pressed = keys(&format!("{typed} "));
            pressed.iter().for_each(|&k| press(&mut racer, k));
            script.extend(pressed);
        }

        let mut race = Race::new(config, pool, Instant::now(), [1]);
        for chunk in script.chunks(5) {
            race.typed(1, chunk, 0.0);
        }

        let ghost = &race.others[&1].test;
        let screen = |t: &TypingTest| -> Vec<(String, String)> {
            let words = t.words().iter().take(t.current() + 1);
            words.map(|w| (w.target.clone(), w.typed.clone())).collect()
        };
        assert_eq!(ghost.current(), racer.current());
        assert_eq!(screen(ghost), screen(&racer));
    }
}
