//! The checkbox mark: an 18 dp box that fills and draws its check (or the
//! indeterminate dash) as it turns on.

use aegle_ui::{
    Color, Container, Point, Result, handle,
    scene::{LineCap, LineJoin, PathBuilder, Rect, RoundedRect, Stroke},
};

use super::{Mark, MarkPaint, SelectionControl, selection_methods};
use crate::color::alpha;

const BOX: f32 = 18.0;

/// Paints the mark, returning its center.
pub(super) fn paint(p: &mut MarkPaint<'_>, mixed: bool) -> Result<Point> {
    let center = Point::new(
        p.origin.x + p.size.width / 2.0,
        p.origin.y + p.size.height / 2.0,
    );
    p.layer(center)?;
    let rect = Rect::new(center.x - BOX / 2.0, center.y - BOX / 2.0, BOX, BOX);
    let a = p.appearance;
    let on = p.on.clamp(0.0, 1.0);
    if on < 1.0 {
        // The outline, 2 dp inside the box.
        let inner = Rect::new(
            rect.origin.x + 1.0,
            rect.origin.y + 1.0,
            BOX - 2.0,
            BOX - 2.0,
        );
        p.builder
            .stroke(RoundedRect::new(inner, 1.0)?, a.border_color, 2.0)?;
    }
    if on > 0.0 {
        let fill = alpha(a.indicator, (on * 2.0).min(1.0));
        p.builder.fill(RoundedRect::new(rect, 2.0)?, fill)?;
        mark(p, rect, mixed, on, a.caret)?;
    }
    Ok(center)
}

/// The check or dash, drawn up to `progress` of its length.
fn mark(p: &mut MarkPaint<'_>, rect: Rect, mixed: bool, progress: f32, color: Color) -> Result {
    let at = |x: f32, y: f32| Point::new(rect.origin.x + x, rect.origin.y + y);
    let points: &[Point] = if mixed {
        &[at(4.0, 9.0), at(14.0, 9.0)]
    } else {
        &[at(3.5, 9.5), at(7.0, 13.0), at(14.5, 5.5)]
    };
    let lengths: Vec<f32> = points
        .windows(2)
        .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
        .collect();
    let mut left = progress * lengths.iter().sum::<f32>();
    let mut path = PathBuilder::new();
    path.move_to(points[0]);
    for (w, length) in points.windows(2).zip(lengths) {
        let t = (left / length).min(1.0);
        path.line_to(Point::new(
            w[0].x + (w[1].x - w[0].x) * t,
            w[0].y + (w[1].y - w[0].y) * t,
        ));
        left -= length;
        if left <= 0.0 {
            break;
        }
    }
    let stroke = Stroke {
        width: 2.0,
        cap: LineCap::Square,
        join: LineJoin::Miter,
    };
    p.builder.stroke_path(
        &path.finish(aegle_ui::scene::FillRule::NonZero)?,
        color,
        stroke,
    )?;
    Ok(())
}

handle! {
    /// A checkbox, with an optional label after it.
    pub Checkbox(SelectionControl): text, interactive, pressed, indicator
}

selection_methods!(Checkbox);

impl Checkbox {
    /// Appends a checkbox; an empty label shows the box alone.
    pub fn new(parent: &Container, text: &str, checked: bool) -> Result<Self> {
        super::add(parent, Mark::Checkbox, text, checked).map(Self)
    }
    /// Shows the indeterminate dash until the next change.
    pub fn set_mixed(&self, mixed: bool) -> Result {
        self.update(|c| c.mixed = mixed)
    }
    /// Whether it shows the indeterminate dash.
    pub fn is_mixed(&self) -> Result<bool> {
        self.read(|c| c.mixed)
    }
    /// Shows the box in the error color, for an invalid required choice.
    pub fn set_error(&self, error: bool) -> Result {
        self.update(|c| c.error = error)
    }
}
