use weaver_layout::*;

fn approx(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn columns_stack_children_and_stretch_them() {
    let mut t = LayoutTree::new();
    let a = t.leaf(Style::new().height(10.0));
    let b = t.leaf(Style::new().height(20.0));
    let root = t.node(Style::column().gap(5.0).padding(10.0), [a, b]);
    let l = t.compute(root, Size::new(100.0, 100.0));
    assert_eq!(l[a], Rect::new(10.0, 10.0, 80.0, 10.0));
    assert_eq!(l[b], Rect::new(10.0, 25.0, 80.0, 20.0));
}

#[test]
fn grow_shares_free_space_by_weight() {
    let mut t = LayoutTree::new();
    let a = t.leaf(Style::new().grow(1.0));
    let b = t.leaf(Style::new().grow(3.0));
    let root = t.node(Style::row(), [a, b]);
    let l = t.compute(root, Size::new(400.0, 50.0));
    approx(l[a].width, 100.0);
    approx(l[b].width, 300.0);
    approx(l[b].x, 100.0);
}

#[test]
fn overflow_shrinks_children_proportionally_and_respects_min() {
    let mut t = LayoutTree::new();
    let a = t.leaf(Style::new().width(300.0));
    let b = t.leaf(Style::new().width(100.0).min_size(100.0, Length::Auto));
    let fixed = t.leaf(Style::new().width(100.0).shrink(0.0));
    let root = t.node(Style::row(), [a, b, fixed]);
    let l = t.compute(root, Size::new(300.0, 10.0));
    approx(l[fixed].width, 100.0);
    approx(l[b].width, 100.0);
    assert!(l[a].width < 300.0);
}

#[test]
fn justify_modes_place_children() {
    let cases = [
        (Justify::Start, [0.0, 20.0]),
        (Justify::Center, [30.0, 50.0]),
        (Justify::End, [60.0, 80.0]),
        (Justify::SpaceBetween, [0.0, 80.0]),
        (Justify::SpaceAround, [15.0, 65.0]),
        (Justify::SpaceEvenly, [20.0, 60.0]),
    ];
    for (justify, expected) in cases {
        let mut t = LayoutTree::new();
        let a = t.leaf(Style::new().size(20.0, 20.0));
        let b = t.leaf(Style::new().size(20.0, 20.0));
        let root = t.node(Style::row().justify(justify), [a, b]);
        let l = t.compute(root, Size::new(100.0, 20.0));
        approx(l[a].x, expected[0]);
        approx(l[b].x, expected[1]);
    }
}

#[test]
fn align_places_on_the_cross_axis() {
    let mut t = LayoutTree::new();
    let start = t.leaf(Style::new().size(10.0, 10.0).align_self(Align::Start));
    let center = t.leaf(Style::new().size(10.0, 10.0).align_self(Align::Center));
    let end = t.leaf(Style::new().size(10.0, 10.0).align_self(Align::End));
    let root = t.node(Style::row(), [start, center, end]);
    let l = t.compute(root, Size::new(100.0, 50.0));
    approx(l[start].y, 0.0);
    approx(l[center].y, 20.0);
    approx(l[end].y, 40.0);
}

#[test]
fn percent_lengths_resolve_against_the_parent() {
    let mut t = LayoutTree::new();
    let half = t.leaf(Style::new().width(Length::Percent(50.0)));
    let root = t.node(Style::row().padding(10.0), [half]);
    let l = t.compute(root, Size::new(220.0, 100.0));
    approx(l[half].width, 100.0);
}

#[test]
fn auto_containers_size_to_their_children() {
    let mut t = LayoutTree::new();
    let a = t.leaf(Style::new().content(40.0, 10.0));
    let b = t.leaf(Style::new().content(60.0, 30.0));
    let chip = t.node(
        Style::row().gap(4.0).padding(Insets::symmetric(2.0, 8.0)),
        [a, b],
    );
    let root = t.node(Style::column().align(Align::Start), [chip]);
    let l = t.compute(root, Size::new(500.0, 500.0));
    approx(l[chip].width, 8.0 + 40.0 + 4.0 + 60.0 + 8.0);
    approx(l[chip].height, 2.0 + 30.0 + 2.0);
}

#[test]
fn grids_fill_rows_and_honour_spans() {
    let mut t = LayoutTree::new();
    let wide = t.leaf(Style::new().height(50.0).span(2));
    let a = t.leaf(Style::new().height(30.0));
    let b = t.leaf(Style::new().height(40.0));
    let root = t.node(Style::grid(2).gap(10.0), [wide, a, b]);
    let l = t.compute(root, Size::new(210.0, 500.0));
    assert_eq!(l[wide], Rect::new(0.0, 0.0, 210.0, 50.0));
    assert_eq!(l[a], Rect::new(0.0, 60.0, 100.0, 30.0));
    assert_eq!(l[b], Rect::new(110.0, 60.0, 100.0, 40.0));
}

#[test]
fn hit_testing_finds_the_deepest_box() {
    let mut t = LayoutTree::new();
    let button = t.leaf(Style::new().size(50.0, 20.0));
    let bar = t.node(Style::row().padding(10.0), [button]);
    let l = t.compute(bar, Size::new(200.0, 40.0));
    assert_eq!(l.hit_test(&t, bar, 15.0, 15.0), Some(button));
    assert_eq!(l.hit_test(&t, bar, 150.0, 15.0), Some(bar));
    assert_eq!(l.hit_test(&t, bar, 500.0, 15.0), None);
}

#[cfg(feature = "motion")]
#[test]
fn animated_layout_settles_exactly_and_reduced_motion_snaps() {
    let mut boxes = AnimatedLayout::new();
    boxes.set_targets([(1, Rect::new(0.0, 0.0, 10.0, 10.0))]);
    assert!(boxes.is_settled());
    boxes.set_targets([(1, Rect::new(100.0, 50.0, 20.0, 20.0))]);
    let mut frames = 0;
    while boxes.advance(1.0 / 60.0) {
        frames += 1;
        assert!(frames < 600, "never settled");
    }
    assert_eq!(boxes.rect(&1), Some(Rect::new(100.0, 50.0, 20.0, 20.0)));

    boxes.set_reduced_motion(true);
    boxes.set_targets([
        (1, Rect::new(0.0, 0.0, 10.0, 10.0)),
        (2, Rect::new(5.0, 5.0, 1.0, 1.0)),
    ]);
    assert!(boxes.is_settled());
    assert_eq!(boxes.rect(&1), Some(Rect::new(0.0, 0.0, 10.0, 10.0)));
}
