//! The layout tree and the layout algorithm.

use crate::{Align, Direction, Display, Justify, Length, Rect, Size, Style};

/// A handle to a box in a [`LayoutTree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    /// The index of the box in creation order.
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
struct Entry {
    style: Style,
    children: Vec<NodeId>,
}

/// A tree of boxes to lay out.
///
/// ```
/// use weaver_layout::{LayoutTree, Size, Style};
///
/// let mut tree = LayoutTree::new();
/// let sidebar = tree.leaf(Style::new().width(200.0));
/// let content = tree.leaf(Style::new().grow(1.0));
/// let app = tree.node(Style::row().gap(16.0).padding(16.0), [sidebar, content]);
///
/// let layout = tree.compute(app, Size::new(1000.0, 600.0));
/// assert_eq!(layout[sidebar].width, 200.0);
/// assert_eq!(layout[content].x, 232.0);
/// assert_eq!(layout[content].width, 752.0);
/// assert_eq!(layout[content].height, 568.0); // stretched
/// ```
#[derive(Clone, Debug, Default)]
pub struct LayoutTree {
    entries: Vec<Entry>,
}

impl LayoutTree {
    /// An empty tree.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a box with no children.
    pub fn leaf(&mut self, style: Style) -> NodeId {
        self.node(style, [])
    }

    /// Add a box with children. Children must already be in the tree.
    pub fn node(&mut self, style: Style, children: impl IntoIterator<Item = NodeId>) -> NodeId {
        self.entries.push(Entry {
            style,
            children: children.into_iter().collect(),
        });
        NodeId(self.entries.len() - 1)
    }

    /// Append a child to an existing box.
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.entries[parent.0].children.push(child);
    }

    /// The style of a box.
    pub fn style(&self, id: NodeId) -> &Style {
        &self.entries[id.0].style
    }

    /// Change the style of a box.
    pub fn set_style(&mut self, id: NodeId, style: Style) {
        self.entries[id.0].style = style;
    }

    /// The children of a box.
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        &self.entries[id.0].children
    }

    /// Number of boxes in the tree.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the tree has no boxes.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Lay out `root` and everything under it inside `available` space.
    ///
    /// The root takes its own style's size if it has one, otherwise the
    /// whole available space.
    pub fn compute(&self, root: NodeId, available: Size) -> Layout {
        let mut rects = vec![Rect::default(); self.entries.len()];
        let style = &self.entries[root.0].style;
        let width = clamp_axis(
            style
                .width
                .resolve(available.width)
                .unwrap_or(available.width),
            style.min_width,
            style.max_width,
            available.width,
        );
        let height = clamp_axis(
            style
                .height
                .resolve(available.height)
                .unwrap_or(available.height),
            style.min_height,
            style.max_height,
            available.height,
        );
        self.place(root, Rect::new(0.0, 0.0, width, height), &mut rects);
        Layout { rects }
    }

    /// The size a box would like to be, given the space around it.
    pub fn measure(&self, id: NodeId, available: Size) -> Size {
        let entry = &self.entries[id.0];
        let style = &entry.style;
        let fixed_w = style.width.resolve(available.width);
        let fixed_h = style.height.resolve(available.height);
        let content = if fixed_w.is_some() && fixed_h.is_some() {
            Size::ZERO
        } else if entry.children.is_empty() {
            style.content
        } else {
            let inner = Size::new(
                (fixed_w.unwrap_or(available.width) - style.padding.horizontal()).max(0.0),
                (fixed_h.unwrap_or(available.height) - style.padding.vertical()).max(0.0),
            );
            let natural = self.measure_children(entry, inner);
            Size::new(
                natural.width + style.padding.horizontal(),
                natural.height + style.padding.vertical(),
            )
        };
        Size::new(
            clamp_axis(
                fixed_w.unwrap_or(content.width),
                style.min_width,
                style.max_width,
                available.width,
            ),
            clamp_axis(
                fixed_h.unwrap_or(content.height),
                style.min_height,
                style.max_height,
                available.height,
            ),
        )
    }

    fn measure_children(&self, entry: &Entry, inner: Size) -> Size {
        let style = &entry.style;
        match style.display {
            Display::Flex => {
                let row = style.direction == Direction::Row;
                let mut main = 0.0;
                let mut cross: f64 = 0.0;
                for &child in &entry.children {
                    let size = self.measure(child, inner);
                    main += size.main(row);
                    cross = cross.max(size.cross(row));
                }
                main += style.gap * entry.children.len().saturating_sub(1) as f64;
                Size::from_axes(main, cross, row)
            }
            Display::Grid { columns } => {
                let columns = columns.max(1);
                let cell_w =
                    (inner.width - style.gap * (columns - 1) as f64).max(0.0) / columns as f64;
                let rows = grid_rows(self, entry, columns);
                let mut widest: f64 = 0.0;
                let mut height = 0.0;
                for row in &rows {
                    let mut row_h: f64 = 0.0;
                    for &(child, _, span) in row {
                        let span_w = cell_w * span as f64 + style.gap * (span - 1) as f64;
                        let size = self.measure(child, Size::new(span_w, inner.height));
                        row_h = row_h.max(size.height);
                        widest = widest.max(size.width / span as f64);
                    }
                    height += row_h;
                }
                height += style.gap * rows.len().saturating_sub(1) as f64;
                let width = widest * columns as f64 + style.gap * (columns - 1) as f64;
                Size::new(width, height)
            }
        }
    }

    fn place(&self, id: NodeId, rect: Rect, rects: &mut [Rect]) {
        rects[id.0] = rect;
        let entry = &self.entries[id.0];
        if entry.children.is_empty() {
            return;
        }
        let inner = rect.inset(entry.style.padding);
        match entry.style.display {
            Display::Flex => self.place_flex(entry, inner, rects),
            Display::Grid { columns } => self.place_grid(entry, columns.max(1), inner, rects),
        }
    }

    fn place_flex(&self, entry: &Entry, inner: Rect, rects: &mut [Rect]) {
        let style = &entry.style;
        let row = style.direction == Direction::Row;
        let inner_size = inner.size();
        let inner_main = inner_size.main(row);
        let inner_cross = inner_size.cross(row);
        let count = entry.children.len();
        let gaps = style.gap * count.saturating_sub(1) as f64;

        // 1. Base sizes along the main axis.
        let mut mains: Vec<f64> = entry
            .children
            .iter()
            .map(|&child| self.measure(child, inner_size).main(row))
            .collect();

        // 2. Grow into free space, or shrink out of overflow.
        let free = inner_main - mains.iter().sum::<f64>() - gaps;
        let styles: Vec<&Style> = entry
            .children
            .iter()
            .map(|c| &self.entries[c.0].style)
            .collect();
        if free > 0.0 {
            let total: f64 = styles.iter().map(|s| s.grow).sum();
            if total > 0.0 {
                for (main, s) in mains.iter_mut().zip(&styles) {
                    *main += free * s.grow / total;
                    *main = clamp_axis(*main, min_main(s, row), max_main(s, row), inner_main);
                }
            }
        } else if free < 0.0 {
            let total: f64 = mains.iter().zip(&styles).map(|(m, s)| m * s.shrink).sum();
            if total > 0.0 {
                for (main, s) in mains.iter_mut().zip(&styles) {
                    let loss = -free * (*main * s.shrink) / total;
                    let min = min_main(s, row).resolve(inner_main).unwrap_or(0.0);
                    *main = (*main - loss).max(min).max(0.0);
                }
            }
        }

        // 3. Distribute what is left.
        let remaining = (inner_main - mains.iter().sum::<f64>() - gaps).max(0.0);
        let (mut cursor, spacing) = justify(style.justify, remaining, count, style.gap);

        // 4. Cross axis and placement.
        for ((&child, main), s) in entry.children.iter().zip(&mains).zip(&styles) {
            let align = s.align_self.unwrap_or(style.align);
            let fixed_cross = if row { s.height } else { s.width }.resolve(inner_cross);
            let cross = match (fixed_cross, align) {
                (Some(c), _) => c,
                (None, Align::Stretch) => inner_cross,
                (None, _) => self
                    .measure(child, Size::from_axes(*main, inner_cross, row))
                    .cross(row),
            };
            let cross = clamp_axis(cross, min_cross(s, row), max_cross(s, row), inner_cross);
            let cross_pos = match align {
                Align::Start | Align::Stretch => 0.0,
                Align::Center => (inner_cross - cross) / 2.0,
                Align::End => inner_cross - cross,
            };
            let rect = if row {
                Rect::new(inner.x + cursor, inner.y + cross_pos, *main, cross)
            } else {
                Rect::new(inner.x + cross_pos, inner.y + cursor, cross, *main)
            };
            self.place(child, rect, rects);
            cursor += main + spacing;
        }
    }

    fn place_grid(&self, entry: &Entry, columns: usize, inner: Rect, rects: &mut [Rect]) {
        let style = &entry.style;
        let cell_w = (inner.width - style.gap * (columns - 1) as f64).max(0.0) / columns as f64;
        let mut y = inner.y;
        for row in grid_rows(self, entry, columns) {
            let spans: Vec<(NodeId, usize, usize, f64)> = row
                .iter()
                .map(|&(child, col, span)| {
                    (
                        child,
                        col,
                        span,
                        cell_w * span as f64 + style.gap * (span - 1) as f64,
                    )
                })
                .collect();
            let row_h = spans
                .iter()
                .map(|&(child, _, _, w)| self.measure(child, Size::new(w, inner.height)).height)
                .fold(0.0, f64::max);
            for (child, col, _, w) in spans {
                let s = &self.entries[child.0].style;
                let h = s.height.resolve(inner.height).unwrap_or(row_h);
                let x = inner.x + col as f64 * (cell_w + style.gap);
                self.place(child, Rect::new(x, y, w, h), rects);
            }
            y += row_h + style.gap;
        }
    }
}

/// Children grouped into grid rows as `(child, column, span)`.
fn grid_rows(tree: &LayoutTree, entry: &Entry, columns: usize) -> Vec<Vec<(NodeId, usize, usize)>> {
    let mut rows: Vec<Vec<(NodeId, usize, usize)>> = Vec::new();
    let mut col = 0;
    for &child in &entry.children {
        let span = tree.entries[child.0].style.column_span.clamp(1, columns);
        if col + span > columns || rows.is_empty() {
            rows.push(Vec::new());
            col = 0;
        }
        rows.last_mut()
            .expect("pushed above")
            .push((child, col, span));
        col += span;
    }
    rows
}

fn justify(justify: Justify, remaining: f64, count: usize, gap: f64) -> (f64, f64) {
    let n = count as f64;
    match justify {
        Justify::Start => (0.0, gap),
        Justify::Center => (remaining / 2.0, gap),
        Justify::End => (remaining, gap),
        Justify::SpaceBetween if count > 1 => (0.0, gap + remaining / (n - 1.0)),
        Justify::SpaceBetween => (0.0, gap),
        Justify::SpaceAround => (remaining / n / 2.0, gap + remaining / n),
        Justify::SpaceEvenly => (remaining / (n + 1.0), gap + remaining / (n + 1.0)),
    }
}

fn clamp_axis(value: f64, min: Length, max: Length, parent: f64) -> f64 {
    let mut value = value;
    if let Some(max) = max.resolve(parent) {
        value = value.min(max);
    }
    if let Some(min) = min.resolve(parent) {
        value = value.max(min);
    }
    value.max(0.0)
}

fn min_main(s: &Style, row: bool) -> Length {
    if row {
        s.min_width
    } else {
        s.min_height
    }
}

fn max_main(s: &Style, row: bool) -> Length {
    if row {
        s.max_width
    } else {
        s.max_height
    }
}

fn min_cross(s: &Style, row: bool) -> Length {
    min_main(s, !row)
}

fn max_cross(s: &Style, row: bool) -> Length {
    max_main(s, !row)
}

/// The result of [`LayoutTree::compute`]: one rectangle per box, in the
/// root's coordinate space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    rects: Vec<Rect>,
}

impl Layout {
    /// The rectangle of a box.
    pub fn rect(&self, id: NodeId) -> Rect {
        self.rects[id.0]
    }

    /// Every rectangle, indexed by [`NodeId::index`].
    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    /// The top-most box under a point, searching from `root`.
    pub fn hit_test(&self, tree: &LayoutTree, root: NodeId, x: f64, y: f64) -> Option<NodeId> {
        if !self.rect(root).contains(x, y) {
            return None;
        }
        for &child in tree.children(root).iter().rev() {
            if let Some(hit) = self.hit_test(tree, child, x, y) {
                return Some(hit);
            }
        }
        Some(root)
    }
}

impl std::ops::Index<NodeId> for Layout {
    type Output = Rect;

    fn index(&self, id: NodeId) -> &Rect {
        &self.rects[id.0]
    }
}
