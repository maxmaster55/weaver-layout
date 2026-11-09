//! How a box wants to be laid out.

use crate::Insets;

/// A length along one axis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Length {
    /// Size to the content.
    #[default]
    Auto,
    /// A fixed size in your units (pixels, points, design units...).
    Px(f64),
    /// A fraction of the parent's inner size: `Percent(50.0)` is half.
    Percent(f64),
}

impl Length {
    pub(crate) fn resolve(self, parent: f64) -> Option<f64> {
        match self {
            Length::Auto => None,
            Length::Px(v) => Some(v),
            Length::Percent(p) => Some(parent * p / 100.0),
        }
    }
}

impl From<f64> for Length {
    fn from(value: f64) -> Self {
        Length::Px(value)
    }
}

impl From<i32> for Length {
    fn from(value: i32) -> Self {
        Length::Px(value as f64)
    }
}

/// The direction children flow in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Top to bottom.
    #[default]
    Column,
    /// Left to right.
    Row,
}

/// Where children sit on the cross axis (and, for [`Align::Stretch`], how
/// big they are).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Fill the cross axis unless the child has a fixed size.
    #[default]
    Stretch,
    /// Top (in a row) or left (in a column).
    Start,
    /// Centred.
    Center,
    /// Bottom (in a row) or right (in a column).
    End,
}

/// How leftover space on the main axis is distributed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    /// Pack at the start.
    #[default]
    Start,
    /// Pack in the middle.
    Center,
    /// Pack at the end.
    End,
    /// First and last child at the edges, equal gaps between.
    SpaceBetween,
    /// Equal space around every child.
    SpaceAround,
    /// Equal space between children and at both edges.
    SpaceEvenly,
}

/// How a container arranges its children.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Display {
    /// Flexbox-style: children in a row or column, with grow and shrink.
    #[default]
    Flex,
    /// A grid with a fixed number of equal columns, filled row by row.
    Grid {
        /// Number of columns (at least 1).
        columns: usize,
    },
}

/// Everything that affects how one box is laid out.
///
/// Styles are built with chainable methods:
///
/// ```
/// use weaver_layout::{Style, Align, Justify};
///
/// let toolbar = Style::row().gap(8.0).padding(12.0).align(Align::Center);
/// let spacer = Style::new().grow(1.0);
/// let icon = Style::new().size(24.0, 24.0);
/// # let _ = (toolbar, spacer, icon);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    /// Flex or grid.
    pub display: Display,
    /// Flow direction for flex containers.
    pub direction: Direction,
    /// Preferred width.
    pub width: Length,
    /// Preferred height.
    pub height: Length,
    /// Smallest allowed width.
    pub min_width: Length,
    /// Smallest allowed height.
    pub min_height: Length,
    /// Largest allowed width.
    pub max_width: Length,
    /// Largest allowed height.
    pub max_height: Length,
    /// Share of leftover main-axis space this box takes.
    pub grow: f64,
    /// Share of overflow this box gives up. Defaults to 1.
    pub shrink: f64,
    /// Space between children.
    pub gap: f64,
    /// Space inside the box, around its children.
    pub padding: Insets,
    /// Cross-axis placement of children.
    pub align: Align,
    /// Override of the parent's `align` for this box.
    pub align_self: Option<Align>,
    /// Main-axis distribution of leftover space.
    pub justify: Justify,
    /// How many grid columns this box spans (grid parents only).
    pub column_span: usize,
    /// Size of the content of a leaf (text, image...). Ignored for boxes
    /// with children, which size to their children.
    pub content: crate::Size,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            display: Display::Flex,
            direction: Direction::Column,
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            grow: 0.0,
            shrink: 1.0,
            gap: 0.0,
            padding: Insets::ZERO,
            align: Align::Stretch,
            align_self: None,
            justify: Justify::Start,
            column_span: 1,
            content: crate::Size::ZERO,
        }
    }
}

impl Style {
    /// A column (the default).
    pub fn new() -> Self {
        Style::default()
    }

    /// Children left to right.
    pub fn row() -> Self {
        Style {
            direction: Direction::Row,
            ..Style::default()
        }
    }

    /// Children top to bottom.
    pub fn column() -> Self {
        Style::default()
    }

    /// A grid with `columns` equal columns.
    pub fn grid(columns: usize) -> Self {
        Style {
            display: Display::Grid {
                columns: columns.max(1),
            },
            ..Style::default()
        }
    }

    /// Fixed width and height.
    pub fn size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.width = width.into();
        self.height = height.into();
        self
    }

    /// Fixed or relative width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Fixed or relative height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Minimum width and height.
    pub fn min_size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.min_width = width.into();
        self.min_height = height.into();
        self
    }

    /// Maximum width and height.
    pub fn max_size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.max_width = width.into();
        self.max_height = height.into();
        self
    }

    /// Take a share of leftover space.
    pub fn grow(mut self, grow: f64) -> Self {
        self.grow = grow.max(0.0);
        self
    }

    /// How much this box gives up when space runs out (0 = never shrink).
    pub fn shrink(mut self, shrink: f64) -> Self {
        self.shrink = shrink.max(0.0);
        self
    }

    /// Space between children.
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    /// Space inside the box.
    pub fn padding(mut self, padding: impl Into<Insets>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Cross-axis placement of children.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Cross-axis placement of this box, overriding the parent.
    pub fn align_self(mut self, align: Align) -> Self {
        self.align_self = Some(align);
        self
    }

    /// Main-axis distribution of children.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Centre children on both axes.
    pub fn center(self) -> Self {
        self.align(Align::Center).justify(Justify::Center)
    }

    /// Span `columns` grid columns.
    pub fn span(mut self, columns: usize) -> Self {
        self.column_span = columns.max(1);
        self
    }

    /// The natural size of a leaf's content, such as measured text.
    pub fn content(mut self, width: f64, height: f64) -> Self {
        self.content = crate::Size::new(width, height);
        self
    }
}
