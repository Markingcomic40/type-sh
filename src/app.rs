use std::io::Write;

use crossterm::{
    queue,
    style::{Print, ResetColor, SetForegroundColor},
};

use crate::terminal::Terminal;
use crate::{core::word_list::WordPool, error::Result};
pub struct App {
    terminal: Terminal,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut wordlist = WordPool::new("english")?;

        let word = wordlist.next_word();

        println!("{}", word);

        let tenwords = wordlist.gen_words(10);

        println!("{:?}", tenwords);

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
