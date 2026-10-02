use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent};

use crate::settings::{display_name, Mode, Settings, MAX_AMOUNT};
use crate::ui::frame::{spans_width, Frame, Span, Style};
use crate::ui::input::{Input, TextInput};
use crate::ui::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Solo,
    /// No zen, since it never ends, and freedom is a rule everyone shares
    Race,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Item {
    Mode(Mode),
    Amount(u64),
    CustomAmount,
    Wordlist(String),
    Freedom,
}

impl Item {
    fn is_selected(&self, s: &Settings) -> bool {
        match self {
            Item::Mode(mode) => s.mode == *mode,
            Item::Amount(n) => s.amount() == Some(*n),
            Item::CustomAmount => s.amount().is_some_and(|n| !s.mode.presets().contains(&n)),
            Item::Wordlist(name) => s.wordlist == *name,
            Item::Freedom => s.freedom,
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
            Item::Freedom => "freedom".into(),
        }
    }
}

/// The bar's items, grouped by what they configure.
fn groups(s: &Settings, kind: Kind) -> Vec<Vec<Item>> {
    let modes = Mode::ALL
        .into_iter()
        .filter(|&mode| kind == Kind::Solo || mode != Mode::Zen);
    let mut groups = vec![modes.map(Item::Mode).collect()];

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
    if kind == Kind::Race {
        groups.push(vec![Item::Freedom]);
    }
    groups
}

fn items(s: &Settings, kind: Kind) -> Vec<Item> {
    groups(s, kind).into_iter().flatten().collect()
}

/// What a key did to the bar
pub enum Outcome {
    Stay,
    /// The settings changed, so whatever they configure needs redoing
    Changed,
    Close,
    /// Not the bar's key: close it and let the screen have it
    Pass,
    Error(String),
}

/// The bar while it has focus
pub struct Bar {
    kind: Kind,
    cursor: usize,
    /// Set while typing in custom
    editing: Option<TextInput>,
}

impl Bar {
    /// Focused on whatever's selected now
    pub fn open(settings: &Settings, kind: Kind) -> Self {
        let cursor = items(settings, kind)
            .iter()
            .position(|item| item.is_selected(settings))
            .unwrap_or(0);
        Self {
            kind,
            cursor,
            editing: None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    pub fn handle_key(&mut self, key: KeyEvent, settings: &mut Settings) -> Outcome {
        if let Some(input) = &mut self.editing {
            return match input.handle_key(key) {
                Input::Editing => Outcome::Stay,
                Input::Cancel => {
                    self.editing = None;
                    Outcome::Stay
                }
                Input::Submit(value) => {
                    self.editing = None;
                    match value.parse() {
                        Ok(n) if (1..=MAX_AMOUNT).contains(&n) => {
                            settings.set_amount(n);
                            Outcome::Changed
                        }
                        _ => Outcome::Error(format!("pick a number from 1 to {MAX_AMOUNT}")),
                    }
                }
            };
        }

        let items = items(settings, self.kind);
        match key.code {
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(items.len() - 1),
            KeyCode::Enter | KeyCode::Char(' ') => {
                return match items[self.cursor].clone() {
                    Item::CustomAmount => {
                        self.editing = Some(TextInput::new().digits_only());
                        Outcome::Stay
                    }
                    item => {
                        pick(item, settings);
                        Outcome::Changed
                    }
                };
            }
            KeyCode::Down | KeyCode::Esc => return Outcome::Close,
            _ => return Outcome::Pass,
        }
        Outcome::Stay
    }
}

fn pick(item: Item, settings: &mut Settings) {
    match item {
        Item::Mode(mode) => settings.mode = mode,
        Item::Amount(n) => settings.set_amount(n),
        Item::Wordlist(name) => settings.wordlist = name,
        Item::Freedom => settings.freedom = !settings.freedom,
        Item::CustomAmount => {}
    }
}

/// Drawn whether or not it has focus; `bar` is the focused state if it does
pub fn draw(
    f: &mut Frame,
    theme: &Theme,
    settings: &Settings,
    kind: Kind,
    bar: Option<&Bar>,
    y: u16,
) {
    let groups = groups(settings, kind);
    let cursor = bar.map(|bar| bar.cursor);
    let input = bar.and_then(|bar| bar.editing.as_ref());

    let build = |separator: &'static str| {
        let mut spans = Vec::new();
        let mut caret = None;
        let mut index = 0;
        for (g, group) in groups.iter().enumerate() {
            if g > 0 {
                spans.push(Span::new(separator, Style::fg(theme.faint)));
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
        (spans, caret)
    };

    // Squeeze the gaps between groups before letting the end get cut off
    let width = f.area().width;
    let (spans, caret) = ["   │   ", "  │  ", " │ "]
        .into_iter()
        .map(build)
        .find(|(spans, _)| spans_width(spans) <= width)
        .unwrap_or_else(|| build(" │ "));

    let x = f.print_centered(f.area(), y, &spans);
    if let Some(end) = caret {
        f.set_cursor(x + spans_width(&spans[..end]), y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_hides_amounts_in_zen() {
        let mut s = Settings::default();
        assert_eq!(groups(&s, Kind::Solo).len(), 3);

        s.mode = Mode::Zen;
        assert_eq!(groups(&s, Kind::Solo).len(), 2);
    }

    #[test]
    fn custom_amount_shows_its_value_once_set() {
        let mut s = Settings::default();
        assert_eq!(Item::CustomAmount.label(&s), "custom");

        s.set_amount(45);
        assert!(Item::CustomAmount.is_selected(&s));
        assert_eq!(Item::CustomAmount.label(&s), "45");
    }

    #[test]
    fn races_drop_zen_and_add_freedom() {
        let s = Settings::default();
        let race = items(&s, Kind::Race);

        assert!(!race.contains(&Item::Mode(Mode::Zen)));
        assert_eq!(race.last(), Some(&Item::Freedom));
        assert!(!items(&s, Kind::Solo).contains(&Item::Freedom));
    }
}
