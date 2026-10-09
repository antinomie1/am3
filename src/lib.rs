//! Material 3 Expressive controls for Aegle.
//!
//! A control library on Aegle's public authoring path: every control has its
//! own `ControlKind` with a skin computed from the theme, a typed handle and a
//! markup element. Colors come from the Material scheme of the node's theme
//! ([`color::Scheme::of`]), so the host's light, dark and high-contrast
//! themes and any subtree theme override restyle the controls; motion uses
//! Material's spring tokens through Aegle transitions.

// The counting allocator of the `thousand` example needs unsafe; the
// library never does.
#![forbid(unsafe_code)]

pub mod color;
pub mod icon;
pub mod tokens;

mod anim;
mod app_bar;
mod badge;
mod button;
mod carousel;
mod chip;
mod date;
mod dialog;
mod divider;
mod fab;
mod fab_menu;
mod field;
mod glyph;
mod group;
mod icon_button;
mod list;
mod loading;
mod markup;
mod menu;
mod navigation;
mod overlay;
mod pressable;
mod progress;
mod scroll;
mod search;
mod segmented;
mod selection;
mod sheet;
mod skin;
mod slider;
pub mod snackbar;
mod split;
mod surface;
mod tabs;
mod text;
mod time;
mod toolbar;
mod tooltip;

pub use app_bar::{AppBarStyle, BottomAppBar, TopAppBar};
pub use badge::Badge;
pub use button::{Button, ButtonShape, ButtonSize, ButtonStyle};
pub use carousel::{Carousel, CarouselControl};
pub use chip::{Chip, ChipKind};
pub use color::{Mode, Role, Scheme, theme};
pub use date::{CalendarControl, Date, DatePicker};
pub use dialog::Dialog;
pub use divider::{Divider, DividerControl};
pub use fab::{Fab, FabColor, FabSize};
pub use fab_menu::{FabMenu, FabMenuItem};
pub use field::{FieldControl, FieldStyle, TextField};
pub use glyph::{IconControl, IconView};
pub use group::{ButtonGroup, GroupStyle, Selection};
pub use icon::{Icon, icons};
pub use icon_button::{IconButton, IconStyle, IconWidth};
pub use list::{List, ListItem};
pub use loading::{LoadingControl, LoadingIndicator};
pub use markup::*;
pub use menu::Menu;
pub use navigation::{NavItem, NavItemControl, NavigationBar, NavigationDrawer, NavigationRail};
pub use pressable::PressableControl;
pub use progress::{Progress, ProgressControl};
pub use scroll::scroll_view;
pub use search::Search;
pub use segmented::{Segment, SegmentedButton};
pub use selection::{Checkbox, Radio, SelectionControl, Switch};
pub use sheet::Sheet;
pub use slider::{Slider, SliderControl, SliderSize};
pub use snackbar::Snackbar;
pub use split::SplitButton;
pub use surface::{Card, CardStyle, SurfaceControl};
pub use tabs::{Tab, TabControl, TabRowControl, TabStyle, Tabs};
pub use text::{Text, TextControl};
pub use time::{DialControl, TimeField, TimePicker};
pub use toolbar::{Toolbar, ToolbarColor};
pub use tooltip::{RichTooltip, set_tooltip};

/// Every kind this library declares, by component, for skin overrides with
/// `Node::set_kind_skin`.
pub mod kinds {
    pub use crate::app_bar::{BOTTOM_APP_BAR, TOP_APP_BAR, TOP_APP_BAR_SCROLLED};
    pub use crate::button::{
        ELEVATED_BUTTON, ELEVATED_TOGGLE, FILLED_BUTTON, FILLED_TOGGLE, OUTLINED_BUTTON,
        OUTLINED_TOGGLE, TEXT_BUTTON, TONAL_BUTTON, TONAL_TOGGLE,
    };
    pub use crate::carousel::{CAROUSEL, CAROUSEL_ITEM};
    pub use crate::chip::{
        ASSIST_CHIP, ELEVATED_ASSIST_CHIP, ELEVATED_FILTER_CHIP, ELEVATED_SUGGESTION_CHIP,
        FILTER_CHIP, INPUT_CHIP, SUGGESTION_CHIP,
    };
    pub use crate::date::{CALENDAR, DOCKED_DATE_PICKER};
    pub use crate::fab::{
        FAB_PRIMARY, FAB_PRIMARY_CONTAINER, FAB_SECONDARY, FAB_SECONDARY_CONTAINER, FAB_TERTIARY,
        FAB_TERTIARY_CONTAINER,
    };
    pub use crate::field::{
        FILLED_TEXT_FIELD, FILLED_TEXT_FIELD_ERROR, OUTLINED_TEXT_FIELD, OUTLINED_TEXT_FIELD_ERROR,
        SEARCH_BAR,
    };
    pub use crate::glyph::ICON;
    pub use crate::icon_button::{
        FILLED_ICON_BUTTON, FILLED_ICON_TOGGLE, OUTLINED_ICON_BUTTON, OUTLINED_ICON_TOGGLE,
        STANDARD_ICON_BUTTON, STANDARD_ICON_TOGGLE, TONAL_ICON_BUTTON, TONAL_ICON_TOGGLE,
    };
    pub use crate::list::{LIST, LIST_ITEM};
    pub use crate::loading::{CONTAINED_LOADING_INDICATOR, LOADING_INDICATOR};
    pub use crate::navigation::{NAV_BAR, NAV_DRAWER_ITEM, NAV_ITEM, NAV_RAIL};
    pub use crate::overlay::SCRIM;
    pub use crate::progress::PROGRESS;
    pub use crate::search::SEARCH_VIEW;
    pub use crate::segmented::SEGMENT;
    pub use crate::selection::{CHECKBOX, CHECKBOX_ERROR, RADIO_BUTTON, SWITCH};
    pub use crate::slider::SLIDER;
    pub use crate::surface::{
        DIALOG, ELEVATED_CARD, FILLED_CARD, OUTLINED_CARD, RICH_TOOLTIP, SHEET, SNACKBAR,
        STANDARD_SHEET,
    };
    pub use crate::tabs::{SECONDARY_TAB, TAB, TAB_ROW};
    pub use crate::text::TEXT;
    pub use crate::time::{DIAL, PERIOD_BOX, TIME_FIELD, TIME_PERIOD, TIME_PICKER};
    pub use crate::toolbar::{TOOLBAR, VIBRANT_TOOLBAR};
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
/// Tab inside and close on Escape or a timeout, tab indicators follow
/// layout, and arrow keys move among radio buttons. Every control that needs them installs them on creation.
pub static HOOKS: aegle_ui::Hooks = aegle_ui::Hooks {
    key: Some(key),
    press: None,
    overlay_at: Some(overlay::overlay_at),
    place: Some(tabs::place),
    removed: Some(removed),
    removed_after: None,
    measure: None,
    realize: None,
    hover: None,
    wake: Some(overlay::wake),
};

fn removed(state: &mut aegle_ui::State, id: aegle_ui::NodeId) {
    overlay::removed(state, id);
    tabs::removed(state, id);
    time::removed(state, id);
    markup::removed(state, id);
}

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
