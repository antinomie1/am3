//! Round-capped strokes of progress indicators: straight or wavy spans of a
//! line, and flat or wavy arcs of a circle.

use std::f32::consts::TAU;

use aegle_ui::{
    Color, Point, Result,
    scene::{FillRule, LineCap, LineJoin, PathBuilder, SceneBuilder, Stroke},
};

/// A wave across the stroke: its amplitude, wavelength and phase in
/// logical pixels.
#[derive(Clone, Copy, Debug)]
pub(super) struct Wave {
    pub amplitude: f32,
    pub length: f32,
    pub phase: f32,
}

fn round(width: f32) -> Stroke {
    Stroke {
        width,
        cap: LineCap::Round,
        join: LineJoin::Round,
    }
}

/// Strokes the line from `x0` to `x1` at height `y`, waving if `wave` has
/// an amplitude. Spans shorter than nothing draw nothing.
pub(super) fn line(
    builder: &mut SceneBuilder,
    (x0, x1): (f32, f32),
    y: f32,
    width: f32,
    wave: Option<Wave>,
    color: Color,
) -> Result {
    if x1 < x0 {
        return Ok(());
    }
    let mut path = PathBuilder::new();
    match wave.filter(|w| w.amplitude > 0.01) {
        None => {
            path.move_to(Point::new(x0, y)).line_to(Point::new(x1, y));
        }
        Some(w) => {
            let at =
                |x: f32| Point::new(x, y + w.amplitude * ((x + w.phase) / w.length * TAU).sin());
            path.move_to(at(x0));
            let steps = ((x1 - x0) / 2.0).ceil().max(1.0) as usize;
            for i in 1..=steps {
                path.line_to(at(x0 + (x1 - x0) * i as f32 / steps as f32));
            }
        }
    }
    builder.stroke_path(&path.finish(FillRule::NonZero)?, color, round(width))?;
    Ok(())
}

/// Strokes the arc of the circle at `center` with `radius` from angle `a0`
/// to `a1` (radians, clockwise from the top), waving radially if `wave`
/// has an amplitude; then `length` counts whole waves around the circle.
pub(super) fn arc(
    builder: &mut SceneBuilder,
    center: Point,
    radius: f32,
    (a0, a1): (f32, f32),
    width: f32,
    wave: Option<Wave>,
    color: Color,
) -> Result {
    if a1 <= a0 {
        return Ok(());
    }
    let wave = wave.filter(|w| w.amplitude > 0.01);
    let at = |a: f32| {
        let r = wave.map_or(radius, |w| {
            radius + w.amplitude * (a * w.length + w.phase).sin()
        });
        Point::new(center.x + r * a.sin(), center.y - r * a.cos())
    };
    // Short chords: flat arcs need few, waves more.
    let per_turn = if wave.is_some() { 160.0 } else { 64.0 };
    let steps = ((a1 - a0) / TAU * per_turn).ceil().max(2.0) as usize;
    let mut path = PathBuilder::new();
    path.move_to(at(a0));
    for i in 1..=steps {
        path.line_to(at(a0 + (a1 - a0) * i as f32 / steps as f32));
    }
    builder.stroke_path(&path.finish(FillRule::NonZero)?, color, round(width))?;
    Ok(())
}
