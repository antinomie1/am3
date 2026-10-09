//! The switch mark: a 52 × 32 dp track whose handle slides and grows from
//! 16 dp (24 with an icon) off to 24 dp on, and swells to 28 dp pressed.

use aegle_ui::{
    Color, Container, Point, Result, handle,
    scene::{Rect, RoundedRect},
};

use super::{Mark, MarkPaint, SelectionControl, selection_methods};
use crate::{Icon, color::alpha, tokens::state};

/// Paints the mark, returning the handle's center.
pub(super) fn paint(
    p: &mut MarkPaint<'_>,
    icons: &[Option<Icon>; 2],
    selected: bool,
) -> Result<Point> {
    let a = p.appearance;
    let s = p.scheme;
    let on = p.on;
    let track = Rect::new(p.origin.x, p.origin.y, p.size.width, p.size.height);
    let radius = p.size.height / 2.0;
    let mix =
        |off: Color, on_color: Color, t: f32| crate::color::blend(off, on_color, t.clamp(0.0, 1.0));
    let (off_track, on_track) = if p.enabled {
        (s.surface_container_highest, a.indicator)
    } else {
        (
            alpha(s.surface_container_highest, state::DISABLED_CONTAINER),
            alpha(s.on_surface, state::DISABLED_CONTAINER),
        )
    };
    let fill = if on <= 0.0 {
        off_track
    } else if on >= 1.0 {
        on_track
    } else {
        mix(off_track, on_track, on)
    };
    p.builder.fill(RoundedRect::new(track, radius)?, fill)?;
    if on < 1.0 {
        let edge = Rect::new(
            track.origin.x + 1.0,
            track.origin.y + 1.0,
            track.size.width - 2.0,
            track.size.height - 2.0,
        );
        let outline = alpha(a.border_color, 1.0 - on.clamp(0.0, 1.0));
        p.builder
            .stroke(RoundedRect::new(edge, radius - 1.0)?, outline, 2.0)?;
    }

    let icon = &icons[usize::from(!selected)];
    let off_size = if icon.is_some() { 24.0 } else { 16.0 };
    let size = off_size + (24.0 - off_size) * on;
    let size = size + (28.0 - size) * p.press.clamp(0.0, 1.0);
    // The handle's center travels between 16 dp from either end.
    let travel = p.size.width - 2.0 * radius;
    let offset = if p.rtl { 1.0 - on } else { on };
    let center = Point::new(
        track.origin.x + radius + travel * offset,
        track.origin.y + radius,
    );
    p.layer(center)?;
    let handle = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size);
    p.builder
        .fill(RoundedRect::new(handle, size / 2.0)?, a.caret)?;
    if let Some(icon) = icon {
        let color = match (p.enabled, selected) {
            (true, true) => s.on_primary_container,
            (true, false) => s.surface_container_highest,
            (false, true) => alpha(s.on_surface, state::DISABLED_CONTENT),
            (false, false) => s.surface_container_highest,
        };
        icon.paint(
            p.builder,
            Point::new(center.x - 8.0, center.y - 8.0),
            16.0,
            color,
        )?;
    }
    Ok(center)
}

handle! {
    /// A switch, with an optional label before it.
    pub Switch(SelectionControl): text, interactive, pressed, indicator
}

selection_methods!(Switch);

impl Switch {
    /// Appends a switch; an empty label shows the track alone.
    pub fn new(parent: &Container, text: &str, on: bool) -> Result<Self> {
        super::add(parent, Mark::Switch, text, on).map(Self)
    }
    /// Shows icons on the handle: one while on, one while off (Material
    /// uses a check and nothing, or a check and a close icon).
    pub fn set_icons(&self, on: Option<Icon>, off: Option<Icon>) -> Result {
        self.update(|c| c.icons = [on, off])
    }
}
