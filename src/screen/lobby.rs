use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};

use crate::core::config::{Limit, Rules};
use crate::net::client::Event;
use crate::net::protocol::{Id, Phase, Player, ToClient, ToServer};
use crate::net::session::Session;
use crate::screen::multiplayer::MultiplayerMenu;
use crate::screen::race::Race;
use crate::screen::{self, Next, Screen};
use crate::settings::display_name;
use crate::ui::frame::{Frame, Style};
use crate::ui::theme::Theme;

const LABEL_WIDTH: u16 = 8;
const WIDTH: u16 = 2 + LABEL_WIDTH + 38;
const MAX_PLAYERS: u16 = 4;

pub struct Lobby {
    session: Session,
    /// Where others join (hosting) or where we joined
    address: String,
    me: Option<Id>,
    players: Vec<Player>,
    rules: Option<Rules>,
    phase: Phase,
    /// Lives from `Start` until everyone's back in the lobby
    race: Option<Race>,
    /// When results run out and everyone's sent back
    results_until: Option<Instant>,
    /// Why the host turned us away, shown once it hangs up
    rejected: Option<String>,
}

impl Lobby {
    pub fn new(session: Session, address: String) -> Self {
        Self {
            session,
            address,
            me: None,
            players: Vec::new(),
            rules: None,
            phase: Phase::Lobby,
            race: None,
            results_until: None,
            rejected: None,
        }
    }

    pub fn tick(&mut self) -> Next {
        let now = Instant::now();
        for event in self.session.poll(now) {
            match event {
                Event::Message(msg) => self.receive(msg, now),
                Event::Closed => {
                    let reason = self.rejected.take().unwrap_or_else(|| {
                        if self.me.is_some() {
                            "the host closed the room".to_owned()
                        } else {
                            "no room answered there".to_owned()
                        }
                    });
                    return Next::To(Screen::Multiplayer(MultiplayerMenu::with_error(reason)));
                }
            }
        }

        for msg in self.race.as_mut().map(Race::tick).unwrap_or_default() {
            self.session.send(&msg);
        }
        Next::Stay
    }

    fn receive(&mut self, msg: ToClient, now: Instant) {
        match msg {
            ToClient::Welcome { id } => self.me = Some(id),
            ToClient::Rejected { reason } => self.rejected = Some(reason),
            ToClient::Room {
                players,
                rules,
                phase,
            } => {
                // Counted from when it arrived, since the host's clock isn't ours
                self.results_until = match phase {
                    Phase::Results { back_in_ms } => Some(now + Duration::from_millis(back_in_ms)),
                    _ => None,
                };
                if phase == Phase::Lobby {
                    self.race = None;
                }
                self.players = players;
                self.rules = Some(rules);
                self.phase = phase;
            }
            ToClient::Start {
                rules,
                words,
                seed,
                countdown_ms,
            } if !words.is_empty() => {
                let go_at = now + Duration::from_millis(countdown_ms);
                let others = self
                    .players
                    .iter()
                    .map(|p| p.id)
                    .filter(|&id| Some(id) != self.me);
                self.race = Some(Race::new(rules.with_seed(seed), words, go_at, others));
            }
            ToClient::Typed { id, keys, wpm } if Some(id) != self.me => {
                if let Some(race) = &mut self.race {
                    race.typed(id, &keys, wpm);
                }
            }
            ToClient::Finished { id, report } => {
                if let Some(race) = &mut self.race {
                    race.finished(id, report);
                }
            }
            ToClient::Start { .. } | ToClient::Typed { .. } | ToClient::Left { .. } => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Next {
        let esc = key.code == KeyCode::Esc;
        match self.phase {
            Phase::Lobby if esc => {
                return Next::To(Screen::Multiplayer(MultiplayerMenu::default()));
            }
            Phase::Racing | Phase::Results { .. } if esc => self.session.send(&ToServer::Cancel),
            Phase::Racing => {
                if let Some(race) = &mut self.race {
                    let (players, me) = (&self.players, self.me);
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') if race.is_done() => {
                            race.watch_next(players, me, -1);
                        }
                        KeyCode::Down | KeyCode::Char('j') if race.is_done() => {
                            race.watch_next(players, me, 1);
                        }
                        _ => race.handle_key(key, Instant::now()),
                    }
                }
            }
            Phase::Lobby | Phase::Results { .. } => {
                let ready = !self.my_player().is_some_and(|p| p.ready);
                self.session.send(&ToServer::Ready { ready });
            }
        }
        Next::Stay
    }

    fn my_player(&self) -> Option<&Player> {
        self.players.iter().find(|p| Some(p.id) == self.me)
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme) {
        match (&self.phase, &self.race) {
            (Phase::Racing, Some(race)) if race.has_view(&self.players, self.me) => {
                race.draw(f, theme, &self.players, self.me);
                screen::draw_hints(f, theme, &self.hints());
            }
            (Phase::Racing | Phase::Results { .. }, Some(race)) => {
                self.draw_results(f, theme, race);
            }
            _ => self.draw_room(f, theme),
        }
    }

    fn draw_room(&self, f: &mut Frame, theme: &Theme) {
        // title, gap, address, rules, gap, a row per player, gap, status
        let height = 2 + 2 + 1 + MAX_PLAYERS + 1 + 1;
        let block = f.area().centered(WIDTH, height);
        let (x, value_x) = (block.x + 2, block.x + 2 + LABEL_WIDTH);
        let label = Style::fg(theme.dim);

        let title = if self.session.is_host() {
            "your room"
        } else {
            "room"
        };
        f.print(x, block.y, title, Style::fg(theme.text).bold());

        let (address_label, address_color) = if self.session.is_host() {
            ("share", theme.accent)
        } else {
            ("joined", theme.text)
        };
        f.print(x, block.y + 2, address_label, label);
        f.print(
            value_x,
            block.y + 2,
            &self.address,
            Style::fg(address_color),
        );

        f.print(x, block.y + 3, "rules", label);
        if let Some(rules) = &self.rules {
            f.print(
                value_x,
                block.y + 3,
                &describe(rules),
                Style::fg(theme.text),
            );
        }

        for (i, player) in self.players.iter().enumerate() {
            self.draw_player(f, theme, x, block.y + 5 + i as u16, player);
        }

        f.print(x, block.bottom() - 1, &self.status(), label);
        screen::draw_hints(f, theme, &self.hints());
    }

    fn draw_player(&self, f: &mut Frame, theme: &Theme, x: u16, y: u16, player: &Player) {
        let color = theme.player(player.color);
        let me = Some(player.id) == self.me;

        f.put(x, y, '●', Style::fg(color));
        let name = Style::fg(color);
        let end = f.print(x + 2, y, &player.name, if me { name.bold() } else { name });
        if me {
            f.print(end + 1, y, "(you)", Style::fg(theme.dim));
        }

        if !matches!(self.phase, Phase::Racing) {
            let (state, color) = if player.ready {
                ("ready", theme.accent)
            } else {
                ("not ready", theme.dim)
            };
            let right = x + WIDTH - 2;
            f.print(right - state.len() as u16, y, state, Style::fg(color));
        }
    }

    /// Everyone's result, best first, with who's ready for a rematch
    fn draw_results(&self, f: &mut Frame, theme: &Theme, race: &Race) {
        let height = 2 + MAX_PLAYERS + 1 + 1;
        let block = f.area().centered(WIDTH, height);
        let x = block.x + 2;
        let right = x + WIDTH - 2;

        f.print(x, block.y, "results", Style::fg(theme.text).bold());

        let mut rows: Vec<_> = self
            .players
            .iter()
            .map(|p| (p, race.report(p.id)))
            .collect();
        rows.sort_by(|(_, a), (_, b)| match (a, b) {
            (Some(a), Some(b)) => b.wpm.total_cmp(&a.wpm),
            (a, b) => b.is_some().cmp(&a.is_some()),
        });

        for (i, (player, report)) in rows.iter().enumerate() {
            let y = block.y + 2 + i as u16;
            let color = theme.player(player.color);
            let name = Style::fg(color);
            let name = if Some(player.id) == self.me {
                name.bold()
            } else {
                name
            };

            f.put(x + 3, y, '●', Style::fg(color));
            f.print(x + 5, y, &player.name, name);
            match report {
                Some(report) => {
                    f.print(x, y, &(i + 1).to_string(), Style::fg(theme.dim));
                    let wpm = format!("{:.0} wpm", report.wpm);
                    f.print(x + 29 - wpm.len() as u16, y, &wpm, Style::fg(theme.accent));
                    let acc = format!("{:.0}%", report.accuracy);
                    f.print(x + 35 - acc.len() as u16, y, &acc, Style::fg(theme.text));
                }
                None => {
                    f.print(x + 22, y, "racing...", Style::fg(theme.dim));
                }
            }

            if matches!(self.phase, Phase::Results { .. }) {
                let (state, color) = if player.ready {
                    ("ready", theme.accent)
                } else {
                    ("not ready", theme.dim)
                };
                f.print(right - state.len() as u16, y, state, Style::fg(color));
            }
        }

        let status = match self.phase {
            Phase::Racing => {
                let left = rows.iter().filter(|(_, r)| r.is_none()).count();
                format!("waiting for {left} more to finish")
            }
            _ => self.status(),
        };
        f.print(x, block.bottom() - 1, &status, Style::fg(theme.dim));
        screen::draw_hints(f, theme, &self.hints());
    }

    fn status(&self) -> String {
        let ready = self.my_player().is_some_and(|p| p.ready);
        match self.phase {
            _ if self.me.is_none() => "connecting...".to_owned(),
            Phase::Lobby if self.players.len() < 2 => "waiting for someone to join".to_owned(),
            Phase::Lobby | Phase::Results { .. } if ready => {
                "waiting for everyone to be ready".to_owned()
            }
            Phase::Lobby => "press any key when you're ready".to_owned(),
            Phase::Racing => "a race is on".to_owned(),
            Phase::Results { .. } => {
                let left = self.results_until.map_or(0, |at| {
                    at.saturating_duration_since(Instant::now()).as_secs()
                });
                format!("back to the lobby in {left}s")
            }
        }
    }

    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        let ready = self.my_player().is_some_and(|p| p.ready);
        let toggle = ("any key", if ready { "unready" } else { "ready" });
        match self.phase {
            Phase::Lobby => vec![toggle, ("esc", "leave")],
            Phase::Racing if self.race.as_ref().is_some_and(Race::is_done) => {
                vec![("↑↓", "watch"), ("esc", "back to lobby for everyone")]
            }
            Phase::Racing => vec![("esc", "back to lobby for everyone")],
            Phase::Results { .. } => vec![toggle, ("esc", "back to lobby for everyone")],
        }
    }
}

fn describe(rules: &Rules) -> String {
    let limit = match rules.limit {
        Limit::Time(secs) => format!("time {secs}"),
        Limit::Words(n) => format!("words {n}"),
        Limit::None => "zen".to_owned(),
    };
    let freedom = if rules.freedom {
        "freedom on"
    } else {
        "freedom off"
    };
    format!("{limit} · {} · {freedom}", display_name(&rules.wordlist))
}
