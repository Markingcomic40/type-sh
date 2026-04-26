use crossterm::{
    cursor, queue,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{Cursor, Stdout, Write};

use crate::{
    core::{game::GameState, typing_test::TypingTest},
    error::Result,
    ui::test_view::{TestView, WordLayout},
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

        let layouts = view.layouts();
        eprintln!("{layouts:#?}");

        for &WordLayout {
            word_index,
            line_number,
            x_position,
        } in layouts.iter()
        {
            queue!(
                stdout,
                cursor::MoveTo(x_position as u16, line_number as u16),
                SetForegroundColor(crossterm::style::Color::Cyan),
                Print(&test.words()[word_index].target),
                ResetColor,
            )?;
            // queue!(stdout);
        }

        stdout.flush()?;

        Ok(())
    }
}
