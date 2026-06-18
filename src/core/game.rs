use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::core::config::TestConfig;
use crate::core::menu::MenuState;
use crate::error::Result;
use crate::preferences::Preferences;
use crate::ui::test_view::ViewportConfig;
use crate::{core::typing_test::TypingTest, ui::test_view::TestView};

pub enum GameState {
    Menu(MenuState),
    Running {
        test: Box<TypingTest>,
        view: TestView,
    },
    Results,
}

pub struct Game {
    pub(crate) state: GameState,
    /// Preserved menu state so we can go back to it later. I dont like it but seems like a blunder from 5 moves ago so ill accept my fate for now and fix it later :sob: TODO
    menu_snapshot: MenuState,
    should_quit: bool,
    terminal_width: u16, // TODO: Do we really even need this......
}

impl Game {
    pub fn new(prefs: &Preferences, terminal_width: u16) -> Result<Self> {
        let config = TestConfig::default();

        // let test = match TypingTest::new(config) {
        //     Ok(type_test) => Box::new(type_test),
        //     Err(e) => {
        //         println!("NO BUENO shoudlnt happen but anywyas either way ill handle this later");
        //         return Err(e);
        //     }
        // };

        // let view: TestView = TestView::new(ViewportConfig::new(terminal_width, 3));

        let menu = MenuState::new(prefs);
        let snapshot = menu.clone();

        Ok(Self {
            state: GameState::Menu(menu),
            menu_snapshot: snapshot,
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

            if test.is_finished() {
                self.should_quit = true;
            }
        }
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn preferences(&self) -> Preferences {
        match &self.state {
            GameState::Menu(menu) => menu.to_preferences(),
            _ => self.menu_snapshot.to_preferences(),
        }
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
            GameState::Menu(_) => self.handle_menu_keys(key),
            GameState::Running { test, view } => Self::handle_test_keys(test, view, key),
            _ => {}
        }
    }

    fn handle_menu_keys(&mut self, key: KeyEvent) {
        // Rust shenanigans: Cant have a &mut of menu and start or quit :)
        enum Action {
            Start,
            Quit,
            Nada,
        }

        let action = {
            let GameState::Menu(menu) = &mut self.state else {
                return;
            };

            let action = if menu.is_inputting() {
                match key.code {
                    KeyCode::Enter => menu.confirm_input(),
                    KeyCode::Esc => menu.cancel_input(),
                    KeyCode::Backspace => menu.input_backspace(),
                    KeyCode::Char(c) => menu.input_char(c),
                    _ => {}
                }
                Action::Nada
            } else {
                match key.code {
                    KeyCode::Enter => {
                        // menu.end
                        Action::Start
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        menu.shift_next();
                        Action::Nada
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        menu.shift_prev();
                        Action::Nada
                    }
                    KeyCode::Right => {
                        menu.select_next();
                        Action::Nada
                    }
                    KeyCode::Left => {
                        menu.select_prev();
                        Action::Nada
                    }
                    KeyCode::Char('e') => {
                        if menu.is_custom_selected() {
                            menu.enter_input_mode();
                        }
                        Action::Nada
                    }
                    KeyCode::Esc => Action::Quit,
                    _ => Action::Nada,
                }
            };

            action
        };

        // NOTE: SUrprisingly i accidentally put this isndie the above and it worked?? Maybe it sees menu isnt used anymore and so it drops??
        match action {
            Action::Nada => {}
            Action::Start => self.start_test(),
            Action::Quit => self.should_quit = true,
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

    fn start_test(&mut self) {
        let (config, menu_clone) = {
            let GameState::Menu(menu) = &self.state else {
                return;
            };

            (menu.to_test_config(), menu.clone())
        };

        self.menu_snapshot = menu_clone;

        let viewport: TestView = TestView::new(ViewportConfig::new(self.terminal_width, 3));

        let test = match TypingTest::new(config) {
            Ok(type_test) => Box::new(type_test),
            Err(e) => {
                if let GameState::Menu(menu) = &mut self.state {
                    // panic!("ERROR: {}", e);
                    menu.set_error("could not load word set");
                }
                return;
            }
        };

        self.state = GameState::Running {
            test,
            view: viewport,
        };
    }
}
