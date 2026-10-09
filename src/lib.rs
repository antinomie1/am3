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
mod badge;
mod button;
mod chip;
mod dialog;
mod divider;
mod fab;
mod fab_menu;
mod glyph;
mod group;
mod icon_button;
mod list;
mod loading;
mod menu;
mod overlay;
mod pressable;
mod progress;
mod segmented;
mod selection;
mod sheet;
mod skin;
mod slider;
pub mod snackbar;
mod split;
mod surface;
mod text;
mod tooltip;

pub use badge::Badge;
pub use button::{Button, ButtonShape, ButtonSize, ButtonStyle};
pub use chip::{Chip, ChipKind};
pub use color::{Mode, Role, Scheme, theme};
pub use dialog::Dialog;
pub use divider::{Divider, DividerControl};
pub use fab::{Fab, FabColor, FabSize};
pub use fab_menu::{FabMenu, FabMenuItem};
pub use glyph::{IconControl, IconView};
pub use group::{ButtonGroup, GroupStyle, Selection};
pub use icon::{Icon, icons};
pub use icon_button::{IconButton, IconStyle, IconWidth};
pub use list::{List, ListItem};
pub use loading::{LoadingControl, LoadingIndicator};
pub use menu::Menu;
pub use pressable::PressableControl;
pub use progress::{Progress, ProgressControl};
pub use segmented::{Segment, SegmentedButton};
pub use selection::{Checkbox, Radio, SelectionControl, Switch};
pub use sheet::Sheet;
pub use slider::{Slider, SliderControl, SliderSize};
pub use snackbar::Snackbar;
pub use split::SplitButton;
pub use surface::{Card, CardStyle, SurfaceControl};
pub use text::{Text, TextControl};
pub use tooltip::{RichTooltip, set_tooltip};

/// Every kind this library declares, by component, for skin overrides with
/// `Node::set_kind_skin`.
pub mod kinds {
    pub use crate::button::{
        ELEVATED_BUTTON, ELEVATED_TOGGLE, FILLED_BUTTON, FILLED_TOGGLE, OUTLINED_BUTTON,
        OUTLINED_TOGGLE, TEXT_BUTTON, TONAL_BUTTON, TONAL_TOGGLE,
    };
    pub use crate::chip::{
        ASSIST_CHIP, ELEVATED_ASSIST_CHIP, ELEVATED_FILTER_CHIP, ELEVATED_SUGGESTION_CHIP,
        FILTER_CHIP, INPUT_CHIP, SUGGESTION_CHIP,
    };
    pub use crate::fab::{
        FAB_PRIMARY, FAB_PRIMARY_CONTAINER, FAB_SECONDARY, FAB_SECONDARY_CONTAINER, FAB_TERTIARY,
        FAB_TERTIARY_CONTAINER,
    };
    pub use crate::glyph::ICON;
    pub use crate::icon_button::{
        FILLED_ICON_BUTTON, FILLED_ICON_TOGGLE, OUTLINED_ICON_BUTTON, OUTLINED_ICON_TOGGLE,
        STANDARD_ICON_BUTTON, STANDARD_ICON_TOGGLE, TONAL_ICON_BUTTON, TONAL_ICON_TOGGLE,
    };
    pub use crate::list::{LIST, LIST_ITEM};
    pub use crate::loading::{CONTAINED_LOADING_INDICATOR, LOADING_INDICATOR};
    pub use crate::overlay::SCRIM;
    pub use crate::progress::PROGRESS;
    pub use crate::segmented::SEGMENT;
    pub use crate::selection::{CHECKBOX, CHECKBOX_ERROR, RADIO_BUTTON, SWITCH};
    pub use crate::slider::SLIDER;
    pub use crate::surface::{
        DIALOG, ELEVATED_CARD, FILLED_CARD, OUTLINED_CARD, RICH_TOOLTIP, SHEET, SNACKBAR,
        STANDARD_SHEET,
    };
    pub use crate::text::TEXT;
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

/// The engine hooks am3 needs: modal layers block input below them, keep
/// Tab inside and close on Escape or a timeout, and arrow keys move among
/// radio buttons. Every control that needs them installs them on creation.
pub static HOOKS: aegle_ui::Hooks = aegle_ui::Hooks {
    key: Some(key),
    press: None,
    overlay_at: Some(overlay::overlay_at),
    place: None,
    removed: Some(overlay::removed),
    removed_after: None,
    measure: None,
    realize: None,
    hover: None,
    wake: Some(overlay::wake),
};

fn key(state: &mut aegle_ui::State, key: &aegle_ui::KeyInput<'_>) -> aegle_ui::Result<bool> {
    Ok(overlay::key(state, key)? || selection::radio_key(state, key)?)
}

/// Margins or padding in logical pixels: left, top, right, bottom.
fn edges(left: f32, top: f32, right: f32, bottom: f32) -> aegle_ui::Insets {
    use aegle_ui::Length::Px;
    aegle_ui::Insets {
        left: Px(left),
        top: Px(top),
        right: Px(right),
        bottom: Px(bottom),
    }
}
