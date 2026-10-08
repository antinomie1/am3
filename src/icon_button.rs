//! Icon buttons: one icon in four color styles, five sizes and three
//! widths, plain or toggle.

use aegle_ui::{Color, Container, ControlKind, Result, handle};

use crate::{
    ButtonShape, ButtonSize, Icon,
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
    tokens::typescale,
};

/// An icon button's color style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconStyle {
    /// Only the icon, for the lowest emphasis.
    #[default]
    Standard,
    /// A primary container.
    Filled,
    /// A secondary container.
    Tonal,
    /// An outline around the icon.
    Outlined,
}

/// An icon button's width relative to its height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconWidth {
    /// Narrower than high, for dense rows.
    Narrow,
    /// Square.
    #[default]
    Default,
    /// Wider than high, for emphasis.
    Wide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IconLook {
    pub style: IconStyle,
    pub size: ButtonSize,
    pub shape: ButtonShape,
    pub width: IconWidth,
    /// The menu part of a split button: its colors stay when selected.
    pub split: bool,
}

impl IconLook {
    pub fn kind(self, toggle: bool) -> &'static ControlKind {
        match (self.style, toggle && !self.split) {
            (IconStyle::Standard, false) => &STANDARD_ICON_BUTTON,
            (IconStyle::Standard, true) => &STANDARD_ICON_TOGGLE,
            (IconStyle::Filled, false) => &FILLED_ICON_BUTTON,
            (IconStyle::Filled, true) => &FILLED_ICON_TOGGLE,
            (IconStyle::Tonal, false) => &TONAL_ICON_BUTTON,
            (IconStyle::Tonal, true) => &TONAL_ICON_TOGGLE,
            (IconStyle::Outlined, false) => &OUTLINED_ICON_BUTTON,
            (IconStyle::Outlined, true) => &OUTLINED_ICON_TOGGLE,
        }
    }

    pub fn spec(self) -> Spec {
        let (height, icon, _, _, outline) = self.size.metrics();
        // Narrow, default and wide widths of each size.
        let widths = match self.size {
            ButtonSize::ExtraSmall => [28.0, 32.0, 40.0],
            ButtonSize::Small => [32.0, 40.0, 52.0],
            ButtonSize::Medium => [48.0, 56.0, 72.0],
            ButtonSize::Large => [64.0, 96.0, 128.0],
            ButtonSize::ExtraLarge => [104.0, 136.0, 184.0],
        };
        let width = widths[self.width as usize];
        let [corner, pressed, selected] = self.size.corners(self.shape);
        Spec {
            height,
            width: Some(width),
            min_width: width,
            padding: 0.0,
            icon,
            gap: 0.0,
            text: typescale::LABEL_LARGE,
            corners: [corner, pressed, selected],
            inner: self.size.inner(self.split),
            hover_inner: self.split,
            outline,
            elevation: [0.0, 0.0],
        }
    }
}

const CLEAR: Color = Color::TRANSPARENT;

kinds! {
    /// A standard icon button.
    STANDARD_ICON_BUTTON = "IconButton", standard => |s, _on| Paint::new(CLEAR, s.on_surface_variant);
    /// A standard icon toggle button.
    STANDARD_ICON_TOGGLE = "IconToggleButton", standard_toggle => |s, on| {
        Paint::new(CLEAR, if on { s.primary } else { s.on_surface_variant })
    };
    /// A filled icon button.
    FILLED_ICON_BUTTON = "FilledIconButton", filled => |s, _on| Paint::new(s.primary, s.on_primary);
    /// A filled icon toggle button.
    FILLED_ICON_TOGGLE = "FilledIconToggleButton", filled_toggle => |s, on| if on {
        Paint::new(s.primary, s.on_primary)
    } else {
        Paint::new(s.surface_container, s.on_surface_variant)
    };
    /// A tonal icon button.
    TONAL_ICON_BUTTON = "TonalIconButton", tonal => |s, _on| Paint::new(s.secondary_container, s.on_secondary_container);
    /// A tonal icon toggle button.
    TONAL_ICON_TOGGLE = "TonalIconToggleButton", tonal_toggle => |s, on| if on {
        Paint::new(s.secondary, s.on_secondary)
    } else {
        Paint::new(s.secondary_container, s.on_secondary_container)
    };
    /// An outlined icon button.
    OUTLINED_ICON_BUTTON = "OutlinedIconButton", outlined => |s, _on| {
        Paint::new(CLEAR, s.on_surface_variant).outlined(s.outline_variant)
    };
    /// An outlined icon toggle button.
    OUTLINED_ICON_TOGGLE = "OutlinedIconToggleButton", outlined_toggle => |s, on| if on {
        Paint::new(s.inverse_surface, s.inverse_on_surface)
    } else {
        Paint::new(CLEAR, s.on_surface_variant).outlined(s.outline_variant)
    };
}

handle! {
    /// An icon button. Its label is not shown; it names the button to
    /// assistive technology.
    pub IconButton(PressableControl): interactive, pressed, indicator
}

pressable_methods!(IconButton);

impl IconButton {
    /// Appends a small standard icon button showing `icon`, named `label`.
    pub fn new(parent: &Container, style: IconStyle, icon: Icon, label: &str) -> Result<Self> {
        let look = Look::Icon(IconLook {
            style,
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            width: IconWidth::default(),
            split: false,
        });
        pressable::add(parent, |fonts, theme| {
            let mut control = PressableControl::new(fonts, theme, look, label)?;
            control.icon = Some(icon);
            control.hide_label = true;
            Ok(control)
        })
        .map(Self)
    }

    fn relook(&self, change: impl FnOnce(&mut IconLook)) -> Result {
        let mut look = self.read(|c| match c.look {
            Look::Icon(look) => look,
            _ => unreachable!("an IconButton handle holds an icon button"),
        })?;
        change(&mut look);
        pressable::relook(self, |c| c.set_look(Look::Icon(look)))
    }

    /// Changes the color style.
    pub fn set_style(&self, style: IconStyle) -> Result {
        self.relook(|look| look.style = style)
    }
    /// Changes the size.
    pub fn set_size(&self, size: ButtonSize) -> Result {
        self.relook(|look| look.size = size)
    }
    /// Changes the resting shape.
    pub fn set_shape(&self, shape: ButtonShape) -> Result {
        self.relook(|look| look.shape = shape)
    }
    /// Changes the width.
    pub fn set_width(&self, width: IconWidth) -> Result {
        self.relook(|look| look.width = width)
    }
}
