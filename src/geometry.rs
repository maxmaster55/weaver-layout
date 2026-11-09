//! Points, sizes, rectangles and edge insets.

/// A width and a height.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    /// Horizontal extent.
    pub width: f64,
    /// Vertical extent.
    pub height: f64,
}

impl Size {
    /// A size of zero.
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    /// A new size.
    pub const fn new(width: f64, height: f64) -> Self {
        Size { width, height }
    }

    pub(crate) fn main(self, row: bool) -> f64 {
        if row {
            self.width
        } else {
            self.height
        }
    }

    pub(crate) fn cross(self, row: bool) -> f64 {
        if row {
            self.height
        } else {
            self.width
        }
    }

    pub(crate) fn from_axes(main: f64, cross: f64, row: bool) -> Self {
        if row {
            Size::new(main, cross)
        } else {
            Size::new(cross, main)
        }
    }
}

/// An axis-aligned rectangle: top-left corner plus size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

impl Rect {
    /// A new rectangle.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// A rectangle at the origin.
    pub const fn from_size(size: Size) -> Self {
        Rect::new(0.0, 0.0, size.width, size.height)
    }

    /// The size of the rectangle.
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// The right edge.
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// The bottom edge.
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// The centre point `(x, y)`.
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Whether the point lies inside (edges included on the top-left).
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }

    /// The rectangle shrunk by `insets` (never below zero size).
    pub fn inset(&self, insets: Insets) -> Rect {
        Rect::new(
            self.x + insets.left,
            self.y + insets.top,
            (self.width - insets.left - insets.right).max(0.0),
            (self.height - insets.top - insets.bottom).max(0.0),
        )
    }

    /// The rectangle moved by `(dx, dy)`.
    pub fn offset(&self, dx: f64, dy: f64) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    /// Linear interpolation towards `other`; `t = 0` is `self`.
    pub fn lerp(&self, other: &Rect, t: f64) -> Rect {
        let mix = |a: f64, b: f64| a + (b - a) * t;
        Rect::new(
            mix(self.x, other.x),
            mix(self.y, other.y),
            mix(self.width, other.width),
            mix(self.height, other.height),
        )
    }
}

/// Space on each side of a box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// Space above.
    pub top: f64,
    /// Space to the right.
    pub right: f64,
    /// Space below.
    pub bottom: f64,
    /// Space to the left.
    pub left: f64,
}

impl Insets {
    /// No space.
    pub const ZERO: Insets = Insets {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// The same space on every side.
    pub const fn all(value: f64) -> Self {
        Insets {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// `vertical` above and below, `horizontal` left and right.
    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Insets {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Total horizontal space.
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    /// Total vertical space.
    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }
}

impl From<f64> for Insets {
    fn from(value: f64) -> Self {
        Insets::all(value)
    }
}
