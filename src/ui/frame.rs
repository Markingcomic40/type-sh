use std::borrow::Cow;

use crossterm::style::Color;

use crate::ui::rect::Rect;

/// How to draw text. fg:None is keep curr color
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
    pub underline: bool,
}

impl Style {
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            bold: false,
            underline: false,
        }
    }

    pub const fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    pub const fn underline(self, underline: bool) -> Self {
        Self { underline, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
}

impl Cell {
    fn blank(bg: Color) -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg,
            bold: false,
            underline: false,
        }
    }

    /// Whether the two cells can be written without changing pen
    pub fn same_style(&self, other: &Self) -> bool {
        (self.fg, self.bg, self.bold, self.underline)
            == (other.fg, other.bg, other.bold, other.underline)
    }
}

/// A run of text in one style
pub struct Span<'a> {
    pub text: Cow<'a, str>,
    pub style: Style,
}

impl<'a> Span<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

/// TOOD: Er this assumes width of chars but idk how that would work out with arabic text n stuff
pub fn text_width(text: &str) -> u16 {
    text.chars().count().try_into().unwrap_or(u16::MAX)
}

pub fn spans_width(spans: &[Span]) -> u16 {
    spans
        .iter()
        .fold(0, |w, span| w.saturating_add(text_width(&span.text)))
}

/// One screen's worth of cells, plus where the caret goes so it aint like going to woeird places
#[derive(Default)]
pub struct Frame {
    area: Rect,
    cells: Vec<Cell>,
    cursor: Option<(u16, u16)>,
}

impl Frame {
    /// Clears the frame to bg + resizing if needed
    pub fn reset(&mut self, width: u16, height: u16, bg: Color) {
        self.area = Rect::new(0, 0, width, height);
        self.cells.clear();
        self.cells
            .resize(usize::from(width) * usize::from(height), Cell::blank(bg));
        self.cursor = None;
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        if x < self.area.width && y < self.area.height {
            self.cursor = Some((x, y));
        }
    }

    /// Sets one cell. Anything off screen is clipped
    pub fn put(&mut self, x: u16, y: u16, ch: char, style: Style) {
        if x >= self.area.width || y >= self.area.height {
            return;
        }

        let cell = &mut self.cells[usize::from(y) * usize::from(self.area.width) + usize::from(x)];
        cell.ch = ch;
        cell.bold = style.bold;
        cell.underline = style.underline;
        if let Some(fg) = style.fg {
            cell.fg = fg;
        }
    }

    /// Prints text from (x, y) and returns the column after it
    pub fn print(&mut self, x: u16, y: u16, text: &str, style: Style) -> u16 {
        text.chars().fold(x, |x, ch| {
            self.put(x, y, ch, style);
            x.saturating_add(1)
        })
    }

    /// Prints spans back to back from (x, y) and returns the column after them
    pub fn print_spans(&mut self, x: u16, y: u16, spans: &[Span]) -> u16 {
        spans
            .iter()
            .fold(x, |x, span| self.print(x, y, &span.text, span.style))
    }

    /// Prints spans centred horizontally in `area` and returns the column they start at
    pub fn print_centered(&mut self, area: Rect, y: u16, spans: &[Span]) -> u16 {
        let x = area.x + area.width.saturating_sub(spans_width(spans)) / 2;
        self.print_spans(x, y, spans);
        x
    }
}
