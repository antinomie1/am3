//! Common buttons: five color styles in five sizes, round or square, plain
//! or toggle. Pressing morphs the corners toward the pressed shape; a
//! selected toggle turns square, an unselected one round.

use aegle_ui::{Container, ControlKind, Result, handle};

use crate::{
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
    tokens::{Corner, shape, typescale},
};

/// A button's color style, by emphasis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    /// A tinted surface with a shadow, for separation from patterned
    /// backgrounds.
    Elevated,
    /// The highest emphasis: primary color.
    #[default]
    Filled,
    /// A secondary-container color between filled and outlined.
    Tonal,
    /// An outline and no container.
    Outlined,
    /// Only a label, for the lowest emphasis.
    Text,
}

/// A button size; icon buttons and button groups share them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// 32 dp high.
    ExtraSmall,
    /// 40 dp high, the default.
    #[default]
    Small,
    /// 56 dp high.
    Medium,
    /// 96 dp high.
    Large,
    /// 136 dp high.
    ExtraLarge,
}

/// The resting shape of a button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    /// Fully rounded ends.
    #[default]
    Round,
    /// Rounded corners of the size's square radius.
    Square,
}

impl ButtonSize {
    /// Height, icon size, the square and pressed radii, and the outline width.
    pub(crate) fn metrics(self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::ExtraSmall => (32.0, 20.0, 12.0, 8.0, 1.0),
            Self::Small => (40.0, 20.0, 12.0, 8.0, 1.0),
            Self::Medium => (56.0, 24.0, 16.0, 12.0, 1.0),
            Self::Large => (96.0, 32.0, 28.0, 16.0, 2.0),
            Self::ExtraLarge => (136.0, 40.0, 28.0, 16.0, 3.0),
        }
    }

    /// The resting, pressed and selected corners of `shape`; a toggle
    /// changes to the other shape when selected.
    pub(crate) fn corners(self, shape: ButtonShape) -> [Corner; 3] {
        let (_, _, square, pressed, _) = self.metrics();
        let (round, square) = (shape::FULL, Corner::Dp(square));
        match shape {
            ButtonShape::Round => [round, Corner::Dp(pressed), square],
            ButtonShape::Square => [square, Corner::Dp(pressed), round],
        }
    }
}

impl ButtonSize {
    /// The corners where a button joins its neighbor, at rest, pressed (and
    /// for a split button also hovered or focused) and selected: a connected
    /// group squares them when pressed, a split button rounds them.
    pub(crate) fn inner(self, split: bool) -> [Corner; 3] {
        let (rest, active) = match (self, split) {
            (Self::ExtraSmall, false) => (4.0, 4.0),
            (Self::Small | Self::Medium, false) => (8.0, 4.0),
            (Self::Large, false) => (16.0, 12.0),
            (Self::ExtraLarge, false) => (20.0, 16.0),
            (Self::ExtraSmall, true) => (4.0, 8.0),
            (Self::Small | Self::Medium, true) => (4.0, 12.0),
            (Self::Large, true) => (8.0, 20.0),
            (Self::ExtraLarge, true) => (12.0, 20.0),
        };
        [Corner::Dp(rest), Corner::Dp(active), shape::FULL]
    }
}

/// The variant of a common button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ButtonLook {
    pub style: ButtonStyle,
    pub size: ButtonSize,
    pub shape: ButtonShape,
    /// Part of a split button: its colors stay when selected.
    pub split: bool,
}

impl ButtonLook {
    pub fn kind(self, toggle: bool) -> &'static ControlKind {
        match (self.style, toggle && !self.split) {
            (ButtonStyle::Elevated, false) => &ELEVATED_BUTTON,
            (ButtonStyle::Elevated, true) => &ELEVATED_TOGGLE,
            (ButtonStyle::Filled, false) => &FILLED_BUTTON,
            (ButtonStyle::Filled, true) => &FILLED_TOGGLE,
            (ButtonStyle::Tonal, false) => &TONAL_BUTTON,
            (ButtonStyle::Tonal, true) => &TONAL_TOGGLE,
            (ButtonStyle::Outlined, false) => &OUTLINED_BUTTON,
            (ButtonStyle::Outlined, true) => &OUTLINED_TOGGLE,
            (ButtonStyle::Text, _) => &TEXT_BUTTON,
        }
    }

    pub fn spec(self) -> Spec {
        let (height, icon, _, _, outline) = self.size.metrics();
        let (padding, gap, text) = match self.size {
            ButtonSize::ExtraSmall => (12.0, 4.0, typescale::LABEL_LARGE),
            ButtonSize::Small => (16.0, 8.0, typescale::LABEL_LARGE),
            ButtonSize::Medium => (24.0, 8.0, typescale::TITLE_MEDIUM),
            ButtonSize::Large => (48.0, 12.0, typescale::HEADLINE_SMALL),
            ButtonSize::ExtraLarge => (64.0, 16.0, typescale::HEADLINE_LARGE),
        };
        let [corner, pressed, selected] = self.size.corners(self.shape);
        let elevation = match self.style {
            ButtonStyle::Elevated => [1.0, 2.0],
            ButtonStyle::Filled | ButtonStyle::Tonal => [0.0, 1.0],
            ButtonStyle::Outlined | ButtonStyle::Text => [0.0, 0.0],
        };
        Spec {
            height,
            width: None,
            min_width: height,
            padding,
            icon,
            gap,
            text,
            corners: [corner, pressed, selected],
            inner: self.size.inner(self.split),
            hover_inner: self.split,
            outline,
            elevation,
        }
    }
}

kinds! {
    /// An elevated button.
    ELEVATED_BUTTON = "ElevatedButton", elevated_button => |s, _on| Paint::new(s.surface_container_low, s.primary);
    /// An elevated toggle button.
    ELEVATED_TOGGLE = "ElevatedToggleButton", elevated_toggle => |s, on| if on {
        Paint::new(s.primary, s.on_primary)
    } else {
        Paint::new(s.surface_container_low, s.primary)
    };
    /// A filled button.
    FILLED_BUTTON = "FilledButton", filled_button => |s, _on| Paint::new(s.primary, s.on_primary);
    /// A filled toggle button.
    FILLED_TOGGLE = "FilledToggleButton", filled_toggle => |s, on| if on {
        Paint::new(s.primary, s.on_primary)
    } else {
        Paint::new(s.surface_container, s.on_surface_variant)
    };
    /// A tonal button.
    TONAL_BUTTON = "TonalButton", tonal_button => |s, _on| Paint::new(s.secondary_container, s.on_secondary_container);
    /// A tonal toggle button.
    TONAL_TOGGLE = "TonalToggleButton", tonal_toggle => |s, on| if on {
        Paint::new(s.secondary, s.on_secondary)
    } else {
        Paint::new(s.secondary_container, s.on_secondary_container)
    };
    /// An outlined button.
    OUTLINED_BUTTON = "OutlinedButton", outlined_button => |s, _on| {
        Paint::new(aegle_ui::Color::TRANSPARENT, s.on_surface_variant).outlined(s.outline_variant)
    };
    /// An outlined toggle button.
    OUTLINED_TOGGLE = "OutlinedToggleButton", outlined_toggle => |s, on| if on {
        Paint::new(s.inverse_surface, s.inverse_on_surface)
    } else {
        Paint::new(aegle_ui::Color::TRANSPARENT, s.on_surface_variant).outlined(s.outline_variant)
    };
    /// A text button.
    TEXT_BUTTON = "TextButton", text_button => |s, _on| Paint::new(aegle_ui::Color::TRANSPARENT, s.primary);
}

handle! {
    /// A common button: a label, optionally with a leading icon.
    pub Button(PressableControl): text, interactive, pressed, indicator
}

pressable_methods!(Button);

impl Button {
    /// Appends a small, round button of `style` to `parent`.
    pub fn new(parent: &Container, style: ButtonStyle, text: &str) -> Result<Self> {
        let look = Look::Button(ButtonLook {
            style,
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            split: false,
        });
        pressable::add(parent, |fonts, theme| {
            PressableControl::new(fonts, theme, look, text)
        })
        .map(Self)
    }

    pub(crate) fn look(&self) -> Result<ButtonLook> {
        self.read(|c| match c.look {
            Look::Button(look) => look,
            _ => unreachable!("a Button handle holds a button"),
        })
    }

    pub(crate) fn relook(&self, change: impl FnOnce(&mut ButtonLook)) -> Result {
        let mut look = self.look()?;
        change(&mut look);
        pressable::relook(self, |c| c.set_look(Look::Button(look)))
    }

    /// Changes the color style.
    pub fn set_style(&self, style: ButtonStyle) -> Result {
        self.relook(|look| look.style = style)
    }
    /// Changes the size, with its height, padding, icon and type role.
    pub fn set_size(&self, size: ButtonSize) -> Result {
        self.relook(|look| look.size = size)
    }
    /// Changes the resting shape.
    pub fn set_shape(&self, shape: ButtonShape) -> Result {
        self.relook(|look| look.shape = shape)
    }
    /// The color style.
    pub fn style(&self) -> Result<ButtonStyle> {
        Ok(self.look()?.style)
    }
    /// The size.
    pub fn size(&self) -> Result<ButtonSize> {
        Ok(self.look()?.size)
    }
}
