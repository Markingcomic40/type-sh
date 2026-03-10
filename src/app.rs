use std::io::Write;

use crossterm::{
    queue,
    style::{Print, ResetColor, SetForegroundColor},
};

use crate::error::Result;
use crate::terminal::Terminal;
pub struct App {
    terminal: Terminal,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut terminal = Terminal::new()?;

        queue!(
            terminal.stdout(),
            SetForegroundColor(crossterm::style::Color::Cyan),
            Print("Grr \n"),
            ResetColor,
            Print("Small grr"),
        )?;

        terminal.stdout().flush()?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        Ok(Self { terminal })
    }

    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }
}
