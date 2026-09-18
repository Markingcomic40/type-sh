//! The app is a state machine of screens:
//!
//! - home goes to play or settings, or quits
//! - play goes to results when the test ends, or home on esc
//! - results goes to play again on tab, or home on esc
//! - settings goes home on esc
//!
//! Each screen handles its own keys and draws itself; switching screens is
//! just returning the next one.

pub mod home;
pub mod play;
pub mod results;
pub mod settings;

use crossterm::event::KeyEvent;

use crate::settings::Settings;
use crate::ui::frame::{spans_width, Frame, Span, Style};
use crate::ui::theme::Theme;

use home::Home;
use play::Play;
use results::Results;
use settings::SettingsMenu;

/// Easier than somehow getting it to works maller lmfao
const MIN_WIDTH: u16 = 50;
const MIN_HEIGHT: u16 = 16;

pub enum Screen {
    Home(Home),
    Play(Box<Play>),
    Settings(SettingsMenu),
    Results(Box<Results>),
}

/// What a screen wants to happen after handling input
pub enum Next {
    Stay,
    To(Screen),
    Quit,
}

impl Screen {
    pub fn handle_key(&mut self, key: KeyEvent, settings: &mut Settings) -> Next {
        match self {
            Screen::Home(home) => home.handle_key(key, settings),
            Screen::Play(play) => play.handle_key(key, settings),
            Screen::Settings(menu) => menu.handle_key(key, settings),
            Screen::Results(results) => results.handle_key(key, settings),
        }
    }

    /// Called every frame, for screens that change without input
    pub fn tick(&mut self) -> Next {
        match self {
            Screen::Play(play) => play.tick(),
            _ => Next::Stay,
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme, settings: &Settings) {
        let area = f.area();
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            draw_too_small(f, theme);
            return;
        }

        match self {
            Screen::Home(home) => home.draw(f, theme),
            Screen::Play(play) => play.draw(f, theme, settings),
            Screen::Settings(menu) => menu.draw(f, theme, settings),
            Screen::Results(results) => results.draw(f, theme),
        }
    }
}

fn draw_too_small(f: &mut Frame, theme: &Theme) {
    let area = f.area();
    let side = |name: &'static str, have: u16, need: u16| {
        let color = if have < need {
            theme.error
        } else {
            theme.correct
        };
        vec![
            Span::new(format!("{name:<8}"), Style::fg(theme.dim)),
            Span::new(have.to_string(), Style::fg(color).bold()),
            Span::new(format!(" / {need}"), Style::fg(theme.dim)),
        ]
    };

    let full = vec![
        vec![Span::new(
            "terminal too small",
            Style::fg(theme.text).bold(),
        )],
        vec![],
        side("width", area.width, MIN_WIDTH),
        side("height", area.height, MIN_HEIGHT),
    ];
    let terse = vec![vec![Span::new(
        format!("need {MIN_WIDTH}x{MIN_HEIGHT}"),
        Style::fg(theme.error),
    )]];

    let fits = |lines: &[Vec<Span>]| {
        lines.len() as u16 <= area.height
            && lines.iter().all(|line| spans_width(line) <= area.width)
    };
    // Past the terse message there's nothing smaller worth saying; let it clip.
    let lines = if fits(&full) { full } else { terse };

    let width = lines
        .iter()
        .map(|line| spans_width(line))
        .max()
        .unwrap_or(0);
    let block = area.centered(width, lines.len() as u16);
    for (y, line) in (block.y..).zip(&lines) {
        f.print_spans(block.x, y, line);
    }
}

pub fn play(settings: &Settings) -> Screen {
    match Play::new(settings) {
        Ok(play) => Screen::Play(Box::new(play)),
        Err(e) => Screen::Home(Home::with_error(e.to_string())),
    }
}

pub fn draw_hints(f: &mut Frame, theme: &Theme, hints: &[(&str, &str)]) {
    let mut spans = Vec::with_capacity(hints.len() * 3);
    for (i, &(key, action)) in hints.iter().enumerate() {
        let gap = if i == 0 { "" } else { "    " };
        spans.push(Span::new(gap, Style::default()));
        spans.push(Span::new(key, Style::fg(theme.text)));
        spans.push(Span::new(format!(" {action}"), Style::fg(theme.dim)));
    }

    let area = f.area();
    f.print_centered(area, area.bottom().saturating_sub(2), &spans);
}
