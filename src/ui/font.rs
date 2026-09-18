use crate::ui::frame::{Frame, Style};

/// Rows a big glyph takes up.
pub const HEIGHT: u16 = 3;

/// 3x5 pixel bitmaps, one byte per pixel row, the low three bits left to right.
fn glyph(c: char) -> Option<[u8; 5]> {
    Some(match c {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        _ => return None,
    })
}

/// Draws `text` [`HEIGHT`] rows tall, packing two pixel rows into each cell
/// with half blocks. Characters without a glyph are skipped.
pub fn draw(f: &mut Frame, x: u16, y: u16, text: &str, style: Style) {
    for (i, c) in text.chars().enumerate() {
        let Some(pixels) = glyph(c) else {
            continue;
        };
        let left = x + 4 * i as u16;

        for row in 0..HEIGHT {
            let top = pixels[2 * row as usize];
            let bottom = pixels.get(2 * row as usize + 1).copied().unwrap_or(0);

            for col in 0..3 {
                let bit = 0b100 >> col;
                let ch = match (top & bit != 0, bottom & bit != 0) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => continue,
                };
                f.put(left + col, y + row, ch, style);
            }
        }
    }
}
