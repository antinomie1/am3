//! Scroll views in Material: Aegle's scroll view without its outline, with
//! a transparent track and an on-surface thumb that darkens while the view
//! is hovered or the thumb dragged. Multiline text fields share the colors.

use aegle_ui::{Appearance, Color, Container, Result, Theme, VisualState};
use aegle_widgets::{ScrollView, Widgets};

use crate::color::{Scheme, alpha};

/// A Material scroll view in `parent`: Aegle's [`ScrollView`] (a clipped,
/// scrolling column) with the Material scrollbar and no outline.
pub fn scroll_view(parent: &Container) -> Result<ScrollView> {
    let view = parent.scroll_view()?;
    view.set_skin(Some(skin))?;
    Ok(view)
}

/// Scrollbar track, resting thumb and active thumb in `s`.
pub(crate) fn scrollbar(s: &Scheme) -> [Color; 3] {
    [
        Color::TRANSPARENT,
        alpha(s.on_surface, 0.38),
        alpha(s.on_surface, 0.6),
    ]
}

fn skin(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        scrollbar: scrollbar(&Scheme::of(theme)),
        ..Appearance::base(theme, state)
    }
}
