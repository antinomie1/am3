//! The loading indicator of Material 3 Expressive: a 38 dp shape that
//! morphs through seven Material shapes while it turns, for waits under
//! five seconds. The contained variant sits on a 48 dp primary-container
//! circle.
//!
//! Each shape is a polar radius function sampled at fixed angles, so a
//! morph interpolates radii and the outline is a smooth closed curve
//! through the samples. This approximates Material's rounded polygons.

use std::{
    f32::consts::{PI, TAU},
    time::Instant,
};

use aegle_layout::Style;
use aegle_ui::{
    Appearance, Container, ControlKind, Point, Result, Size, Theme, VisualState,
    control::{Control, MeasureCx, PaintCx},
    handle,
    scene::{FillRule, PathBuilder, Rect, RoundedRect},
};

use crate::Scheme;

const SAMPLES: usize = 48;
const SIZE: f32 = 48.0;
const SHAPE: f32 = 38.0;
/// Each morph takes this long, and the next starts at once.
const STEP: f32 = 0.65;

/// Material's loading shapes in order: soft burst, 9-sided cookie,
/// pentagon, pill, sunny, 4-sided cookie and oval, as radius at an angle.
fn radius(shape: usize, a: f32) -> f32 {
    let lobes = |n: f32, depth: f32| 1.0 - depth * (1.0 - (n * a).cos()) / 2.0;
    match shape % 7 {
        0 => 1.0 - 0.22 * ((1.0 - (10.0 * a).cos()) / 2.0).powf(0.6),
        1 => lobes(9.0, 0.16),
        2 => {
            // A pentagon, its corners rounded toward a circle.
            let side = TAU / 5.0;
            let d = (a.rem_euclid(side) - side / 2.0).abs();
            let flat = (PI / 5.0).cos() / d.cos();
            0.75 * flat + 0.25 * (PI / 5.0).cos() + 0.06
        }
        3 => {
            // A pill: a superellipse twice as wide as high.
            let (c, s) = (a.cos().abs(), a.sin().abs());
            (c.powi(4) + (s / 0.55).powi(4)).powf(-0.25)
        }
        4 => lobes(8.0, 0.12),
        5 => lobes(4.0, 0.24),
        _ => {
            let (c, s) = (a.cos(), a.sin() / 0.8);
            (c * c + s * s).powf(-0.5)
        }
    }
}

/// Samples of each shape, scaled so its farthest point is 1.
fn samples(shape: usize) -> [f32; SAMPLES] {
    let mut r = [0.0; SAMPLES];
    for (i, v) in r.iter_mut().enumerate() {
        *v = radius(shape, i as f32 / SAMPLES as f32 * TAU);
    }
    let max = r.iter().copied().fold(0.0, f32::max);
    r.map(|v| v / max)
}

/// The control inside a [`LoadingIndicator`].
pub struct LoadingControl {
    contained: bool,
    shapes: [[f32; SAMPLES]; 7],
    epoch: Option<Instant>,
}

impl Control for LoadingControl {
    fn kind(&self) -> &'static ControlKind {
        if self.contained {
            &CONTAINED_LOADING_INDICATOR
        } else {
            &LOADING_INDICATOR
        }
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(Size::new(SIZE, SIZE))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let a = *cx.appearance;
        let center = Point::new(cx.size.width / 2.0, cx.size.height / 2.0);
        if self.contained {
            let rect = Rect::new(center.x - SIZE / 2.0, center.y - SIZE / 2.0, SIZE, SIZE);
            cx.builder
                .fill(RoundedRect::new(rect, SIZE / 2.0)?, a.background)?;
        }
        // Reduced motion holds the first shape still.
        let t = if cx.reduced_motion {
            0.0
        } else {
            let epoch = *self.epoch.get_or_insert(cx.time);
            cx.time.saturating_duration_since(epoch).as_secs_f32()
        };
        let step = (t / STEP).floor();
        let u = t / STEP - step;
        // The morph follows an expressive spring's overshoot, approximated.
        let k = 1.0 - (-6.0 * u).exp() * (8.0 * u).cos();
        let (from, to) = (
            &self.shapes[step as usize % 7],
            &self.shapes[(step as usize + 1) % 7],
        );
        // Each morph turns a quarter on top of a slow steady turn.
        let turn = (step + k) * TAU / 4.0 + t * TAU / 4.666;
        let scale = SHAPE / 2.0 * if self.contained { 0.84 } else { 1.0 };
        let points: Vec<Point> = (0..SAMPLES)
            .map(|i| {
                let r = (from[i] + (to[i] - from[i]) * k) * scale;
                let angle = i as f32 / SAMPLES as f32 * TAU + turn;
                Point::new(center.x + r * angle.cos(), center.y + r * angle.sin())
            })
            .collect();
        // A closed Catmull-Rom curve through the samples, as cubics.
        let mut path = PathBuilder::new();
        path.move_to(points[0]);
        for i in 0..SAMPLES {
            let p = |j: usize| points[(i + j + SAMPLES - 1) % SAMPLES];
            let (p0, p1, p2, p3) = (p(0), p(1), p(2), p(3));
            let c1 = Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
            let c2 = Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);
            path.cubic_to(c1, c2, p2);
        }
        path.close();
        cx.builder
            .fill_path(&path.finish(FillRule::NonZero)?, a.indicator)?;
        if !cx.reduced_motion {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        cx.node
            .set_role(aegle_ui::accesskit::Role::ProgressIndicator);
    }
}

fn plain(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        indicator: Scheme::of(theme).primary,
        ..Appearance::base(theme, state)
    }
}

fn contained(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    Appearance {
        background: s.primary_container,
        indicator: s.on_primary_container,
        ..Appearance::base(theme, state)
    }
}

/// A loading indicator.
pub static LOADING_INDICATOR: ControlKind = ControlKind {
    name: "LoadingIndicator",
    skin: plain,
    accepts: aegle_ui::Accepts::INDICATOR,
    container: false,
};
/// A loading indicator on a container.
pub static CONTAINED_LOADING_INDICATOR: ControlKind = ControlKind {
    name: "ContainedLoadingIndicator",
    skin: contained,
    accepts: aegle_ui::Accepts::INDICATOR,
    container: false,
};

handle! {
    /// A morphing loading indicator, for short indeterminate waits.
    pub LoadingIndicator(LoadingControl): indicator
}

impl LoadingIndicator {
    /// Appends a loading indicator, on a primary-container circle if
    /// `contained`.
    pub fn new(parent: &Container, contained: bool) -> Result<Self> {
        let node = parent.add(|_, _| {
            let control = LoadingControl {
                contained,
                shapes: std::array::from_fn(samples),
                epoch: None,
            };
            let style = Style {
                flex_shrink: 0.0,
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn Control>, style))
        })?;
        Ok(Self(node))
    }
}
