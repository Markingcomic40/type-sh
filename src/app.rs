use std::time::Duration;

use crossterm::event;

use crate::core::game::{AppEvent, Game};
use crate::error::Result;
use crate::preferences::Preferences;
use crate::terminal::Terminal;
use crate::ui::renderer::Renderer;
use crate::ui::theme::Theme;

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

        let (terminal_width, _) = Terminal::size();

        // renderer.render(terminal.stdout())?;

        let terminal = Terminal::new()?;

        let prefs = Preferences::load();

        let game = Game::new(&prefs, terminal_width)?;
        let renderer = Renderer::new(Theme::gruvbox());

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
                for app_ev in self.game.handle_event(ev) {
                    match app_ev {
                        AppEvent::ThemeChanged(name) => {
                            if let Ok(theme) = Theme::load(&name) {
                                self.renderer.set_theme(theme);
                            }
                        }
                    }
                }
            }

            self.game.tick();

            if self.game.should_quit() {
                let _ = self.game.preferences().save()?;
                break;
            }
        }

        Ok(())
    }
}
