use std::io::{self, Stdout, stdout};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
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
