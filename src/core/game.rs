use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::config::TestConfig;
use crate::error::Result;
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

        let view = TestView::new();

        Ok(Self {
            state: GameState::Running { test, view },
            should_quit: false,
            terminal_width,
        })
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn handle_keys(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
        }
    }
}
