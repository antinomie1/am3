//! The container shape of a pressable control: one radius on its start
//! side and one on its end side, so connected groups, split buttons and
//! segments can join square inner edges to rounded outer ones.

use aegle_ui::{
    Point, Rect, Result,
    scene::{FillRule, Path, PathBuilder, RoundedRect},
};

/// How far a cubic control point sits along a quarter circle's tangent.
const KAPPA: f32 = 0.552_284_8;

/// A shape with independent left and right radii, as a rounded rectangle
/// when they match and otherwise as a cached path.
#[derive(Default)]
pub(crate) struct Shape {
    cached: Option<([f32; 5], Path)>,
}

/// What to draw: a rounded rectangle, or a path for unequal sides.
pub(crate) enum Outline<'a> {
    Rect(RoundedRect),
    Path(&'a Path, RoundedRect),
}

impl Outline<'_> {
    /// A rounded rectangle approximating the shape: itself, or one with the
    /// larger radius, for shadows.
    pub fn bounds(&self) -> RoundedRect {
        match self {
            Self::Rect(rect) | Self::Path(_, rect) => *rect,
        }
    }
}

impl Shape {
    /// The outline of the box `at` with `left` and `right` radii, inset by
    /// `inset` on every side (for strokes inside the edge).
    pub fn outline(&mut self, at: Rect, left: f32, right: f32, inset: f32) -> Result<Outline<'_>> {
        let rect = Rect::new(
            at.origin.x + inset,
            at.origin.y + inset,
            (at.size.width - 2.0 * inset).max(0.0),
            (at.size.height - 2.0 * inset).max(0.0),
        );
        let (left, right) = ((left - inset).max(0.0), (right - inset).max(0.0));
        let bounds = RoundedRect::new(rect, left.max(right))?;
        if (left - right).abs() < 0.01 {
            return Ok(Outline::Rect(bounds));
        }
        let key = [
            rect.origin.x,
            rect.size.width,
            rect.size.height,
            left,
            right,
        ];
        if self.cached.as_ref().is_none_or(|(k, _)| *k != key) {
            self.cached = Some((key, path(rect, left, right)?));
        }
        let (_, path) = self.cached.as_ref().expect("just cached");
        Ok(Outline::Path(path, bounds))
    }
}

pub(crate) fn path(rect: Rect, left: f32, right: f32) -> Result<Path> {
    rounded(rect, [left, right, right, left])
}

/// A rectangle with its top-left, top-right, bottom-right and bottom-left
/// corners rounded by their own radii.
pub(crate) fn rounded(rect: Rect, radii: [f32; 4]) -> Result<Path> {
    let half = rect.size.width.min(rect.size.height) / 2.0;
    let [tl, tr, br, bl] = radii.map(|r| r.clamp(0.0, half));
    let (x0, y0) = (rect.origin.x, rect.origin.y);
    let (x1, y1) = (x0 + rect.size.width, y0 + rect.size.height);
    let p = Point::new;
    let mut b = PathBuilder::new();
    b.move_to(p(x0 + tl, y0)).line_to(p(x1 - tr, y0));
    corner(&mut b, p(x1 - tr, y0), p(x1, y0 + tr), p(x1, y0), tr);
    b.line_to(p(x1, y1 - br));
    corner(&mut b, p(x1, y1 - br), p(x1 - br, y1), p(x1, y1), br);
    b.line_to(p(x0 + bl, y1));
    corner(&mut b, p(x0 + bl, y1), p(x0, y1 - bl), p(x0, y1), bl);
    b.line_to(p(x0, y0 + tl));
    corner(&mut b, p(x0, y0 + tl), p(x0 + tl, y0), p(x0, y0), tl);
    b.close();
    Ok(b.finish(FillRule::NonZero)?)
}

/// A quarter circle from `from` to `to` around the square corner `at`.
pub(crate) fn corner(b: &mut PathBuilder, from: Point, to: Point, at: Point, radius: f32) {
    if radius <= 0.0 {
        b.line_to(to);
        return;
    }
    let toward =
        |a: Point, c: Point| Point::new(a.x + (c.x - a.x) * KAPPA, a.y + (c.y - a.y) * KAPPA);
    b.cubic_to(toward(from, at), toward(to, at), to);
}
