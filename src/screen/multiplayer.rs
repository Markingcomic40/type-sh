use crossterm::event::{KeyCode, KeyEvent};

use crate::core::config::{Limit, Rules};
use crate::net::session::{self, Session};
use crate::screen::home::Home;
use crate::screen::lobby::Lobby;
use crate::screen::{self, Next, Screen};
use crate::settings::{clean_name, Settings};
use crate::ui::frame::{Frame, Style};
use crate::ui::input::{Input, TextInput};
use crate::ui::theme::Theme;

const LABEL_WIDTH: u16 = 8;
const WIDTH: u16 = 2 + LABEL_WIDTH + 38;

#[derive(Clone, Copy)]
enum Row {
    Name,
    Host,
    Join,
}

const ROWS: [Row; 3] = [Row::Name, Row::Host, Row::Join];

impl Row {
    fn label(self) -> &'static str {
        match self {
            Row::Name => "name",
            Row::Host => "host",
            Row::Join => "join",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Row::Name => "what everyone else sees you as",
            Row::Host => "start a room on this network",
            Row::Join => "the address shown in the host's room",
        }
    }
}

#[derive(Default)]
pub struct MultiplayerMenu {
    row: usize,
    editing: Option<TextInput>,
    error: Option<String>,
}

impl MultiplayerMenu {
    pub fn with_error(error: String) -> Self {
        Self {
            error: Some(error),
            ..Self::default()
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, settings: &mut Settings) -> Next {
        let row = ROWS[self.row];

        if let Some(input) = &mut self.editing {
            return match input.handle_key(key) {
                Input::Editing => Next::Stay,
                Input::Cancel => {
                    self.editing = None;
                    Next::Stay
                }
                Input::Submit(value) => {
                    self.editing = None;
                    self.submit(row, &value, settings)
                }
            };
        }

        self.error = None;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.row = (self.row + ROWS.len() - 1) % ROWS.len(),
            KeyCode::Down | KeyCode::Char('j') => self.row = (self.row + 1) % ROWS.len(),
            KeyCode::Enter | KeyCode::Char(' ') => match row {
                Row::Name => self.editing = Some(TextInput::with_value(&settings.name)),
                Row::Host => return self.host(settings),
                Row::Join => self.editing = Some(TextInput::new()),
            },
            KeyCode::Esc | KeyCode::Char('q') => return Next::To(Screen::Home(Home::default())),
            _ => {}
        }
        Next::Stay
    }

    fn submit(&mut self, row: Row, value: &str, settings: &mut Settings) -> Next {
        match row {
            Row::Name => match clean_name(value) {
                Some(name) => settings.name = name,
                None => self.error = Some("a name needs at least one character".to_owned()),
            },
            Row::Join if !value.trim().is_empty() => {
                return match Session::join(value, &settings.name) {
                    Ok(session) => lobby(session, value.trim().to_owned()),
                    Err(e) => {
                        self.error = Some(format!("couldn't join: {e}"));
                        Next::Stay
                    }
                };
            }
            Row::Join | Row::Host => {}
        }
        Next::Stay
    }

    fn host(&mut self, settings: &Settings) -> Next {
        match Session::host(race_rules(settings), &settings.name) {
            Ok(session) => {
                let address =
                    session::lan_ip().map_or("no network found".to_owned(), |ip| ip.to_string());
                lobby(session, address)
            }
            Err(e) => {
                self.error = Some(format!("couldn't host: {e}"));
                Next::Stay
            }
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme, settings: &Settings) {
        // title, gap, rows spaced by a row, gap, help or error
        let height = 2 + 2 * ROWS.len() as u16 + 1;
        let block = f.area().centered(WIDTH, height);
        let (x, value_x) = (block.x, block.x + 2 + LABEL_WIDTH);

        f.print(x + 2, block.y, "multiplayer", Style::fg(theme.text).bold());

        for (i, row) in ROWS.iter().enumerate() {
            let y = block.y + 2 + 2 * i as u16;
            let focused = i == self.row;

            if focused {
                f.put(x, y, '›', Style::fg(theme.accent));
            }
            let label = if focused { theme.text } else { theme.dim };
            f.print(x + 2, y, row.label(), Style::fg(label));

            if let Some(input) = self.editing.as_ref().filter(|_| focused) {
                let placeholder = match row {
                    Row::Join => "e.g. 192.168.1.23",
                    _ => "your name",
                };
                input.draw(f, value_x, y, placeholder, theme);
                continue;
            }

            let (value, color) = match row {
                Row::Name => (settings.name.as_str(), theme.accent),
                Row::Host => ("start a room", theme.dim),
                Row::Join => ("type an address", theme.dim),
            };
            let color = if focused && !matches!(row, Row::Name) {
                theme.text
            } else {
                color
            };
            f.print(value_x, y, value, Style::fg(color));
        }

        let (message, color) = match &self.error {
            Some(error) => (error.as_str(), theme.error),
            None => (ROWS[self.row].help(), theme.dim),
        };
        f.print(x + 2, block.bottom() - 1, message, Style::fg(color));

        let hints: &[_] = if self.editing.is_some() {
            &[("enter", "confirm"), ("esc", "cancel")]
        } else {
            &[("↑↓", "move"), ("enter", "select"), ("esc", "back")]
        };
        screen::draw_hints(f, theme, hints);
    }
}

fn lobby(session: Session, address: String) -> Next {
    Next::To(Screen::Lobby(Box::new(Lobby::new(session, address))))
}

/// Zen never ends, so a race of it would have no winner
fn race_rules(settings: &Settings) -> Rules {
    let mut rules = settings.rules();
    if rules.limit == Limit::None {
        rules.limit = Limit::Words(settings.words);
    }
    rules
}
