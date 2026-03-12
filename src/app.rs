use std::{io::Write, time::Duration};

use crossterm::{
    event::{self, Event},
    queue,
    style::{Print, ResetColor, SetForegroundColor},
};

use crate::ui::renderer::Renderer;
use crate::{core::game::Game, terminal::Terminal};
use crate::{core::word_pool::WordPool, error::Result};
pub struct App {
    terminal: Terminal,
    renderer: Renderer,
    game: Game,
}

impl App {
    pub fn new() -> Result<Self> {
        // let mut wordlist = WordPool::new("english")?;

        // let word = wordlist.next_word();

        // println!("{}", word);

        // let tenwords = wordlist.gen_words(10);

        // println!("{:?}", tenwords);

        let terminal = Terminal::new()?;

        let renderer = Renderer::new();

        let (terminal_width, _) = Terminal::size();

        // renderer.render(terminal.stdout())?;

        let game = Game::new(terminal_width)?;

        Ok(Self {
            terminal,
            renderer,
            game,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.renderer
                .render(self.terminal.stdout(), self.game.state())?;

            if event::poll(Duration::from_millis(30))? {
                let ev = event::read()?;
                match ev {
                    Event::Key(key) => self.game.handle_keys(key),
                    _ => {}
                }
            }

            if self.game.should_quit() {
                break;
            }
        }

        Ok(())
    }
}
