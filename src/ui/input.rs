use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::frame::{Frame, Style};
use crate::ui::theme::Theme;

pub enum Input {
    Editing,
    Submit(String),
    Cancel,
}

/// A single line text field.
pub struct TextInput {
    value: String,
    digits_only: bool,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            digits_only: false,
        }
    }

    pub fn digits_only(self) -> Self {
        Self {
            digits_only: true,
            ..self
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Input {
        match key.code {
            KeyCode::Enter => return Input::Submit(std::mem::take(&mut self.value)),
            KeyCode::Esc => return Input::Cancel,
            KeyCode::Backspace => {
                self.value.pop();
            }
            KeyCode::Char(c) if !self.digits_only || c.is_ascii_digit() => self.value.push(c),
            _ => {}
        }
        Input::Editing
    }

    /// Draws the value, or `placeholder` while empty, and parks the caret after
    /// it. A value too long for the line shows its end, since that's where the
    /// typing happens.
    pub fn draw(&self, f: &mut Frame, x: u16, y: u16, placeholder: &str, theme: &Theme) {
        if self.value.is_empty() {
            f.print(x, y, placeholder, Style::fg(theme.dim));
            f.set_cursor(x, y);
            return;
        }

        // Leave a column free for the caret.
        let room = usize::from(f.area().width.saturating_sub(x + 1));
        let len = self.value.chars().count();
        let (x, shown) = if len <= room {
            (x, self.value.as_str())
        } else {
            // One column goes to the ellipsis.
            let skip = len - room.saturating_sub(1);
            let start = self.value.char_indices().nth(skip).map_or(0, |(i, _)| i);
            (
                f.print(x, y, "...", Style::fg(theme.dim)),
                &self.value[start..],
            )
        };

        let end = f.print(x, y, shown, Style::fg(theme.text));
        f.set_cursor(end, y);
    }
}
