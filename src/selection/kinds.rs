//! The selection controls' kinds and skins.

use aegle_ui::{Appearance, ControlKind, Theme, VisualState};

use crate::{Scheme, color::alpha, skin, tokens::state};

/// The selection skins: the state layer follows the content under it,
/// the selected parts are primary (or error) and everything fades to 38 %
/// on-surface when disabled.
pub(super) fn selection(theme: &Theme, state: VisualState, error: bool) -> Appearance {
    let s = Scheme::of(theme);
    let base = Appearance::base(theme, state);
    let accent = if error { s.error } else { s.primary };
    let on_accent = if error { s.on_error } else { s.on_primary };
    let outline = if error { s.error } else { s.on_surface_variant };
    let layer = skin::layer_opacity(state).max(if state.pressed { state::PRESSED } else { 0.0 });
    let layer_color = if state.checked { accent } else { s.on_surface };
    let switch = std::ptr::eq(state.kind, &SWITCH);
    let disabled = |c| alpha(c, state::DISABLED_CONTENT);
    Appearance {
        foreground: if state.enabled {
            s.on_surface
        } else {
            disabled(s.on_surface)
        },
        background: alpha(layer_color, layer),
        indicator: if state.enabled {
            accent
        } else {
            disabled(s.on_surface)
        },
        border_color: match (state.enabled, switch) {
            (true, false) => outline,
            (true, true) => s.outline,
            (false, _) => disabled(s.on_surface),
        },
        border_width: 2.0,
        caret: match (state.enabled, switch, state.checked) {
            (true, false, _) => on_accent,
            (true, true, true) if state.hovered || state.pressed || state.focused => {
                s.primary_container
            }
            (true, true, true) => s.on_primary,
            (true, true, false) if state.hovered || state.pressed || state.focused => {
                s.on_surface_variant
            }
            (true, true, false) => s.outline,
            (false, _, true) => s.surface,
            (false, _, false) => disabled(s.on_surface),
        },
        focus_color: s.secondary,
        focus_width: 0.0,
        ..base
    }
}

fn checkbox_skin(theme: &Theme, state: VisualState) -> Appearance {
    selection(theme, state, false)
}

fn error_skin(theme: &Theme, state: VisualState) -> Appearance {
    selection(theme, state, true)
}

const ACCEPTS: aegle_ui::Accepts = skin::PRESSABLE;

/// A checkbox.
pub static CHECKBOX: ControlKind = ControlKind {
    name: "Checkbox",
    skin: checkbox_skin,
    accepts: ACCEPTS,
    container: false,
};
/// A checkbox reporting an error.
pub static CHECKBOX_ERROR: ControlKind = ControlKind {
    name: "ErrorCheckbox",
    skin: error_skin,
    accepts: ACCEPTS,
    container: false,
};
/// A radio button.
pub static RADIO_BUTTON: ControlKind = ControlKind {
    name: "RadioButton",
    skin: checkbox_skin,
    accepts: ACCEPTS,
    container: false,
};
/// A switch.
pub static SWITCH: ControlKind = ControlKind {
    name: "Switch",
    skin: checkbox_skin,
    accepts: ACCEPTS,
    container: false,
};
