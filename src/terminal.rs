use std::io::{self, BufWriter, Stdout, Write};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::ui::frame::{Cell, Frame};

/// Owns the terminal: raw mode and the alternate screen are entered on creation and restored on drop.
///
/// Drawing is double buffered. 
pub struct Terminal {
    out: BufWriter<Stdout>,
    front: Frame,
    back: Frame,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = BufWriter::new(io::stdout());
        execute!(
            out,
            EnterAlternateScreen,
            cursor::Hide,
            SetCursorStyle::SteadyBar
        )?;

        Ok(Self {
            out,
            front: Frame::default(),
            back: Frame::default(),
        })
    }

    pub fn draw(&mut self, background: Color, render: impl FnOnce(&mut Frame)) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        self.back.reset(width, height, background);
        render(&mut self.back);
        self.flush()?;
        std::mem::swap(&mut self.front, &mut self.back);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        let resized = self.front.area() != self.back.area();
        if resized {
            queue!(self.out, terminal::Clear(ClearType::All))?;
        }

        let width = usize::from(self.back.area().width);
        let front = self.front.cells();
        let mut pen: Option<Cell> = None;
        let mut next: Option<(u16, u16)> = None;
        let mut dirty = false;

        for (i, cell) in self.back.cells().iter().enumerate() {
            if !resized && front[i] == *cell {
                continue;
            }
            if !dirty {
                // Keep the caret from flying around the screen while we draw.
                queue!(self.out, cursor::Hide)?;
                dirty = true;
            }

            let pos = ((i % width) as u16, (i / width) as u16);

            if next != Some(pos) {
                queue!(self.out, cursor::MoveTo(pos.0, pos.1))?;
            }

            if !pen.is_some_and(|pen| pen.same_style(cell)) {
                set_pen(&mut self.out, cell)?;
                pen = Some(*cell);
            }
            
            queue!(self.out, Print(cell.ch))?;
            next = Some((pos.0 + 1, pos.1));
        }

        if dirty || self.back.cursor() != self.front.cursor() {
            match self.back.cursor() {
                Some((x, y)) => queue!(self.out, cursor::MoveTo(x, y), cursor::Show)?,
                None => queue!(self.out, cursor::Hide)?,
            }
        }

        self.out.flush()
    }
}

fn set_pen(out: &mut impl Write, cell: &Cell) -> io::Result<()> {
    queue!(
        out,
        SetAttribute(Attribute::Reset),
        SetForegroundColor(cell.fg),
        SetBackgroundColor(cell.bg)
    )?;
    if cell.bold {
        queue!(out, SetAttribute(Attribute::Bold))?;
    }
    if cell.underline {
        queue!(out, SetAttribute(Attribute::Underlined))?;
    }
    Ok(())
}

fn restore(out: &mut impl Write) {
    let _ = execute!(
        out,
        SetAttribute(Attribute::Reset),
        SetCursorStyle::DefaultUserShape,
        cursor::Show,
        LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore(&mut self.out);
    }
}

/// Puts the terminal back before a panic message prints, so its readable
pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore(&mut io::stdout());
        original(info);
    }));
}
