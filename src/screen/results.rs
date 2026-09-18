use crossterm::event::{KeyCode, KeyEvent};

use crate::core::config::{Limit, TestConfig};
use crate::core::stats::Report;
use crate::core::typing_test::TypingTest;
use crate::screen::home::Home;
use crate::screen::{self, Next, Screen};
use crate::settings::{display_name, Settings};
use crate::ui::frame::{spans_width, text_width, Frame, Span, Style};
use crate::ui::rect::Rect;
use crate::ui::theme::Theme;
use crate::ui::{chart, font};

/// Widest the results get before they stop reading as one block
const MAX_WIDTH: u16 = 100;

/// Rows under the chart: a gap, then the stats' label, value and detail rows
const STATS_HEIGHT: u16 = 5;

pub struct Results {
    config: TestConfig,
    report: Report,
}

/// ngl i got lazy and just yeeted the design from monkey t yep :sweat: Creds to them n their design team B)
impl Results {
    pub fn new(test: &TypingTest) -> Self {
        Self {
            config: test.config().clone(),
            report: test.report(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, settings: &Settings) -> Next {
        match key.code {
            KeyCode::Tab | KeyCode::Enter => Next::To(screen::play(settings)),
            KeyCode::Esc => Next::To(Screen::Home(Home::default())),
            _ => Next::Stay,
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &Theme) {
        let area = f.area();
        let chart_height = area.height.saturating_sub(11).clamp(7, 14);
        let block = area.centered(
            area.width.saturating_sub(6).min(MAX_WIDTH),
            chart_height + STATS_HEIGHT,
        );
        
        // Leave room below for the hints
        let block = Rect {
            y: block.y.saturating_sub(1),
            ..block
        };

        // Big digits need 9 rows beside the chart and room for "100%"
        let big = block.width >= 72 && chart_height >= 9;
        let (side, chart) = Rect::new(block.x, block.y, block.width, chart_height)
            .split_left(if big { 17 } else { 9 });

        self.draw_headline(f, theme, side, big);
        chart::draw(f, chart, theme, &self.report.timeline);
        self.draw_stats(
            f,
            theme,
            Rect::new(block.x, chart.bottom() + 2, block.width, 3),
        );

        screen::draw_hints(f, theme, &[("tab", "next test"), ("esc", "home")]);
    }

    fn draw_headline(&self, f: &mut Frame, theme: &Theme, area: Rect, big: bool) {
        let label = Style::fg(theme.dim);
        let value = Style::fg(theme.accent);
        let wpm = format!("{:.0}", self.report.wpm);
        let acc = format!("{:.0}%", self.report.accuracy);

        let height = if big { font::HEIGHT } else { 1 };
        let acc_y = area.y + height + 2;
        f.print(area.x, area.y, "wpm", label);
        f.print(area.x, acc_y, "acc", label);

        if big {
            font::draw(f, area.x, area.y + 1, &wpm, value);
            font::draw(f, area.x, acc_y + 1, &acc, value);
        } else {
            f.print(area.x, area.y + 1, &wpm, value.bold());
            f.print(area.x, acc_y + 1, &acc, value.bold());
        }
    }

    fn draw_stats(&self, f: &mut Frame, theme: &Theme, area: Rect) {
        let r = &self.report;
        let value = Style::fg(theme.accent);
        let chars = r.chars;
        let slash = || Span::new("/", Style::fg(theme.dim));

        let test = match self.config.limit {
            Limit::Time(secs) => format!("time {secs}"),
            Limit::Words(n) => format!("words {n}"),
            Limit::None => "zen".to_owned(),
        };

        let columns: [(&str, Vec<Span>, &str); 5] = [
            (
                "test",
                vec![Span::new(test, value)],
                display_name(&self.config.wordlist),
            ),
            ("raw", vec![Span::new(format!("{:.0}", r.raw), value)], ""),
            (
                "characters",
                vec![
                    Span::new(chars.correct.to_string(), value),
                    slash(),
                    Span::new(chars.incorrect.to_string(), Style::fg(theme.error)),
                    slash(),
                    Span::new(chars.extra.to_string(), Style::fg(theme.error_extra)),
                    slash(),
                    Span::new(chars.missed.to_string(), Style::fg(theme.dim)),
                ],
                "ok/wrong/extra/missed",
            ),
            (
                "consistency",
                vec![Span::new(format!("{:.0}%", r.consistency), value)],
                "",
            ),
            (
                "time",
                vec![Span::new(
                    format!("{:.0}s", r.duration.as_secs_f64()),
                    value,
                )],
                "",
            ),
        ];

        // Spread the columns edge to edge, dropping the details if they wont fit
        let widths = |details: bool| -> Vec<u16> {
            columns
                .iter()
                .map(|(label, spans, detail)| {
                    let detail = if details { text_width(detail) } else { 0 };
                    text_width(label).max(spans_width(spans)).max(detail)
                })
                .collect()
        };
        let fits = |widths: &[u16]| {
            widths.iter().sum::<u16>() + 2 * (widths.len() as u16 - 1) <= area.width
        };
        let details = fits(&widths(true));
        let widths = widths(details);
        let gap = area.width.saturating_sub(widths.iter().sum()) / (widths.len() as u16 - 1);

        let mut x = area.x;
        for ((label, spans, detail), width) in columns.iter().zip(widths) {
            f.print(x, area.y, label, Style::fg(theme.dim));
            f.print_spans(x, area.y + 1, spans);
            if details {
                f.print(x, area.y + 2, detail, Style::fg(theme.dim));
            }
            x += width + gap;
        }
    }
}
