//! The radio mark: a 20 dp ring whose center dot springs in when chosen.

use aegle_ui::{
    Container, Point, Result, handle,
    scene::{Rect, RoundedRect},
};

use super::{Mark, MarkPaint, SelectionControl, selection_methods};

const RING: f32 = 20.0;
const DOT: f32 = 10.0;

/// Paints the mark, returning its center.
pub(super) fn paint(p: &mut MarkPaint<'_>) -> Result<Point> {
    let center = Point::new(
        p.origin.x + p.size.width / 2.0,
        p.origin.y + p.size.height / 2.0,
    );
    p.layer(center)?;
    let a = p.appearance;
    let color = if p.on > 0.5 {
        a.indicator
    } else {
        a.border_color
    };
    let circle = |d: f32| {
        RoundedRect::new(
            Rect::new(center.x - d / 2.0, center.y - d / 2.0, d, d),
            d / 2.0,
        )
    };
    p.builder.stroke(circle(RING - 2.0)?, color, 2.0)?;
    let dot = DOT * p.on.max(0.0);
    if dot > 0.1 {
        p.builder.fill(circle(dot)?, a.indicator)?;
    }
    Ok(center)
}

handle! {
    /// A radio button, exclusive among its sibling radio buttons, with an
    /// optional label after it. Arrow keys move the choice.
    pub Radio(SelectionControl): text, interactive, pressed, indicator
}

selection_methods!(Radio);

impl Radio {
    /// Appends a radio button; an empty label shows the ring alone.
    pub fn new(parent: &Container, text: &str, checked: bool) -> Result<Self> {
        super::add(parent, Mark::Radio, text, checked).map(Self)
    }
}
