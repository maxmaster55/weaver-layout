//! Spring-animated layout transitions (the `motion` feature).
//!
//! When a layout changes, boxes should *move* to their new place instead of
//! jumping. [`AnimatedLayout`] keeps one spring per box, keyed by anything
//! stable (an id string, an enum, a [`NodeId`](crate::NodeId)). Give it the
//! new rectangles whenever the layout changes and advance it every frame.
//! Interrupting a transition keeps each box's velocity, so rapid changes
//! stay smooth.
//!
//! ```
//! use weaver_layout::{AnimatedLayout, Rect};
//!
//! let mut boxes = AnimatedLayout::new();
//! boxes.set_targets([("card", Rect::new(0.0, 0.0, 100.0, 50.0))]);
//! boxes.set_targets([("card", Rect::new(200.0, 0.0, 100.0, 50.0))]);
//!
//! boxes.advance(1.0 / 60.0);
//! let moving = boxes.rect(&"card").unwrap();
//! assert!(moving.x > 0.0 && moving.x < 200.0);
//!
//! while boxes.advance(1.0 / 60.0) {}
//! assert_eq!(boxes.rect(&"card").unwrap().x, 200.0);
//! ```

use std::collections::HashMap;
use std::hash::Hash;

use weave_core::{Animated, MotionToken, SpringConfig, Vec4};

use crate::Rect;

fn to_vec4(r: Rect) -> Vec4 {
    Vec4 {
        x: r.x,
        y: r.y,
        z: r.width,
        w: r.height,
    }
}

fn to_rect(v: Vec4) -> Rect {
    Rect::new(v.x, v.y, v.z, v.w)
}

/// Rectangles that follow their targets with springs.
#[derive(Clone, Debug)]
pub struct AnimatedLayout<K> {
    boxes: HashMap<K, Animated<Vec4>>,
    spring: SpringConfig,
    reduced_motion: bool,
}

impl<K: Eq + Hash + Clone> Default for AnimatedLayout<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone> AnimatedLayout<K> {
    /// Uses Weave's default spatial spring.
    pub fn new() -> Self {
        Self::with_spring(
            MotionToken::DefaultSpatial
                .spring_config()
                .expect("spatial tokens are springs"),
        )
    }

    /// Uses your own spring.
    pub fn with_spring(spring: SpringConfig) -> Self {
        AnimatedLayout {
            boxes: HashMap::new(),
            spring,
            reduced_motion: false,
        }
    }

    /// When on, boxes jump straight to their targets (for users who ask
    /// for reduced motion).
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.reduced_motion = reduced;
        if reduced {
            for value in self.boxes.values_mut() {
                value.settle();
            }
        }
    }

    /// Set where every box should be. New keys appear at their target,
    /// keys that are missing are dropped, and moving boxes keep their
    /// velocity.
    pub fn set_targets(&mut self, targets: impl IntoIterator<Item = (K, Rect)>) {
        let mut next = HashMap::new();
        for (key, rect) in targets {
            let target = to_vec4(rect);
            let value = match self.boxes.remove(&key) {
                Some(mut value) => {
                    value.set_target(target);
                    if self.reduced_motion {
                        value.settle();
                    }
                    value
                }
                None => Animated::resting(target, self.spring),
            };
            next.insert(key, value);
        }
        self.boxes = next;
    }

    /// Start a box from `from` (for example where it was before a
    /// re-parenting) and let it travel to its current target.
    pub fn start_from(&mut self, key: &K, from: Rect) {
        if self.reduced_motion {
            return;
        }
        if let Some(value) = self.boxes.get_mut(key) {
            let target = value.target();
            *value = Animated::new(to_vec4(from), target, self.spring);
        }
    }

    /// Advance every spring by `dt` seconds. Returns whether anything is
    /// still moving, so you know whether to request another frame.
    pub fn advance(&mut self, dt: f64) -> bool {
        let mut active = false;
        for value in self.boxes.values_mut() {
            active |= value.advance(dt);
        }
        active
    }

    /// Where a box is drawn right now.
    pub fn rect(&self, key: &K) -> Option<Rect> {
        self.boxes.get(key).map(|v| to_rect(v.value()))
    }

    /// Where a box is heading.
    pub fn target(&self, key: &K) -> Option<Rect> {
        self.boxes.get(key).map(|v| to_rect(v.target()))
    }

    /// Every box and where it is drawn now.
    pub fn iter(&self) -> impl Iterator<Item = (&K, Rect)> {
        self.boxes.iter().map(|(k, v)| (k, to_rect(v.value())))
    }

    /// Whether every box has arrived.
    pub fn is_settled(&self) -> bool {
        self.boxes.values().all(Animated::is_settled)
    }
}
