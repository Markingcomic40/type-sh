use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::config::Limit;
use crate::core::typing_test::TypingTest;
use crate::error::Result;
use crate::screen::bar::{self, Bar, Kind, Outcome};
use crate::screen::home::Home;
use crate::screen::results::Results;
use crate::screen::words;
use crate::screen::{self, Next, Screen};
use crate::settings::Settings;
use crate::ui::frame::{Frame, Span, Style};
use crate::ui::theme::Theme;

pub struct Play {
    test: TypingTest,
    /// Some while the config bar has focus rather than the words
    bar: Option<Bar>,
    error: Option<String>,
}

impl Play {
    pub fn new(settings: &Settings) -> Result<Self> {
        Ok(Self {
            test: TypingTest::new(settings.test_config(rand::random()))?,
            bar: None,
            error: None,
        })
    }

    pub fn tick(&mut self) -> Next {
        if self.test.is_finished() {
            return Next::To(Screen::Results(Box::new(Results::new(&self.test))));
        }
        Next::Stay
    }

    pub fn handle_key(&mut self, key: KeyEvent, settings: &mut Settings) -> Next {
        self.error = None;
        if let Some(bar) = &mut self.bar {
            match bar.handle_key(key, settings) {
                Outcome::Stay => {}
                Outcome::Changed => self.restart(settings),
                Outcome::Close => self.bar = None,
                Outcome::Pass => {
                    self.bar = None;
                    return self.handle_key(key, settings);
                }
                Outcome::Error(e) => self.error = Some(e),
            }
            return Next::Stay;
        }

        let started = self.test.has_started();
        match key.code {
            KeyCode::Tab => self.restart(settings),
            KeyCode::Esc if started => self.test.stop(),
            KeyCode::Esc => return Next::To(Screen::Home(Home::default())),
            KeyCode::Up if !started => self.bar = Some(Bar::open(settings, Kind::Solo)),
            KeyCode::Backspace => self.test.backspace(),
            KeyCode::Char(' ') => self.test.space(),
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.test.type_char(c)
            }
            _ => {}
        }

        Next::Stay
    }

    fn restart(&mut self, settings: &Settings) {
        match TypingTest::new(settings.test_config(rand::random())) {
            Ok(test) => self.test = test,
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme, settings: &Settings) {
        let area = f.area();
        let words = words::area(area);

        self.draw_counter(f, theme, words.x, words.y - 2);
        words::draw(
            f,
            theme,
            words,
            self.test.words(),
            self.test.current(),
            self.bar.is_none(),
            &[],
        );

        if let Some(error) = &self.error {
            let span = Span::new(error.as_str(), Style::fg(theme.error));
            f.print_centered(area, words.bottom() + 2, &[span]);
        }

        if self.test.has_started() {
            if self.test.config().limit == Limit::None {
                screen::draw_hints(f, theme, &[("esc", "finish")]);
            }
            return;
        }

        bar::draw(
            f,
            theme,
            settings,
            Kind::Solo,
            self.bar.as_ref(),
            area.y + 2,
        );

        let hints: &[_] = match &self.bar {
            Some(bar) if bar.is_editing() => &[("enter", "confirm"), ("esc", "cancel")],
            Some(_) => &[("←→", "move"), ("enter", "select"), ("↓", "back")],
            None => &[("tab", "restart"), ("↑", "options"), ("esc", "home")],
        };
        screen::draw_hints(f, theme, hints);
    }

    /// Time left, words done, or words typed etc.
    fn draw_counter(&self, f: &mut Frame, theme: &Theme, x: u16, y: u16) {
        let test = &self.test;
        let text = words::counter(test);
        let color = if test.has_started() {
            theme.accent
        } else {
            theme.dim
        };
        f.print(x, y, &text, Style::fg(color));
    }
}
