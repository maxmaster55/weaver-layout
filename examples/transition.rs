//! Renders the README animation: a dashboard that re-flows between three
//! layouts, with every card springing to its new place.
//!
//! ```sh
//! cargo run --example transition --features motion -- /tmp/frames
//! convert -delay 3 -loop 0 /tmp/frames/*.svg docs/transition.gif
//! ```

use std::fmt::Write as _;
use std::{env, fs, path::PathBuf};

use weaver_layout::{AnimatedLayout, LayoutTree, Rect, Size, Style};

const W: f64 = 640.0;
const H: f64 = 360.0;
const COLORS: [&str; 6] = [
    "#6c8cff", "#ff8a65", "#4dd0a1", "#ffd166", "#b388ff", "#4fc3f7",
];

/// Three arrangements of the same six cards.
fn arrangement(step: usize) -> Vec<(usize, Rect)> {
    let mut tree = LayoutTree::new();
    let root = match step % 3 {
        // A 3-column grid; the first card spans two columns.
        0 => {
            let cards: Vec<_> = (0..6)
                .map(|i| {
                    let style = Style::new().height(if i == 0 { 150.0 } else { 140.0 });
                    tree.leaf(if i == 0 { style.span(2) } else { style })
                })
                .collect();
            tree.node(Style::grid(3).gap(16.0).padding(24.0), cards)
        }
        // A sidebar with a big content area.
        1 => {
            let side: Vec<_> = (1..6).map(|_| tree.leaf(Style::new().grow(1.0))).collect();
            let sidebar = tree.node(Style::column().width(150.0).gap(12.0), side);
            let main = tree.leaf(Style::new().grow(1.0));
            let main_id = main;
            let root = tree.node(Style::row().gap(16.0).padding(24.0), [main, sidebar]);
            // Card 0 is the content area, cards 1..6 are in the sidebar.
            let layout = tree.compute(root, Size::new(W, H));
            let mut out = vec![(0, layout[main_id])];
            for (i, &child) in tree.children(sidebar).iter().enumerate() {
                out.push((i + 1, layout[child]));
            }
            return out;
        }
        // A row of tall cards, centred.
        _ => {
            let cards: Vec<_> = (0..6)
                .map(|i| tree.leaf(Style::new().size(70.0, 120.0 + (i as f64 % 3.0) * 40.0)))
                .collect();
            tree.node(Style::row().gap(16.0).padding(24.0).center(), cards)
        }
    };
    let layout = tree.compute(root, Size::new(W, H));
    tree.children(root)
        .iter()
        .enumerate()
        .map(|(i, &c)| (i, layout[c]))
        .collect()
}

fn frame_svg(boxes: &AnimatedLayout<usize>) -> String {
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}"><rect width="100%" height="100%" fill="#0f1420"/>"##
    );
    let mut items: Vec<_> = boxes.iter().collect();
    items.sort_by_key(|(k, _)| **k);
    for (key, r) in items {
        let _ = write!(
            svg,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="14" fill="{}"/>"##,
            r.x,
            r.y,
            r.width.max(0.0),
            r.height.max(0.0),
            COLORS[*key]
        );
        let _ = write!(
            svg,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="8" rx="4" fill="#ffffff" opacity="0.55"/>"##,
            r.x + 14.0,
            r.y + 16.0,
            (r.width * 0.45).max(0.0),
        );
    }
    svg.push_str("</svg>");
    svg
}

fn main() {
    let out = PathBuf::from(env::args().nth(1).unwrap_or_else(|| "target/frames".into()));
    fs::create_dir_all(&out).expect("create output directory");
    let mut boxes = AnimatedLayout::new();
    boxes.set_targets(arrangement(0));
    let dt = 1.0 / 30.0;
    let mut frame = 0;
    for step in 1..=3 {
        boxes.set_targets(arrangement(step));
        // 1.2 s per transition, then hold.
        for _ in 0..36 {
            boxes.advance(dt);
            fs::write(out.join(format!("{frame:04}.svg")), frame_svg(&boxes)).unwrap();
            frame += 1;
        }
    }
    println!("wrote {frame} frames to {}", out.display());
}
