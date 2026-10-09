//! What a pressable control is: its component and variant ([`Look`]) and
//! the geometry that gives ([`Spec`]).

use aegle_ui::ControlKind;

use crate::tokens::{Corner, TypeStyle};

/// The geometry of one size of a pressable component.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Spec {
    pub height: f32,
    /// A fixed width, as icon buttons and FABs have.
    pub width: Option<f32>,
    pub min_width: f32,
    /// Space before the first and after the last content.
    pub padding: f32,
    /// Space beside an icon at the start or end instead, as chips have.
    pub icon_padding: f32,
    pub icon: f32,
    pub gap: f32,
    pub text: TypeStyle,
    /// Outer corners at rest, pressed and selected.
    pub corners: [Corner; 3],
    /// Corners where the control joins a neighbor, in the same states.
    pub inner: [Corner; 3],
    /// Hover and focus take the inner corners to their pressed shape too.
    pub hover_inner: bool,
    /// Outline width, scaling the skin's border width.
    pub outline: f32,
    /// Elevation level at rest and while hovered.
    pub elevation: [f32; 2],
}

/// Which component a pressable control is, with its variant: this picks
/// its kind and its [`Spec`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Look {
    Button(crate::button::ButtonLook),
    Icon(crate::icon_button::IconLook),
    Fab(crate::fab::FabLook),
    Segment(crate::segmented::SegmentLook),
    FabItem(crate::fab::FabColor),
    Chip(crate::chip::ChipLook),
    Time(crate::time::TimeLook),
}

impl Look {
    pub(super) fn kind(self, toggle: bool) -> &'static ControlKind {
        match self {
            Self::Button(look) => look.kind(toggle),
            Self::Icon(look) => look.kind(toggle),
            Self::Fab(look) => look.kind(),
            Self::Segment(_) => &crate::segmented::SEGMENT,
            Self::FabItem(color) => crate::fab_menu::item_kind(color),
            Self::Chip(look) => look.kind(),
            Self::Time(look) => look.kind(),
        }
    }

    pub(super) fn spec(self) -> Spec {
        match self {
            Self::Button(look) => look.spec(),
            Self::Icon(look) => look.spec(),
            Self::Fab(look) => look.spec(),
            Self::Segment(look) => look.spec(),
            Self::FabItem(_) => crate::fab_menu::item_spec(),
            Self::Chip(look) => look.spec(),
            Self::Time(look) => look.spec(),
        }
    }
}
