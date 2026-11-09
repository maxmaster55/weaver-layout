<div align="center">

# weaver-layout

### Flexbox and grid layout for Rust, with layout changes that move.

Rows, columns, grids, gaps, padding, grow and shrink, alignment, min/max.
<br>
No dependencies, no renderer, works with any drawing library.

</div>

<p align="center">
  <img src="docs/transition.gif" width="640" alt="Six cards re-flow from a grid to a sidebar layout to a centred row, each card springing to its new position">
</p>
<p align="center"><sub>Three layouts of the same six cards, computed by <code>LayoutTree</code> and animated by <code>AnimatedLayout</code>. Regenerate with the command at the bottom.</sub></p>

## Quick start

```toml
[dependencies]
weaver-layout = { git = "https://github.com/maxmaster55/weaver-layout" }
```

```rust
use weaver_layout::{Align, LayoutTree, Size, Style};

let mut tree = LayoutTree::new();
let logo   = tree.leaf(Style::new().size(32.0, 32.0));
let title  = tree.leaf(Style::new().content(120.0, 20.0).grow(1.0));
let avatar = tree.leaf(Style::new().size(32.0, 32.0));
let bar = tree.node(
    Style::row().height(56.0).padding(12.0).gap(12.0).align(Align::Center),
    [logo, title, avatar],
);

let layout = tree.compute(bar, Size::new(400.0, 800.0));
println!("{:?}", layout[title]); // Rect { x: 56.0, y: 18.0, width: 276.0, height: 20.0 }
```

You describe boxes with a `Style`, build a tree, and call `compute` with the
space you have. You get back one `Rect` per box. Draw them however you like.

## What a `Style` can say

| Method | Meaning |
| --- | --- |
| `Style::row()` / `Style::column()` | Children flow left-to-right or top-to-bottom |
| `Style::grid(n)` | `n` equal columns, filled row by row; children can `.span(k)` |
| `.size(w, h)`, `.width(..)`, `.height(..)` | Fixed sizes: `200.0` or `Length::Percent(50.0)` |
| `.min_size(..)`, `.max_size(..)` | Bounds that grow and shrink respect |
| `.grow(1.0)` | Take a share of the leftover space |
| `.shrink(0.0)` | Never give up space when it runs out (default is `1.0`) |
| `.gap(8.0)`, `.padding(16.0)` | Space between children, and inside the box (`Insets::symmetric(v, h)` too) |
| `.align(Align::Center)` | Cross-axis placement: `Stretch` (default), `Start`, `Center`, `End` |
| `.align_self(..)` | Override the parent's alignment for one child |
| `.justify(Justify::SpaceBetween)` | Main-axis spacing: `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly` |
| `.center()` | Centre children on both axes |
| `.content(w, h)` | The natural size of a leaf, such as measured text |

Containers without a fixed size shrink-wrap their children, and
`layout.hit_test(&tree, root, x, y)` finds the deepest box under a point.

## Animated layout changes

Turn on the `motion` feature and let boxes travel to their new places with
springs from the [Weave motion engine](https://github.com/maxmaster55/weave-motion):

```toml
weaver-layout = { git = "https://github.com/maxmaster55/weaver-layout", features = ["motion"] }
```

```rust
use weaver_layout::{AnimatedLayout, Rect};

let mut boxes = AnimatedLayout::new();          // keyed by anything: ids, enums, NodeIds
boxes.set_targets([("card", Rect::new(0.0, 0.0, 100.0, 50.0))]);

// Later, the layout changed:
boxes.set_targets([("card", Rect::new(200.0, 0.0, 100.0, 50.0))]);

// Every frame:
let still_moving = boxes.advance(1.0 / 60.0);
let draw_here = boxes.rect(&"card").unwrap();
```

Changing the targets mid-flight keeps each box's velocity, so rapid layout
changes never jerk. `set_reduced_motion(true)` snaps boxes straight to their
targets for users who ask for less motion.

## Part of Weaver

`weaver-layout` is one of the [Weaver](https://github.com/maxmaster55) libraries.
[weaver-genui](https://github.com/maxmaster55/weaver_ui) uses it to lay out
generated UI documents, but it has no dependency on any of them.

## Regenerate the animation

```sh
cargo run --example transition --features motion -- /tmp/frames
convert -delay 3 -loop 0 /tmp/frames/*.svg -layers Optimize docs/transition.gif
```

ImageMagick needs an SVG delegate such as `rsvg-convert`.

## License

MIT or Apache-2.0, at your option.
