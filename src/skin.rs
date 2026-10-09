//! What every Material skin and control shares: state layers, disabled
//! colors, the focus ring and elevation shadows.

use aegle_ui::{
    Appearance, Color, Rect, Result, Theme, VisualState,
    scene::{RoundedRect, SceneBuilder},
};

use crate::{
    Scheme,
    color::{alpha, blend},
    tokens::{elevation, state},
};

/// A control's colors at rest, before state layers and disabling.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Paint {
    pub container: Color,
    pub content: Color,
    /// Icons, when they differ from the label.
    pub icon: Color,
    pub outline: Option<Color>,
}

impl Paint {
    pub fn new(container: Color, content: Color) -> Self {
        Self {
            container,
            content,
            icon: content,
            outline: None,
        }
    }

    pub fn icon(mut self, icon: Color) -> Self {
        self.icon = icon;
        self
    }

    pub fn outlined(mut self, outline: Color) -> Self {
        self.outline = Some(outline);
        self
    }
}

fn clear(color: Color) -> bool {
    color.to_rgba()[3] == 0
}

/// `content` layered over `container` at `opacity`.
pub(crate) fn layer(container: Color, content: Color, opacity: f32) -> Color {
    if clear(container) {
        alpha(content, opacity)
    } else {
        blend(container, content, opacity)
    }
}

/// The opacity of the hover or focus layer; the press shows as a ripple
/// over the hover layer.
pub(crate) fn layer_opacity(state: VisualState) -> f32 {
    if !state.enabled {
        0.0
    } else if state.focused {
        state::FOCUS
    } else if state.hovered || state.pressed {
        state::HOVER
    } else {
        0.0
    }
}

/// The appearance of a pressable control from its resting colors. The
/// indicator carries the icon color; the control draws the focus ring
/// itself, outside its bounds, so the engine's is off.
pub(crate) fn pressable(theme: &Theme, state: VisualState, s: &Scheme, rest: Paint) -> Appearance {
    let base = Appearance::base(theme, state);
    let (background, foreground, icon, outline) = if state.enabled {
        let opacity = layer_opacity(state);
        let background = if opacity > 0.0 {
            layer(rest.container, rest.content, opacity)
        } else {
            rest.container
        };
        (background, rest.content, rest.icon, rest.outline)
    } else {
        let content = alpha(s.on_surface, state::DISABLED_CONTENT);
        let container = if clear(rest.container) {
            Color::TRANSPARENT
        } else {
            alpha(s.on_surface, state::DISABLED_CONTAINER)
        };
        let outline = rest
            .outline
            .map(|_| alpha(s.on_surface, state::DISABLED_CONTAINER));
        (container, content, content, outline)
    };
    Appearance {
        background,
        foreground,
        indicator: icon,
        border_color: outline.unwrap_or(Color::TRANSPARENT),
        border_width: if outline.is_some() { 1.0 } else { 0.0 },
        focus_color: s.secondary,
        focus_width: 0.0,
        caret: s.primary,
        selection: alpha(s.primary, 0.4),
        ..base
    }
}

/// Material's focus indicator: 3 dp in the focus color, 2 dp outside the
/// control's shape `rect`.
pub(crate) fn focus_ring(
    builder: &mut SceneBuilder,
    rect: Rect,
    radius: f32,
    color: Color,
) -> Result {
    const OFFSET: f32 = 2.0 + 1.5;
    let ring = Rect::new(
        rect.origin.x - OFFSET,
        rect.origin.y - OFFSET,
        rect.size.width + 2.0 * OFFSET,
        rect.size.height + 2.0 * OFFSET,
    );
    builder.stroke(RoundedRect::new(ring, radius + OFFSET)?, color, 3.0)?;
    Ok(())
}

/// The shadow of `shape` at elevation `level`.
pub(crate) fn shadow(
    builder: &mut SceneBuilder,
    shape: RoundedRect,
    level: f32,
    color: Color,
) -> Result {
    if let Some(shadow) = elevation(level, color) {
        let r = shape.rect();
        let moved = Rect::new(
            r.origin.x + shadow.offset.x,
            r.origin.y + shadow.offset.y,
            r.size.width,
            r.size.height,
        );
        builder.shadow(
            RoundedRect::new(moved, shape.radius())?,
            shadow.color,
            shadow.blur,
        )?;
    }
    Ok(())
}

/// Declares pressable kinds, each with a skin computing its resting colors
/// from the scheme: `NAME = "Name", skin_fn => |scheme, selected| paint;`.
macro_rules! kinds {
    ($($(#[$doc:meta])* $kind:ident = $name:literal, $skin:ident => |$s:ident, $on:ident| $paint:expr;)*) => {$(
        fn $skin(theme: &aegle_ui::Theme, state: aegle_ui::VisualState) -> aegle_ui::Appearance {
            let $s = &$crate::Scheme::of(theme);
            let $on = state.checked;
            $crate::skin::pressable(theme, state, $s, $paint)
        }
        $(#[$doc])*
        pub static $kind: aegle_ui::ControlKind = aegle_ui::ControlKind {
            name: $name,
            skin: $skin,
            accepts: $crate::skin::PRESSABLE,
            container: false,
        };
    )*};
}
pub(crate) use kinds;

/// What pressable kinds accept: text, interaction, press and icon colors.
pub(crate) const PRESSABLE: aegle_ui::Accepts = aegle_ui::Accepts::TEXT
    .with(aegle_ui::Accepts::INTERACTIVE)
    .with(aegle_ui::Accepts::PRESSED)
    .with(aegle_ui::Accepts::INDICATOR);
