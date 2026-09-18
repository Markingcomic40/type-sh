use crossterm::style::Color;

use crate::core::stats::Second;
use crate::ui::frame::{spans_width, text_width, Frame, Span, Style};
use crate::ui::rect::Rect;
use crate::ui::theme::Theme;

/// Rows around the plot: the legend above; the axis, errors and time labels below.
const CHROME_ROWS: u16 = 4;

/// Bit for the dot at [row][column] of a braille cell.
const DOTS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

const BRAILLE_BLANK: u32 = 0x2800;

pub fn draw(f: &mut Frame, area: Rect, theme: &Theme, timeline: &[Second]) {
    if timeline.is_empty() || area.height <= CHROME_ROWS || area.width < 12 {
        return;
    }

    let peak = timeline
        .iter()
        .map(|s| s.wpm.max(s.raw))
        .fold(0.0, f64::max);
    let top = nice_ceiling(peak);

    let (legend, rest) = area.split_top(1);
    let label_width = text_width(&top.to_string());
    let (labels, plot) = rest.split_left(label_width + 2);
    let plot = Rect::new(plot.x, plot.y, plot.width, rest.height - 3);

    draw_legend(f, legend, theme);
    draw_y_axis(f, labels, plot, top, theme);

    let xs = x_positions(timeline.len(), dots_wide(plot));
    let y_of = |value: f64| {
        let max = dots_tall(plot) - 1;
        max - (value / f64::from(top) * f64::from(max)).round() as i32
    };

    let mut canvas = Canvas::new(plot);
    let raw: Vec<_> = xs
        .iter()
        .zip(timeline)
        .map(|(&x, s)| (x, y_of(s.raw)))
        .collect();
    let wpm: Vec<_> = xs
        .iter()
        .zip(timeline)
        .map(|(&x, s)| (x, y_of(s.wpm)))
        .collect();
    // WPM goes on last so it wins any cell the two lines share.
    canvas.polyline(&raw, theme.dim);
    canvas.polyline(&wpm, theme.accent);
    canvas.draw(f);

    let error_row = plot.bottom() + 1;
    for (&x, second) in xs.iter().zip(timeline) {
        if second.errors > 0 {
            f.put(
                plot.x + (x / 2) as u16,
                error_row,
                '✕',
                Style::fg(theme.error),
            );
        }
    }

    draw_time_labels(f, plot, &xs, theme);
}

/// Rounds `value` up to a number that makes clean axis labels, halves included.
fn nice_ceiling(value: f64) -> u32 {
    let step = match value {
        v if v <= 100.0 => 10.0,
        v if v <= 200.0 => 20.0,
        _ => 50.0,
    };
    ((value / step).ceil() * step).max(step) as u32
}

fn dots_wide(plot: Rect) -> i32 {
    i32::from(plot.width) * 2
}

fn dots_tall(plot: Rect) -> i32 {
    i32::from(plot.height) * 4
}

/// Spreads `n` points evenly across `width` dots.
fn x_positions(n: usize, width: i32) -> Vec<i32> {
    if n == 1 {
        return vec![0];
    }
    let last = f64::from(width - 1);
    (0..n)
        .map(|i| (i as f64 * last / (n - 1) as f64).round() as i32)
        .collect()
}

fn draw_legend(f: &mut Frame, area: Rect, theme: &Theme) {
    let label = Style::fg(theme.dim);
    let spans = [
        Span::new("━━", Style::fg(theme.accent)),
        Span::new(" wpm   ", label),
        Span::new("━━", Style::fg(theme.dim)),
        Span::new(" raw   ", label),
        Span::new("✕", Style::fg(theme.error)),
        Span::new(" errors", label),
    ];
    let x = area.right().saturating_sub(spans_width(&spans));
    f.print_spans(x, area.y, &spans);
}

/// Labels the top, middle and bottom of the plot, with the axis line beside them.
fn draw_y_axis(f: &mut Frame, labels: Rect, plot: Rect, top: u32, theme: &Theme) {
    let axis = Style::fg(theme.faint);
    let axis_x = plot.x - 1;
    let mut label = |y: u16, value: u32| {
        let text = value.to_string();
        let x = labels.x + labels.width - 2 - text_width(&text);
        f.print(x, y, &text, Style::fg(theme.dim));
    };

    label(plot.y, top);
    label(plot.bottom(), 0);
    // A middle label only lines up with a row when the height is even.
    let middle = plot
        .height
        .is_multiple_of(2)
        .then_some(plot.y + plot.height / 2);
    if let Some(y) = middle {
        label(y, top / 2);
    }

    for y in plot.y..plot.bottom() {
        let tick = y == plot.y || Some(y) == middle;
        f.put(axis_x, y, if tick { '┤' } else { '│' }, axis);
    }
    f.put(axis_x, plot.bottom(), '└', axis);
    for x in plot.x..plot.right() {
        f.put(x, plot.bottom(), '─', axis);
    }
}

/// Labels seconds along the bottom, spaced out so they never touch.
fn draw_time_labels(f: &mut Frame, plot: Rect, xs: &[i32], theme: &Theme) {
    let n = xs.len();
    let widest = n.to_string().len() + 2;
    let step = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600]
        .into_iter()
        .find(|step| (n / step + 1) * widest <= usize::from(plot.width))
        .unwrap_or(600);

    let y = plot.bottom() + 2;
    let mut free = plot.x;
    for (i, &x) in xs.iter().enumerate() {
        let second = i + 1;
        if second != 1 && second % step != 0 {
            continue;
        }

        let text = second.to_string();
        let width = text_width(&text);
        let x = (plot.x + (x / 2) as u16)
            .saturating_sub(width / 2)
            .max(free);
        if x + width > plot.right() {
            break;
        }
        f.print(x, y, &text, Style::fg(theme.dim));
        free = x + width + 1;
    }
}

/// A grid of braille cells to plot dots onto.
struct Canvas {
    area: Rect,
    cells: Vec<(u8, Color)>,
}

impl Canvas {
    fn new(area: Rect) -> Self {
        let len = usize::from(area.width) * usize::from(area.height);
        Self {
            area,
            cells: vec![(0, Color::Reset); len],
        }
    }

    fn dot(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= dots_wide(self.area) || y >= dots_tall(self.area) {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        let cell = &mut self.cells[(y / 4) * usize::from(self.area.width) + x / 2];
        cell.0 |= DOTS[y % 4][x % 2];
        cell.1 = color;
    }

    /// Bresenham's line.
    fn line(&mut self, (mut x0, mut y0): (i32, i32), (x1, y1): (i32, i32), color: Color) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = ((x1 - x0).signum(), (y1 - y0).signum());
        let mut err = dx + dy;

        loop {
            self.dot(x0, y0, color);
            if (x0, y0) == (x1, y1) {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Joins the points up. A lone point becomes a flat line across the plot.
    fn polyline(&mut self, points: &[(i32, i32)], color: Color) {
        match points {
            [] => {}
            [(_, y)] => self.line((0, *y), (dots_wide(self.area) - 1, *y), color),
            _ => {
                for pair in points.windows(2) {
                    self.line(pair[0], pair[1], color);
                }
            }
        }
    }

    fn draw(&self, f: &mut Frame) {
        let width = usize::from(self.area.width);
        for (i, &(bits, color)) in self.cells.iter().enumerate() {
            if bits == 0 {
                continue;
            }
            let ch = char::from_u32(BRAILLE_BLANK + u32::from(bits)).expect("braille is valid");
            let (x, y) = ((i % width) as u16, (i / width) as u16);
            f.put(self.area.x + x, self.area.y + y, ch, Style::fg(color));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_ceiling_rounds_up_to_clean_steps() {
        assert_eq!(nice_ceiling(0.0), 10);
        assert_eq!(nice_ceiling(87.0), 90);
        assert_eq!(nice_ceiling(101.0), 120);
        assert_eq!(nice_ceiling(201.0), 250);
    }

    #[test]
    fn x_positions_span_the_whole_width() {
        assert_eq!(x_positions(1, 10), [0]);
        assert_eq!(x_positions(3, 11), [0, 5, 10]);
    }

    #[test]
    fn dots_combine_within_a_cell() {
        let mut canvas = Canvas::new(Rect::new(0, 0, 1, 1));
        canvas.dot(0, 0, Color::Red);
        canvas.dot(1, 3, Color::Blue);

        assert_eq!(canvas.cells[0], (0x01 | 0x80, Color::Blue));
    }
}
