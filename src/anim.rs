//! The library's own animations: a scalar following a spring, and the press
//! ripple. Both sample the frame time and report whether they still move, so
//! a control requests frames only while something animates.

use std::time::{Duration, Instant};

use aegle_motion::{Easing, Tween};
use aegle_ui::{
    Color, Point, Rect, Result, Size,
    scene::{RoundedRect, SceneBuilder},
};

use crate::{color::alpha, tokens::Spring};

/// A value that eases to each new target along a spring.
#[derive(Debug, Default)]
pub(crate) struct Value {
    current: Option<f32>,
    tween: Option<(Instant, Tween<f32>)>,
}

impl Value {
    /// The value at `now` on its way to `target`; the first target and
    /// reduced motion snap. Also whether it still moves.
    pub fn at(&mut self, target: f32, spring: Spring, now: Instant, reduced: bool) -> (f32, bool) {
        let mut value = self.current.unwrap_or(target);
        if let Some((start, tween)) = &self.tween {
            let elapsed = now.saturating_duration_since(*start);
            value = tween.sample(elapsed);
            if tween.finished(elapsed) {
                self.tween = None;
            }
        }
        let aiming = self.tween.as_ref().map_or(value, |(_, t)| t.target());
        if aiming != target {
            self.tween = None;
            if self.current.is_some() && !reduced {
                let spring = spring.aegle();
                let tween = Tween::new(value, target, spring.duration(), Easing::Spring(spring));
                self.tween = Some((now, tween.expect("finite spring tween")));
            } else {
                value = target;
            }
        }
        self.current = Some(value);
        (value, self.tween.is_some())
    }
}

/// The ripple of the latest press: it grows from the press point until it
/// covers the container, then fades once released.
#[derive(Debug)]
pub(crate) struct Ripple {
    origin: Point,
    start: Instant,
    released: Option<Instant>,
}

const GROW: Duration = Duration::from_millis(350);
const FADE: Duration = Duration::from_millis(200);

impl Ripple {
    pub fn new(origin: Point, start: Instant) -> Self {
        Self {
            origin,
            start,
            released: None,
        }
    }

    pub fn release(&mut self, at: Instant) {
        self.released.get_or_insert(at);
    }

    /// Paints the ripple in `color` at the pressed state-layer opacity,
    /// inside the caller's clip; returns whether it is still visible.
    pub fn paint(
        &self,
        builder: &mut SceneBuilder,
        size: Size,
        color: Color,
        now: Instant,
    ) -> Result<bool> {
        let grown = now.saturating_duration_since(self.start).as_secs_f32() / GROW.as_secs_f32();
        let grow = 1.0 - (1.0 - grown.min(1.0)).powi(3);
        // Fading starts at release, but never before the ripple has grown.
        let fade = self.released.map_or(0.0, |at| {
            let from = at.max(self.start + GROW);
            now.saturating_duration_since(from).as_secs_f32() / FADE.as_secs_f32()
        });
        if fade >= 1.0 {
            return Ok(false);
        }
        let far = |edge: f32, at: f32| (edge - at).abs().max(at);
        let reach = far(size.width, self.origin.x).hypot(far(size.height, self.origin.y));
        let r = reach * (0.2 + 0.8 * grow);
        let disc = Rect::new(self.origin.x - r, self.origin.y - r, 2.0 * r, 2.0 * r);
        let opacity = crate::tokens::state::PRESSED * (1.0 - fade);
        builder.fill(RoundedRect::new(disc, r)?, alpha(color, opacity))?;
        Ok(true)
    }
}
