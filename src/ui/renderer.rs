use crossterm::{
    cursor, queue,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{Stdout, Write};

use crate::error::Result;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, stdout: &mut Stdout) -> Result<()> {
        let (w, h) = terminal::size().unwrap_or((80, 24));

        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
        )?;

        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(crossterm::style::Color::Cyan),
            Print("Grr"),
            ResetColor,
            cursor::MoveTo(0, 1),
            Print("Small grr"),
        )?;

        stdout.flush()?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        Ok(())
    }
}
