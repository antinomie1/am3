//! Material 3 Expressive controls for Aegle.
//!
//! A control library on Aegle's public authoring path: every control has its
//! own `ControlKind` with a skin computed from the theme, a typed handle and a
//! markup element. Colors come from the Material scheme of the node's theme
//! ([`color::Scheme::of`]), so the host's light, dark and high-contrast
//! themes and any subtree theme override restyle the controls; motion uses
//! Material's spring tokens through Aegle transitions.

pub mod color;
pub mod icon;
pub mod tokens;

mod anim;
mod button;
mod fab;
mod icon_button;
mod pressable;
mod skin;

pub use button::{Button, ButtonShape, ButtonSize, ButtonStyle};
pub use color::{Mode, Role, Scheme, theme};
pub use fab::{Fab, FabColor, FabSize};
pub use icon::{Icon, icons};
pub use icon_button::{IconButton, IconStyle, IconWidth};
pub use pressable::PressableControl;

/// Every kind this library declares, by component, for skin overrides with
/// `Node::set_kind_skin`.
pub mod kinds {
    pub use crate::button::{
        ELEVATED_BUTTON, ELEVATED_TOGGLE, FILLED_BUTTON, FILLED_TOGGLE, OUTLINED_BUTTON,
        OUTLINED_TOGGLE, TEXT_BUTTON, TONAL_BUTTON, TONAL_TOGGLE,
    };
    pub use crate::fab::{
        FAB_PRIMARY, FAB_PRIMARY_CONTAINER, FAB_SECONDARY, FAB_SECONDARY_CONTAINER, FAB_TERTIARY,
        FAB_TERTIARY_CONTAINER,
    };
    pub use crate::icon_button::{
        FILLED_ICON_BUTTON, FILLED_ICON_TOGGLE, OUTLINED_ICON_BUTTON, OUTLINED_ICON_TOGGLE,
        STANDARD_ICON_BUTTON, STANDARD_ICON_TOGGLE, TONAL_ICON_BUTTON, TONAL_ICON_TOGGLE,
    };
}

/// Gives a new control's colors Material's default effects spring.
fn effects(node: &aegle_ui::Node) -> aegle_ui::Result {
    #[cfg(feature = "motion")]
    node.set_property_transition(
        aegle_ui::TransitionProperty::Paint,
        Some(tokens::motion::DEFAULT_EFFECTS.transition()),
    )?;
    #[cfg(not(feature = "motion"))]
    let _ = node;
    Ok(())
}
