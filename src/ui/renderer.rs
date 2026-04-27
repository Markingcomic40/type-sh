use std::io::{Stdout, Write};

use crossterm::{
    cursor, queue,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};

use crate::core::game::GameState;
use crate::core::typing_test::{TypingTest, WordState};
use crate::ui::test_view::{TestView, WordLayout};
use crate::ui::theme::Theme;

pub struct Renderer {
    pub theme: Theme,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, stdout: &mut Stdout, state: &GameState) -> std::io::Result<()> {
        // let (w, h) = terminal::size().unwrap_or((80, 24));

        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
        )?;

        match state {
            GameState::Running { test, view } => self.render_test(stdout, test, view)?,
            _ => panic!("AAA"),
        }

        stdout.flush()
    }

    fn render_test(
        &self,
        stdout: &mut Stdout,
        test: &TypingTest,
        view: &TestView,
    ) -> std::io::Result<()> {
        queue!(stdout, cursor::Show)?;

        let h_offest = view.viewport().h_offset;

        let mut cursor_pos: Option<(u16, u16)> = None;

        for &WordLayout {
            word_index,
            line_number,
            x_position,
        } in view.layouts().iter()
        {
            if line_number >= view.viewport().visible_lines {
                break;
            }

            let word_x = h_offest + x_position as u16;
            let word_y = 4 + line_number as u16;

            queue!(stdout, cursor::MoveTo(word_x, word_y))?;

            if word_index == test.current_word_index() {
                let typed_len = test.current_input().len() as u16;
                cursor_pos = Some((word_x + typed_len, word_y))
            }

            self.render_word(stdout, test, word_index)?;
        }

        if let Some((x, y)) = cursor_pos {
            queue!(stdout, cursor::MoveTo(x, y))?;
        }

        Ok(())
    }

    fn render_word(
        &self,
        stdout: &mut Stdout,
        test: &TypingTest,
        word_index: usize,
    ) -> std::io::Result<()> {
        let word = &test.words()[word_index];
        let curr_index = test.current_word_index();

        match word_index.cmp(&curr_index) {
            std::cmp::Ordering::Less => self.render_completed_word(stdout, word)?,
            std::cmp::Ordering::Equal => {
                self.render_active_word(stdout, word, test.current_input())?
            }
            std::cmp::Ordering::Greater => self.render_upcoming_word(stdout, word)?,
        }

        Ok(())
    }

    fn render_completed_word(&self, stdout: &mut Stdout, word: &WordState) -> std::io::Result<()> {
        let typed: &str = word.typed.as_deref().unwrap_or("");
        self.render_char_diff(stdout, &word.target, typed, self.theme.incorrect)
    }

    fn render_active_word(
        &self,
        stdout: &mut Stdout,
        word: &WordState,
        input: &str,
    ) -> std::io::Result<()> {
        self.render_char_diff(stdout, &word.target, input, self.theme.secondary)
    }
    fn render_upcoming_word(&self, stdout: &mut Stdout, word: &WordState) -> std::io::Result<()> {
        queue!(
            stdout,
            SetForegroundColor(self.theme.secondary),
            Print(&word.target),
            ResetColor,
        )
    }

    fn render_char_diff(
        &self,
        stdout: &mut Stdout,
        target: &str,
        typed: &str,
        untyped_color: crossterm::style::Color,
    ) -> std::io::Result<()> {
        let overlap = std::cmp::min(target.len(), typed.len());

        for (tc, tt) in typed
            .chars()
            .take(overlap)
            .zip(target.chars().take(overlap))
        {
            let color = if tc == tt {
                self.theme.correct
            } else {
                self.theme.incorrect
            };
            queue!(stdout, SetForegroundColor(color), Print(tc))?;
        }

        // Typed was shorter
        for tt in target.chars().skip(overlap) {
            queue!(stdout, SetForegroundColor(untyped_color), Print(tt))?;
        }

        // Typed was longer
        for tc in typed.chars().skip(overlap) {
            queue!(
                stdout,
                SetForegroundColor(self.theme.incorrect_subtle),
                Print(tc)
            )?;
        }

        queue!(stdout, ResetColor)
    }
}
