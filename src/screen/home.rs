use crossterm::event::{KeyCode, KeyEvent};

use crate::screen::settings::SettingsMenu;
use crate::screen::{self, Next, Screen};
use crate::settings::Settings;
use crate::ui::frame::{Frame, Span, Style};
use crate::ui::rect::Rect;
use crate::ui::theme::Theme;

const LOGO: [&str; 2] = ["▀█▀ █▄█ █▀█ █▀▀   █▀▀ █▄▄", " █   █  █▀▀ ██▄ ▄ ▄▄█ █ █"];

/// Where ".sh" starts in the logo, so it can be coloured apart from "type".
const LOGO_SUFFIX_COL: usize = 16;

const TAGLINE: &str = "type like a true 10x engineer";

#[derive(Clone, Copy)]
enum Item {
    Play,
    Settings,
    Quit,
}

const ITEMS: [Item; 3] = [Item::Play, Item::Settings, Item::Quit];

impl Item {
    fn label(self) -> &'static str {
        match self {
            Item::Play => "play",
            Item::Settings => "settings",
            Item::Quit => "quit",
        }
    }
}

#[derive(Default)]
pub struct Home {
    selected: usize,
    error: Option<String>,
}

impl Home {
    pub fn with_error(error: String) -> Self {
        Self {
            error: Some(error),
            ..Self::default()
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, settings: &Settings) -> Next {
        self.error = None;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + ITEMS.len() - 1) % ITEMS.len();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % ITEMS.len();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                return match ITEMS[self.selected] {
                    Item::Play => Next::To(screen::play(settings)),
                    Item::Settings => Next::To(Screen::Settings(SettingsMenu::default())),
                    Item::Quit => Next::Quit,
                };
            }
            KeyCode::Esc | KeyCode::Char('q') => return Next::Quit,
            _ => {}
        }

        Next::Stay
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme) {
        const HEIGHT: u16 = 2 + 1 + 1 + 3 + 5 + 1 + 1;
        let area = f.area();
        let block = area.centered(area.width, HEIGHT);

        draw_logo(f, block, theme);
        f.print_centered(
            block,
            block.y + 3,
            &[Span::new(TAGLINE, Style::fg(theme.dim))],
        );

        let longest = ITEMS
            .iter()
            .map(|item| item.label().len())
            .max()
            .unwrap_or(0);
        let x = block.x + (block.width - longest as u16 - 2) / 2;
        for (i, item) in ITEMS.iter().enumerate() {
            let y = block.y + 7 + 2 * i as u16;
            if i == self.selected {
                f.put(x, y, '›', Style::fg(theme.accent));
                f.print(x + 2, y, item.label(), Style::fg(theme.accent).bold());
            } else {
                f.print(x + 2, y, item.label(), Style::fg(theme.dim));
            }
        }

        if let Some(error) = &self.error {
            let span = Span::new(error.as_str(), Style::fg(theme.error));
            f.print_centered(block, block.bottom() - 1, &[span]);
        }

        screen::draw_hints(
            f,
            theme,
            &[("↑↓", "select"), ("enter", "confirm"), ("esc", "quit")],
        );
    }
}

fn draw_logo(f: &mut Frame, area: Rect, theme: &Theme) {
    let width = LOGO[0].chars().count() as u16;
    let x = area.x + area.width.saturating_sub(width) / 2;

    for (row, line) in LOGO.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let color = if col < LOGO_SUFFIX_COL {
                theme.text
            } else {
                theme.accent
            };
            f.put(x + col as u16, area.y + row as u16, ch, Style::fg(color));
        }
    }
}
