//! # weaver-layout
//!
//! A small layout engine: flexbox-style rows and columns, grids, padding,
//! gaps, grow and shrink, alignment and min/max sizes. No dependencies, no
//! renderer, no allocation per frame beyond the result.
//!
//! With the `motion` feature, [`AnimatedLayout`] moves boxes to their new
//! places with springs from the Weave motion engine whenever the layout
//! changes.
//!
//! ```
//! use weaver_layout::{Align, LayoutTree, Size, Style};
//!
//! let mut tree = LayoutTree::new();
//! let logo = tree.leaf(Style::new().size(32.0, 32.0));
//! let title = tree.leaf(Style::new().content(120.0, 20.0).grow(1.0));
//! let avatar = tree.leaf(Style::new().size(32.0, 32.0));
//! let bar = tree.node(
//!     Style::row().height(56.0).padding(12.0).gap(12.0).align(Align::Center),
//!     [logo, title, avatar],
//! );
//!
//! let layout = tree.compute(bar, Size::new(400.0, 800.0));
//! assert_eq!(layout[avatar].x, 400.0 - 12.0 - 32.0);
//! assert_eq!(layout[title].y, 18.0); // centred in the 32px inner height
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "motion")]
mod animate;
mod geometry;
mod style;
mod tree;

#[cfg(feature = "motion")]
pub use animate::AnimatedLayout;
pub use geometry::{Insets, Rect, Size};
pub use style::{Align, Direction, Display, Justify, Length, Style};
pub use tree::{Layout, LayoutTree, NodeId};
