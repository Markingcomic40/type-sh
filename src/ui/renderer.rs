use std::io::{Stdout, Write};

use crossterm::{
    cursor, queue,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};

use crate::ui::theme::Theme;
use crate::{
    core::statistics::TestResults,
    ui::test_view::{TestView, WordLayout},
};
use crate::{
    core::typing_test::{TypingTest, WordState},
    ui::menu_view::MenuView,
};
use crate::{
    core::{game::GameState, menu::MenuState},
    ui::menu_view::compute_menu_layout,
};

const TEXT_START_ROW: u16 = 5;

pub struct Renderer {
    pub theme: Theme,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, stdout: &mut Stdout, state: &GameState) -> std::io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
        )?;

        match state {
            GameState::Menu(menu) => {
                let layout = compute_menu_layout(menu);
                self.render_menu(stdout, &layout)?
            }
            GameState::Running { test, view } => self.render_test(stdout, test, view)?,
            GameState::Results(results) => self.render_results(stdout, results)?,
        }

        stdout.flush()
    }

    fn render_menu(&self, stdout: &mut Stdout, layout: &MenuView) -> std::io::Result<()> {
        let (width, _) = terminal::size().unwrap_or((80, 24));

        let cx = width / 2;

        if layout.is_inputting {
            queue!(stdout, cursor::Show)?;
        } else {
            queue!(stdout, cursor::Hide)?;
        }

        let banner = "TypeSH";
        queue!(
            stdout,
            cursor::MoveTo(cx.saturating_sub(banner.len() as u16 / 2), 1),
            SetForegroundColor(self.theme.primary),
            Print(banner),
            ResetColor,
        )?;

        let hint = if layout.is_inputting {
            "enter confirm  |  esc cancel"
        } else {
            "enter start  |  e edit  |  tab navigate  |  <-> select  |  esc quit"
        };
        queue!(
            stdout,
            cursor::MoveTo(cx.saturating_sub(hint.len() as u16 / 2), 3),
            SetForegroundColor(self.theme.secondary),
            Print(hint),
            ResetColor,
        )?;

        let label_col_width = 10u16;
        let content_start = cx.saturating_sub(30);
        let mut row = 6u16;
        let mut input_cursor: Option<(u16, u16)> = None;

        for section in &layout.sections {
            if section.is_hidden {
                continue;
            }

            let label_color = if section.is_focused {
                self.theme.primary
            } else {
                self.theme.secondary
            };
            queue!(
                stdout,
                cursor::MoveTo(content_start, row),
                SetForegroundColor(label_color),
                Print(section.label),
                ResetColor,
            )?;

            if let Some(input) = &section.active_input {
                let input_x = content_start + label_col_width;

                if input.buffer.is_empty() {
                    queue!(
                        stdout,
                        cursor::MoveTo(input_x, row),
                        SetForegroundColor(self.theme.secondary),
                        Print(input.placeholder),
                        ResetColor,
                    )?;
                } else {
                    queue!(
                        stdout,
                        cursor::MoveTo(input_x, row),
                        SetForegroundColor(self.theme.primary),
                        Print(&input.buffer),
                        ResetColor,
                    )?;
                }
                input_cursor = Some((input_x + input.buffer.len() as u16, row));
            } else {
                let mut x = content_start + label_col_width;
                for (i, option) in section.options.iter().enumerate() {
                    let is_selected = i == section.selected_index;
                    let color = if is_selected {
                        self.theme.primary
                    } else {
                        self.theme.secondary
                    };

                    queue!(
                        stdout,
                        cursor::MoveTo(x, row),
                        SetForegroundColor(color),
                        Print(option),
                        ResetColor,
                    )?;

                    x += option.len() as u16 + 3;
                }
            }

            row += 2;
        }

        if let Some(err) = layout.error_message {
            queue!(
                stdout,
                cursor::MoveTo(content_start + label_col_width, row),
                SetForegroundColor(self.theme.incorrect),
                Print(err),
                ResetColor,
            )?;
        }

        if let Some((x, y)) = input_cursor {
            queue!(stdout, cursor::MoveTo(x, y))?;
        }
        Ok(())
    }

    // TODO: Maybe we keep track of user stats in the future and have a page for rendering stats :eyes: or its like in the results page to compare historical
    fn render_results(&self, stdout: &mut Stdout, results: &TestResults) -> std::io::Result<()> {
        let (width, height) = terminal::size().unwrap_or((80, 24));

        queue!(stdout, cursor::Hide)?;

        let cx = width / 2;
        let cy = height / 3;

        let wpm_text = format!("{:.0}", results.wpm);
        let wpm_label = "wpm";
        queue!(
            stdout,
            cursor::MoveTo(cx.saturating_sub(wpm_text.len() as u16), cy),
            SetForegroundColor(self.theme.primary),
            Print(&wpm_text),
            ResetColor,
        )?;
        queue!(
            stdout,
            cursor::MoveTo(
                cx.saturating_sub(wpm_label.len() as u16),
                cy.saturating_sub(1)
            ),
            SetForegroundColor(self.theme.secondary),
            Print(wpm_label),
            ResetColor,
        )?;

        let stats_y = cy + 3;
        let stats = [
            ("raw", format!("{:.0}", results.raw_wpm)),
            ("acc", format!("{:.1}%", results.accuracy)),
            ("con", format!("{:.0}%", results.consistency)),
            ("time", format!("{:.1}s", results.time_elapsed)),
            (
                "chars",
                format!("{}/{}", results.correct_chars, results.total_chars),
            ),
        ];

        let col_width = 14u16;
        let total_width = col_width * stats.len() as u16;
        let start_x = cx.saturating_sub(total_width / 2);

        for (i, (label, value)) in stats.iter().enumerate() {
            let x = start_x + (i as u16 * col_width);

            queue!(
                stdout,
                cursor::MoveTo(x, stats_y),
                SetForegroundColor(self.theme.secondary),
                Print(label),
                ResetColor,
            )?;

            queue!(
                stdout,
                cursor::MoveTo(x, stats_y + 1),
                SetForegroundColor(self.theme.primary),
                Print(value),
                ResetColor,
            )?;
        }

        let hint = "tab menu  |  esc quit";
        queue!(
            stdout,
            cursor::MoveTo(cx.saturating_sub(hint.len() as u16 / 2), stats_y + 4),
            SetForegroundColor(self.theme.secondary),
            Print(hint),
            ResetColor,
        )?;

        Ok(())
    }

    fn render_test(
        &self,
        stdout: &mut Stdout,
        test: &TypingTest,
        view: &TestView,
    ) -> std::io::Result<()> {
        queue!(stdout, cursor::Show)?;

        let h_offest = view.viewport().h_offset;

        if let Some(remaining) = test.remaining_secs() {
            let timer_text = format!("{remaining}");
            let color = if test.has_started() {
                self.theme.primary
            } else {
                self.theme.secondary
            };

            queue!(
                stdout,
                cursor::MoveTo(h_offest, TEXT_START_ROW - 2),
                SetForegroundColor(color),
                Print(&timer_text),
                ResetColor
            )?
        }

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
            let word_y = TEXT_START_ROW + line_number as u16;

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
            std::cmp::Ordering::Less => self.render_completed_word(stdout, word),
            std::cmp::Ordering::Equal => {
                self.render_active_word(stdout, word, test.current_input())
            }
            std::cmp::Ordering::Greater => self.render_upcoming_word(stdout, &word.target),
        }
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

    fn render_upcoming_word(&self, stdout: &mut Stdout, target: &str) -> std::io::Result<()> {
        queue!(
            stdout,
            SetForegroundColor(self.theme.secondary),
            Print(target),
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
