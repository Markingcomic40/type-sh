use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::config::Limit;
use crate::core::typing_test::{Glyph, TypingTest, Word};
use crate::error::Result;
use crate::screen::home::Home;
use crate::screen::results::Results;
use crate::screen::{self, Next, Screen};
use crate::settings::{display_name, Mode, Settings, MAX_AMOUNT};
use crate::ui::frame::{spans_width, Frame, Span, Style};
use crate::ui::input::{Input, TextInput};
use crate::ui::rect::Rect;
use crate::ui::theme::Theme;

/// Lines u see ont he screen of words at most
const LINES: u16 = 3;

/// Widest the words get
const MAX_TEXT_WIDTH: u16 = 80;

#[derive(Clone, PartialEq, Eq)]
enum Item {
    Mode(Mode),
    Amount(u64),
    CustomAmount,
    Wordlist(String),
}

impl Item {
    fn is_selected(&self, s: &Settings) -> bool {
        match self {
            Item::Mode(mode) => s.mode == *mode,
            Item::Amount(n) => s.amount() == Some(*n),
            Item::CustomAmount => s.amount().is_some_and(|n| !s.mode.presets().contains(&n)),
            Item::Wordlist(name) => s.wordlist == *name,
        }
    }

    fn label(&self, s: &Settings) -> Cow<'_, str> {
        match self {
            Item::Mode(mode) => mode.name().into(),
            Item::Amount(n) => n.to_string().into(),
            Item::CustomAmount => match s.amount().filter(|_| self.is_selected(s)) {
                Some(n) => n.to_string().into(),
                None => "custom".into(),
            },
            Item::Wordlist(name) => display_name(name).into(),
        }
    }
}

/// The bar's items, grouped by what they configure.
fn bar_groups(s: &Settings) -> Vec<Vec<Item>> {
    let mut groups = vec![Mode::ALL.map(Item::Mode).to_vec()];
    if s.mode != Mode::Zen {
        let amounts = s.mode.presets().iter().map(|&n| Item::Amount(n));
        groups.push(amounts.chain([Item::CustomAmount]).collect());
    }
    groups.push(
        s.wordlists()
            .into_iter()
            .map(|name| Item::Wordlist(name.to_owned()))
            .collect(),
    );
    groups
}

fn bar_items(s: &Settings) -> Vec<Item> {
    bar_groups(s).into_iter().flatten().collect()
}

struct Bar {
    cursor: usize,
    /// Set while typing in custom 
    editing: Option<TextInput>,
}

pub struct Play {
    test: TypingTest,
    /// Some while the config bar has focus rather than the words
    bar: Option<Bar>,
    error: Option<String>,
}

impl Play {
    pub fn new(settings: &Settings) -> Result<Self> {
        Ok(Self {
            test: TypingTest::new(settings.test_config())?,
            bar: None,
            error: None,
        })
    }

    pub fn tick(&mut self) -> Next {
        if self.test.is_finished() {
            return Next::To(Screen::Results(Box::new(Results::new(&self.test))));
        }
        Next::Stay
    }

    pub fn handle_key(&mut self, key: KeyEvent, settings: &mut Settings) -> Next {
        self.error = None;
        if self.bar.is_some() {
            self.handle_bar_key(key, settings);
            return Next::Stay;
        }

        let started = self.test.has_started();
        match key.code {
            KeyCode::Tab => self.restart(settings),
            KeyCode::Esc if started => self.test.stop(),
            KeyCode::Esc => return Next::To(Screen::Home(Home::default())),
            KeyCode::Up if !started => self.open_bar(settings),
            KeyCode::Backspace => self.test.backspace(),
            KeyCode::Char(' ') => self.test.space(),
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.test.type_char(c)
            }
            _ => {}
        }

        Next::Stay
    }

    fn handle_bar_key(&mut self, key: KeyEvent, settings: &mut Settings) {
        let Some(bar) = &mut self.bar else {
            return;
        };

        if let Some(input) = &mut bar.editing {
            match input.handle_key(key) {
                Input::Editing => {}
                Input::Cancel => bar.editing = None,
                Input::Submit(value) => {
                    bar.editing = None;
                    match value.parse() {
                        Ok(n) if (1..=MAX_AMOUNT).contains(&n) => {
                            settings.set_amount(n);
                            self.restart(settings);
                        }
                        _ => self.error = Some(format!("pick a number from 1 to {MAX_AMOUNT}")),
                    }
                }
            }
            return;
        }

        let items = bar_items(settings);
        match key.code {
            KeyCode::Left => bar.cursor = bar.cursor.saturating_sub(1),
            KeyCode::Right => bar.cursor = (bar.cursor + 1).min(items.len() - 1),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let item = items[bar.cursor].clone();
                self.pick(item, settings);
            }
            KeyCode::Down | KeyCode::Esc => self.bar = None,

            _ => {
                self.bar = None;
                self.handle_key(key, settings);
            }
        }
    }

    fn open_bar(&mut self, settings: &Settings) {
        let cursor = bar_items(settings)
            .iter()
            .position(|item| item.is_selected(settings))
            .unwrap_or(0);
        self.bar = Some(Bar {
            cursor,
            editing: None,
        });
    }

    fn pick(&mut self, item: Item, settings: &mut Settings) {
        match item {
            Item::Mode(mode) => settings.mode = mode,
            Item::Amount(n) => settings.set_amount(n),
            Item::Wordlist(name) => settings.wordlist = name,
            Item::CustomAmount => {
                if let Some(bar) = &mut self.bar {
                    bar.editing = Some(TextInput::new().digits_only());
                }
                return;
            }
        }
        self.restart(settings);
    }

    fn restart(&mut self, settings: &Settings) {
        match TypingTest::new(settings.test_config()) {
            Ok(test) => self.test = test,
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme, settings: &Settings) {
        let area = f.area();
        let width = area.width.saturating_sub(8).min(MAX_TEXT_WIDTH);
        let words = Rect::new(
            area.x + (area.width - width) / 2,
            (area.height / 2).saturating_sub(LINES),
            width,
            LINES,
        );

        self.draw_counter(f, theme, words.x, words.y - 2);
        self.draw_words(f, theme, words);

        if let Some(error) = &self.error {
            let span = Span::new(error.as_str(), Style::fg(theme.error));
            f.print_centered(area, words.bottom() + 2, &[span]);
        }

        if self.test.has_started() {
            if self.test.config().limit == Limit::None {
                screen::draw_hints(f, theme, &[("esc", "finish")]);
            }
            return;
        }

        self.draw_bar(f, theme, settings, area.y + 2);

        let hints: &[_] = match &self.bar {
            Some(Bar {
                editing: Some(_), ..
            }) => &[("enter", "confirm"), ("esc", "cancel")],
            Some(_) => &[("←→", "move"), ("enter", "select"), ("↓", "back")],
            None => &[("tab", "restart"), ("↑", "options"), ("esc", "home")],
        };
        screen::draw_hints(f, theme, hints);
    }

    /// Time left, words done, or words typed etc.
    fn draw_counter(&self, f: &mut Frame, theme: &Theme, x: u16, y: u16) {
        let test = &self.test;
        let text = match test.config().limit {
            Limit::Time(secs) => secs.saturating_sub(test.elapsed().as_secs()).to_string(),
            Limit::Words(n) => format!("{}/{n}", test.current()),
            Limit::None => test.current().to_string(),
        };
        let color = if test.has_started() {
            theme.accent
        } else {
            theme.dim
        };
        f.print(x, y, &text, Style::fg(color));
    }

    fn draw_words(&self, f: &mut Frame, theme: &Theme, area: Rect) {
        let words = self.test.words();
        let current = self.test.current().min(words.len() - 1);
        let layout = wrap(words.iter().map(Word::width), usize::from(area.width));
        let first = layout[current].0.saturating_sub(1);

        for (i, (word, &(line, col))) in words.iter().zip(&layout).enumerate() {
            if line < first {
                continue;
            }
            if line >= first + usize::from(LINES) {
                break;
            }

            let x = area.x + col as u16;
            let y = area.y + (line - first) as u16;
            draw_word(f, theme, x, y, word, i < current);

            if i == current && self.bar.is_none() {
                f.set_cursor(x + word.typed.chars().count() as u16, y);
            }
        }
    }

    fn draw_bar(&self, f: &mut Frame, theme: &Theme, settings: &Settings, y: u16) {
        let groups = bar_groups(settings);
        let cursor = self.bar.as_ref().map(|bar| bar.cursor);
        let input = self.bar.as_ref().and_then(|bar| bar.editing.as_ref());

        let mut spans = Vec::new();
        let mut caret = None;
        let mut index = 0;
        for (g, group) in groups.iter().enumerate() {
            if g > 0 {
                spans.push(Span::new("   │   ", Style::fg(theme.faint)));
            }
            for (i, item) in group.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::new("  ", Style::default()));
                }

                let focused = cursor == Some(index);
                let color = match (item.is_selected(settings), focused) {
                    (true, _) => theme.accent,
                    (false, true) => theme.text,
                    (false, false) => theme.dim,
                };
                let style = Style::fg(color).underline(focused);

                match input.filter(|_| focused) {
                    Some(input) => {
                        spans.push(Span::new(input.value(), style));
                        caret = Some(spans.len());
                    }
                    None => spans.push(Span::new(item.label(settings), style)),
                }
                index += 1;
            }
        }

        let x = f.print_centered(f.area(), y, &spans);
        if let Some(end) = caret {
            f.set_cursor(x + spans_width(&spans[..end]), y);
        }
    }
}

fn draw_word(f: &mut Frame, theme: &Theme, x: u16, y: u16, word: &Word, finished: bool) {
    let underline = finished && !word.is_correct();

    for (i, glyph) in word.glyphs().enumerate() {
        let (ch, color) = match glyph {
            Glyph::Correct(c) => (c, theme.correct),
            Glyph::Incorrect(c) => (c, theme.error),
            Glyph::Extra(c) => (c, theme.error_extra),
            Glyph::Untyped(c) => (c, theme.dim),
        };
        f.put(x + i as u16, y, ch, Style::fg(color).underline(underline));
    }
}

/// Lays words out left to right with a space between, wrapping at width yield for each word line n column
fn wrap(widths: impl Iterator<Item = usize>, width: usize) -> Vec<(usize, usize)> {
    let (mut line, mut col) = (0, 0);

    widths
        .map(|w| {
            // A word too long for any line still gets one to itself.
            if col > 0 && col + w > width {
                line += 1;
                col = 0;
            }
            let pos = (line, col);
            col += w + 1;
            pos
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_breaks_before_words_that_would_overflow() {
        let layout = wrap([3, 3, 3, 10, 1].into_iter(), 8);

        assert_eq!(layout, [(0, 0), (0, 4), (1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn bar_hides_amounts_in_zen() {
        let mut s = Settings::default();
        assert_eq!(bar_groups(&s).len(), 3);

        s.mode = Mode::Zen;
        assert_eq!(bar_groups(&s).len(), 2);
    }

    #[test]
    fn custom_amount_shows_its_value_once_set() {
        let mut s = Settings::default();
        assert_eq!(Item::CustomAmount.label(&s), "custom");

        s.set_amount(45);
        assert!(Item::CustomAmount.is_selected(&s));
        assert_eq!(Item::CustomAmount.label(&s), "45");
    }
}
