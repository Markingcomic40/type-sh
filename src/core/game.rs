use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::core::config::TestConfig;
use crate::error::Result;
use crate::ui::test_view::ViewportConfig;
use crate::{core::typing_test::TypingTest, ui::test_view::TestView};

pub enum GameState {
    Menu,
    Running {
        test: Box<TypingTest>,
        view: TestView,
    },
    Results,
}

pub struct Game {
    pub(crate) state: GameState,
    should_quit: bool,
    terminal_width: u16,
}

impl Game {
    pub fn new(terminal_width: u16) -> Result<Self> {
        let config = TestConfig::default();

        let test = match TypingTest::new(config) {
            Ok(type_test) => Box::new(type_test),
            Err(e) => {
                println!("NO BUENO shoudlnt happen but anywyas either way ill handle this later");
                return Err(e);
            }
        };

        let view: TestView = TestView::new(ViewportConfig::new(terminal_width, 3));

        Ok(Self {
            state: GameState::Running { test, view },
            should_quit: false,
            terminal_width,
        })
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn tick(&mut self) {
        if let GameState::Running { test, view } = &mut self.state {
            if view.update_layout(test) {
                test.append_words();
            }
        }
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                self.handle_keys(key);
            }
            Event::Resize(width, ..) => {
                self.terminal_width = width;

                if let GameState::Running { view, .. } = &mut self.state {
                    view.on_resize(width);
                }
            }
            _ => {}
        }
    }

    pub fn handle_keys(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
        }

        // Darn this borrow bs is annoying
        match &mut self.state {
            GameState::Running { test, view } => Self::handle_test_keys(test, view, key),
            _ => {}
        }
    }

    fn handle_test_keys(test: &mut TypingTest, view: &TestView, key: KeyEvent) {
        match key.code {
            KeyCode::Char(' ') => {
                test.handle_space();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                test.handle_char(c);
            }
            KeyCode::Backspace => {
                test.handle_backspace();
            }
            _ => {}
        }
    }

    // fn can_accept_char() {}
}
