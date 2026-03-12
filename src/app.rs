use std::io::Write;

use crossterm::{
    queue,
    style::{Print, ResetColor, SetForegroundColor},
};

use crate::terminal::Terminal;
use crate::ui::renderer::Renderer;
use crate::{core::word_list::WordPool, error::Result};
pub struct App {
    terminal: Terminal,
    renderer: Renderer,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut wordlist = WordPool::new("english")?;

        let word = wordlist.next_word();

        println!("{}", word);

        let tenwords = wordlist.gen_words(10);

        println!("{:?}", tenwords);

        let mut terminal = Terminal::new()?;

        let renderer = Renderer::new();

        renderer.render(terminal.stdout())?;

        Ok(Self { terminal, renderer })
    }

    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }
}
