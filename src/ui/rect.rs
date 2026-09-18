/// A rectangular region of the screen, in cells
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// A `width` x `height` rect centred in this one, shrunk to fit if needed.
    pub fn centered(self, width: u16, height: u16) -> Self {
        let width = width.min(self.width);
        let height = height.min(self.height);
        Self::new(
            self.x + (self.width - width) / 2,
            self.y + (self.height - height) / 2,
            width,
            height,
        )
    }

    /// Splits off the first `n` rows, returning `(top, rest)`.
    pub fn split_top(self, n: u16) -> (Self, Self) {
        let n = n.min(self.height);
        (
            Self::new(self.x, self.y, self.width, n),
            Self::new(self.x, self.y + n, self.width, self.height - n),
        )
    }

    /// Splits off the first `n` columns, returning `(left, rest)`.
    pub fn split_left(self, n: u16) -> (Self, Self) {
        let n = n.min(self.width);
        (
            Self::new(self.x, self.y, n, self.height),
            Self::new(self.x + n, self.y, self.width - n, self.height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_shrinks_to_fit() {
        let area = Rect::new(0, 0, 10, 4);

        assert_eq!(area.centered(4, 2), Rect::new(3, 1, 4, 2));
        assert_eq!(area.centered(20, 20), area);
    }

    #[test]
    fn splits_cover_the_whole_rect() {
        let area = Rect::new(2, 3, 10, 4);

        assert_eq!(
            area.split_top(1),
            (Rect::new(2, 3, 10, 1), Rect::new(2, 4, 10, 3))
        );
        assert_eq!(area.split_left(20), (area, Rect::new(12, 3, 0, 4)));
    }
}
