use crossterm::{
    cursor, queue,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{Stdout, Write};

use crate::{
    core::{game::GameState, typing_test::TypingTest},
    error::Result,
    ui::test_view::TestView,
};

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, stdout: &mut Stdout, state: &GameState) -> Result<()> {
        let (w, h) = terminal::size().unwrap_or((80, 24));

        match state {
            GameState::Running { test, view } => self.render_test(stdout, test, view),
            _ => Ok(()),
        }

        // queue!(
        //     stdout,
        //     cursor::MoveTo(0, 0),
        //     terminal::Clear(terminal::ClearType::All),
        // )?;

        // queue!(
        //     stdout,
        //     cursor::MoveTo(0, 0),
        //     SetForegroundColor(crossterm::style::Color::Cyan),
        //     Print("Grr"),
        //     ResetColor,
        //     cursor::MoveTo(0, 1),
        //     Print("Small grr"),
        // )?;

        // stdout.flush()?;

        // std::thread::sleep(std::time::Duration::from_secs(3));

        // Ok(())
    }

    fn render_test(&self, stdout: &mut Stdout, test: &TypingTest, view: &TestView) -> Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
        )?;

        let layout = view.layouts();

        queue!(
            stdout,
            cursor::MoveTo(layout[0].x_position as u16, layout[0].line_number as u16),
            SetForegroundColor(crossterm::style::Color::Cyan),
            Print(&test.words()[layout[0].word_index].target),
            ResetColor,
            cursor::MoveTo(0, 1),
            Print("Small grr"),
        )?;

        stdout.flush()?;

        Ok(())
    }
}
