use crossterm::event::{KeyCode, KeyEvent};

use crate::error::Result;
use crate::screen::home::Home;
use crate::screen::{self, Next, Screen};
use crate::settings::{display_name, Settings};
use crate::ui::frame::{text_width, Frame, Style};
use crate::ui::input::{Input, TextInput};
use crate::ui::theme::Theme;

const LABEL_WIDTH: u16 = 12;
const OPTION_GAP: u16 = 3;

const CUSTOM: &str = "custom...";

#[derive(Clone, Copy)]
enum Row {
    Theme,
    Wordlist,
    Freedom,
}

const ROWS: [Row; 3] = [Row::Theme, Row::Wordlist, Row::Freedom];

impl Row {
    fn label(self) -> &'static str {
        match self {
            Row::Theme => "theme",
            Row::Wordlist => "wordlist",
            Row::Freedom => "freedom",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Row::Theme => "pick custom... to load your own theme from a json file",
            Row::Wordlist => "pick custom... to load your own words from a file",
            Row::Freedom => "let backspace go back into words you've already finished",
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            Row::Theme => "path to a theme .json",
            Row::Wordlist | Row::Freedom => "path to a file of words",
        }
    }

    /// Whether the row ends in a slot for loading a file
    fn takes_path(self) -> bool {
        matches!(self, Row::Theme | Row::Wordlist)
    }

    /// The row's choices, and which of them is curr
    fn options(self, s: &Settings) -> (Vec<&str>, usize) {
        let (options, current) = match self {
            Row::Theme => (s.themes(), s.theme.as_str()),
            Row::Wordlist => (s.wordlists(), s.wordlist.as_str()),
            Row::Freedom => (vec!["off", "on"], if s.freedom { "on" } else { "off" }),
        };
        let selected = options.iter().position(|&o| o == current).unwrap_or(0);
        (options, selected)
    }

    fn select(self, s: &mut Settings, option: String) {
        match self {
            Row::Theme => s.theme = option,
            Row::Wordlist => s.wordlist = option,
            Row::Freedom => s.freedom = option == "on",
        }
    }

    fn set_custom(self, s: &mut Settings, path: &str) -> Result<()> {
        match self {
            Row::Theme => s.set_custom_theme(path),
            Row::Wordlist => s.set_custom_wordlist(path),
            Row::Freedom => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct SettingsMenu {
    row: usize,
    /// Whether the focused rows custom slot is highlighted rather than its value
    on_custom: bool,
    /// Set while typing in a path for the focused row
    editing: Option<TextInput>,
    error: Option<String>,
}

impl SettingsMenu {
    pub fn handle_key(&mut self, key: KeyEvent, settings: &mut Settings) -> Next {
        let row = ROWS[self.row];

        if let Some(input) = &mut self.editing {
            match input.handle_key(key) {
                Input::Editing => {}
                Input::Cancel => self.editing = None,
                Input::Submit(path) => {
                    self.editing = None;
                    match row.set_custom(settings, &path) {
                        Ok(()) => self.on_custom = false,
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
            }
            return Next::Stay;
        }

        self.error = None;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.focus(self.row + ROWS.len() - 1),
            KeyCode::Down | KeyCode::Char('j') => self.focus(self.row + 1),
            KeyCode::Left | KeyCode::Char('h') => self.step(settings, -1),
            KeyCode::Right | KeyCode::Char('l') => self.step(settings, 1),
            KeyCode::Enter | KeyCode::Char(' ' | 'e') => {
                if row.takes_path() {
                    self.on_custom = true;
                    self.editing = Some(TextInput::new());
                } else {
                    self.step(settings, 1);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => return Next::To(Screen::Home(Home::default())),
            _ => {}
        }

        Next::Stay
    }

    fn focus(&mut self, row: usize) {
        self.row = row % ROWS.len();
        self.on_custom = false;
    }

    fn step(&mut self, settings: &mut Settings, delta: isize) {
        let row = ROWS[self.row];
        let (options, selected) = row.options(settings);
        let slots = options.len() + usize::from(row.takes_path());
        let at = if self.on_custom {
            options.len()
        } else {
            selected
        };
        let next = (at as isize + delta).rem_euclid(slots as isize) as usize;

        self.on_custom = next == options.len();
        if let Some(option) = options.get(next).map(|o| o.to_string()) {
            row.select(settings, option);
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme, settings: &Settings) {
        let area = f.area();
        let rows: Vec<_> = ROWS.iter().map(|row| row.options(settings)).collect();
        let widest = ROWS
            .iter()
            .zip(&rows)
            .map(|(row, (options, _))| {
                let names = options.iter().map(|o| display_name(o));
                let custom = row.takes_path().then_some(CUSTOM);
                let widths: Vec<u16> = names.chain(custom).map(text_width).collect();
                widths.iter().sum::<u16>() + OPTION_GAP * (widths.len() as u16 - 1)
            })
            .chain(
                ROWS.iter()
                    .map(|row| text_width(row.help()).saturating_sub(LABEL_WIDTH)),
            )
            .max()
            .unwrap_or(0);

        // title, gap, rows spaced by a row, gap, help or error
        let height = 2 + 2 * ROWS.len() as u16 + 1;
        let block = area.centered(2 + LABEL_WIDTH + widest, height);
        let (x, text_x) = (block.x, block.x + 2 + LABEL_WIDTH);

        f.print(x + 2, block.y, "settings", Style::fg(theme.text).bold());

        for (i, (row, (options, selected))) in ROWS.iter().zip(&rows).enumerate() {
            let y = block.y + 2 + 2 * i as u16;
            let focused = i == self.row;

            if focused {
                f.put(x, y, '›', Style::fg(theme.accent));
            }
            let label_color = if focused { theme.text } else { theme.dim };
            f.print(x + 2, y, row.label(), Style::fg(label_color));

            if let Some(input) = self.editing.as_ref().filter(|_| focused) {
                input.draw(f, text_x, y, row.placeholder(), theme);
                continue;
            }

            let mut ox = text_x;
            for (j, option) in options.iter().enumerate() {
                let color = if j == *selected {
                    theme.accent
                } else {
                    theme.dim
                };
                ox = f.print(ox, y, display_name(option), Style::fg(color)) + OPTION_GAP;
            }
            if row.takes_path() {
                let highlighted = focused && self.on_custom;
                let color = if highlighted { theme.text } else { theme.dim };
                f.print(ox, y, CUSTOM, Style::fg(color).underline(highlighted));
            }
        }

        let (message, color) = match &self.error {
            Some(error) => (error.as_str(), theme.error),
            None => (ROWS[self.row].help(), theme.dim),
        };
        f.print(x + 2, block.bottom() - 1, message, Style::fg(color));

        let hints: &[_] = if self.editing.is_some() {
            &[("enter", "load"), ("esc", "cancel")]
        } else if self.on_custom {
            &[
                ("↑↓", "move"),
                ("←→", "change"),
                ("enter", "load file"),
                ("esc", "back"),
            ]
        } else {
            &[("↑↓", "move"), ("←→", "change"), ("esc", "back")]
        };
        screen::draw_hints(f, theme, hints);
    }
}
