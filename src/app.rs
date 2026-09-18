use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::error::Result;
use crate::screen::home::Home;
use crate::screen::{Next, Screen};
use crate::settings::Settings;
use crate::terminal::Terminal;
use crate::ui::theme::Theme;

/// Longest to wait so eg timer keeps tickin
const FRAME: Duration = Duration::from_millis(50);

pub struct App {
    terminal: Terminal,
    settings: Settings,
    theme: Theme,
    theme_name: String,
    screen: Screen,
    running: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let settings = Settings::load();
        let theme = Theme::load(&settings.theme).unwrap_or_default();

        Ok(Self {
            terminal: Terminal::new()?,
            theme_name: settings.theme.clone(),
            settings,
            theme,
            screen: Screen::Home(Home::default()),
            running: true,
        })
    }

    pub fn run(mut self) -> Result<()> {
        while self.running {
            let Self {
                terminal,
                screen,
                theme,
                settings,
                ..
            } = &mut self;
            terminal.draw(theme.background, |f| screen.draw(f, theme, settings))?;

            if event::poll(FRAME)? {
                // Drain everything queued so a burst of keys costs one redraw
                while event::poll(Duration::ZERO)? {
                    self.handle(event::read()?);
                }
            }

            let next = self.screen.tick();
            self.go(next);
        }

        self.settings.save()
    }

    fn handle(&mut self, event: Event) {
        // on resize the next frame is drawn at the new size
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        let next = self.screen.handle_key(key, &mut self.settings);
        self.go(next);
        self.sync_theme();
    }

    fn go(&mut self, next: Next) {
        match next {
            Next::Stay => {}
            Next::To(screen) => self.screen = screen,
            Next::Quit => self.running = false,
        }
    }

    fn sync_theme(&mut self) {
        if self.settings.theme == self.theme_name {
            return;
        }

        if let Ok(theme) = Theme::load(&self.settings.theme) {
            self.theme = theme;
        }
        self.theme_name.clone_from(&self.settings.theme);
    }
}
