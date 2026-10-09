//! Chips: compact elements for actions, filters, entered items and
//! suggestions, 32 dp high with small corners, flat (outlined) or elevated.

use aegle_ui::{Color, Container, ControlKind, Result, handle};

use crate::{
    Icon, icons,
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
    tokens::{shape, typescale},
};

/// What a chip is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipKind {
    /// A smart or automated action, with a primary-colored icon.
    Assist,
    /// A toggle that filters content, with a check when selected.
    Filter,
    /// An entered item, such as a contact, removable by its trailing icon.
    Input,
    /// A dynamically suggested reply or query.
    Suggestion,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChipLook {
    pub kind: ChipKind,
    pub elevated: bool,
}

impl ChipLook {
    pub fn kind(self) -> &'static ControlKind {
        match (self.kind, self.elevated) {
            (ChipKind::Assist, false) => &ASSIST_CHIP,
            (ChipKind::Assist, true) => &ELEVATED_ASSIST_CHIP,
            (ChipKind::Filter, false) => &FILTER_CHIP,
            (ChipKind::Filter, true) => &ELEVATED_FILTER_CHIP,
            (ChipKind::Input, _) => &INPUT_CHIP,
            (ChipKind::Suggestion, false) => &SUGGESTION_CHIP,
            (ChipKind::Suggestion, true) => &ELEVATED_SUGGESTION_CHIP,
        }
    }

    pub fn spec(self) -> Spec {
        Spec {
            height: 32.0,
            width: None,
            min_width: 32.0,
            padding: 16.0,
            icon_padding: 8.0,
            icon: 18.0,
            gap: 8.0,
            text: typescale::LABEL_LARGE,
            corners: [shape::SMALL; 3],
            inner: [shape::SMALL; 3],
            hover_inner: false,
            outline: 1.0,
            elevation: if self.elevated {
                [1.0, 2.0]
            } else {
                [0.0, 1.0]
            },
        }
    }
}

const CLEAR: Color = Color::TRANSPARENT;

kinds! {
    /// An outlined assist chip.
    ASSIST_CHIP = "AssistChip", assist => |s, _on| {
        Paint::new(CLEAR, s.on_surface).icon(s.primary).outlined(s.outline_variant)
    };
    /// An elevated assist chip.
    ELEVATED_ASSIST_CHIP = "ElevatedAssistChip", elevated_assist => |s, _on| {
        Paint::new(s.surface_container_low, s.on_surface).icon(s.primary)
    };
    /// An outlined filter chip.
    FILTER_CHIP = "FilterChip", filter => |s, on| if on {
        Paint::new(s.secondary_container, s.on_secondary_container)
    } else {
        Paint::new(CLEAR, s.on_surface_variant).icon(s.primary).outlined(s.outline_variant)
    };
    /// An elevated filter chip.
    ELEVATED_FILTER_CHIP = "ElevatedFilterChip", elevated_filter => |s, on| if on {
        Paint::new(s.secondary_container, s.on_secondary_container)
    } else {
        Paint::new(s.surface_container_low, s.on_surface_variant).icon(s.primary)
    };
    /// An input chip.
    INPUT_CHIP = "InputChip", input => |s, on| if on {
        Paint::new(s.secondary_container, s.on_secondary_container)
    } else {
        Paint::new(CLEAR, s.on_surface_variant).outlined(s.outline_variant)
    };
    /// An outlined suggestion chip.
    SUGGESTION_CHIP = "SuggestionChip", suggestion => |s, _on| {
        Paint::new(CLEAR, s.on_surface_variant).icon(s.primary).outlined(s.outline_variant)
    };
    /// An elevated suggestion chip.
    ELEVATED_SUGGESTION_CHIP = "ElevatedSuggestionChip", elevated_suggestion => |s, _on| {
        Paint::new(s.surface_container_low, s.on_surface_variant).icon(s.primary)
    };
}

handle! {
    /// A chip of any [`ChipKind`].
    pub Chip(PressableControl): text, interactive, pressed, indicator
}

pressable_methods!(Chip);

impl Chip {
    /// Appends a flat chip of `kind`. Filter chips start unselected; input
    /// chips show a remove icon.
    pub fn new(parent: &Container, kind: ChipKind, text: &str) -> Result<Self> {
        let look = Look::Chip(ChipLook {
            kind,
            elevated: false,
        });
        pressable::add(parent, |fonts, theme| {
            let mut control = PressableControl::new(fonts, theme, look, text)?;
            match kind {
                ChipKind::Filter => {
                    control.selected = Some(false);
                    control.selected_icon = Some(icons::check());
                }
                ChipKind::Input => {
                    control.trailing = Some(icons::close());
                    control.removable = true;
                }
                _ => {}
            }
            Ok(control)
        })
        .map(Self)
    }

    /// Raises a chip onto a tonal surface with a shadow, for chips over
    /// images or other varied backgrounds. Input chips stay flat.
    pub fn set_elevated(&self, elevated: bool) -> Result {
        pressable::relook(self, |c| {
            if let Look::Chip(mut look) = c.look {
                look.elevated = elevated && look.kind != ChipKind::Input;
                c.set_look(Look::Chip(look));
            }
        })
    }

    /// Shows an avatar or icon before the label.
    pub fn set_leading(&self, icon: Option<Icon>) -> Result {
        self.set_icon(icon)
    }

    /// Adds a handler run when an input chip's remove icon is activated, or
    /// Delete or Backspace is pressed on it. Removing the chip is up to the
    /// handler.
    pub fn on_remove(&self, mut callback: impl FnMut(Chip) -> Result + 'static) -> Result {
        self.on_click_any(move |chip| {
            if chip.read(|c| c.removing)? {
                callback(chip)?;
            }
            Ok(())
        })
    }
}
