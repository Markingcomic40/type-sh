use std::io::{self, Stdout, stdout};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

// Terminal Wrapper: Enter raw mode on creation and restore terminal on drop.
pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, SetCursorStyle::SteadyBar,)?;

        Ok(Self { stdout })
    }

    pub fn stdout(&mut self) -> &mut Stdout {
        &mut self.stdout
    }

    pub fn size() -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(
            self.stdout,
            SetCursorStyle::DefaultUserShape,
            cursor::Show,
            LeaveAlternateScreen
        );

        let _ = terminal::disable_raw_mode();
    }
}
