//! The typing area, shared by single player and races

use crossterm::style::Color;

use crate::core::config::Limit;
use crate::core::typing_test::{Glyph, TypingTest, Word};
use crate::ui::frame::{Frame, Style};
use crate::ui::rect::Rect;
use crate::ui::theme::Theme;

/// Lines u see ont he screen of words at most
pub const LINES: u16 = 3;

/// Widest the words get
const MAX_TEXT_WIDTH: u16 = 80;

/// Someone else's caret, drawn in their colour on a letter not typed yet
pub struct Marker {
    pub word: usize,
    pub char: usize,
    pub color: Color,
}

/// Where the words go on `screen`
pub fn area(screen: Rect) -> Rect {
    let width = screen.width.saturating_sub(8).min(MAX_TEXT_WIDTH);
    Rect::new(
        screen.x + (screen.width - width) / 2,
        (screen.height / 2).saturating_sub(LINES),
        width,
        LINES,
    )
}

/// Time left, words done, or words typed etc.
pub fn counter(test: &TypingTest) -> String {
    match test.config().limit {
        Limit::Time(secs) => secs.saturating_sub(test.elapsed().as_secs()).to_string(),
        Limit::Words(n) => format!("{}/{n}", test.current()),
        Limit::None => test.current().to_string(),
    }
}

pub fn draw(
    f: &mut Frame,
    theme: &Theme,
    area: Rect,
    words: &[Word],
    current: usize,
    caret: bool,
    markers: &[Marker],
) {
    if words.is_empty() {
        return;
    }
    let current = current.min(words.len() - 1);
    let layout = wrap(words.iter().map(Word::width), usize::from(area.width));
    let first = layout[current].0.saturating_sub(1);

    for (i, (word, &(line, col))) in words.iter().zip(&layout).enumerate() {
        if line < first {
            continue;
        }
        if line >= first + usize::from(LINES) {
            break;
        }

        let x = area.x + col as u16;
        let y = area.y + (line - first) as u16;
        let mark = |c: usize| {
            markers
                .iter()
                .find(|m| m.word == i && m.char == c)
                .map(|m| m.color)
        };
        draw_word(f, theme, x, y, word, i < current, mark);

        if i == current && caret {
            f.set_cursor(x + word.typed.chars().count() as u16, y);
        }
    }
}

fn draw_word(
    f: &mut Frame,
    theme: &Theme,
    x: u16,
    y: u16,
    word: &Word,
    finished: bool,
    mark: impl Fn(usize) -> Option<Color>,
) {
    let underline = finished && !word.is_correct();

    for (i, glyph) in word.glyphs().enumerate() {
        let (ch, style) = match glyph {
            Glyph::Correct(c) => (c, Style::fg(theme.correct)),
            Glyph::Incorrect(c) => (c, Style::fg(theme.error)),
            Glyph::Extra(c) => (c, Style::fg(theme.error_extra)),
            Glyph::Untyped(c) => match mark(i) {
                Some(color) => (c, Style::fg(color).bold()),
                None => (c, Style::fg(theme.dim)),
            },
        };
        f.put(x + i as u16, y, ch, style.underline(underline));
    }
}

/// Lays words out left to right with a space between, wrapping at width yield for each word line n column
fn wrap(widths: impl Iterator<Item = usize>, width: usize) -> Vec<(usize, usize)> {
    let (mut line, mut col) = (0, 0);

    widths
        .map(|w| {
            // A word too long for any line still gets one to itself.
            if col > 0 && col + w > width {
                line += 1;
                col = 0;
            }
            let pos = (line, col);
            col += w + 1;
            pos
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_breaks_before_words_that_would_overflow() {
        let layout = wrap([3, 3, 3, 10, 1].into_iter(), 8);

        assert_eq!(layout, [(0, 0), (0, 4), (1, 0), (2, 0), (3, 0)]);
    }
}
