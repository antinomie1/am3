//! Floating action buttons: the FAB in three sizes and six colors, and the
//! extended FAB that adds a label and collapses back to a FAB.

use aegle_ui::{Container, ControlKind, Result, handle};

use crate::{
    Icon,
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
    tokens::{shape, typescale},
};

/// A FAB's color set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabColor {
    /// Primary container, the default.
    #[default]
    PrimaryContainer,
    /// Secondary container.
    SecondaryContainer,
    /// Tertiary container.
    TertiaryContainer,
    /// Primary, for the strongest emphasis.
    Primary,
    /// Secondary.
    Secondary,
    /// Tertiary.
    Tertiary,
}

/// A FAB size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabSize {
    /// 56 dp.
    #[default]
    Regular,
    /// 80 dp.
    Medium,
    /// 96 dp.
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FabLook {
    pub color: FabColor,
    pub size: FabSize,
    pub extended: bool,
}

impl FabLook {
    pub fn kind(self) -> &'static ControlKind {
        match self.color {
            FabColor::PrimaryContainer => &FAB_PRIMARY_CONTAINER,
            FabColor::SecondaryContainer => &FAB_SECONDARY_CONTAINER,
            FabColor::TertiaryContainer => &FAB_TERTIARY_CONTAINER,
            FabColor::Primary => &FAB_PRIMARY,
            FabColor::Secondary => &FAB_SECONDARY,
            FabColor::Tertiary => &FAB_TERTIARY,
        }
    }

    pub fn spec(self) -> Spec {
        let (height, icon, corner, padding, gap, text) = match self.size {
            FabSize::Regular => (56.0, 24.0, shape::LARGE, 16.0, 8.0, typescale::TITLE_MEDIUM),
            FabSize::Medium => (
                80.0,
                28.0,
                shape::LARGE_INCREASED,
                26.0,
                12.0,
                typescale::TITLE_LARGE,
            ),
            FabSize::Large => (
                96.0,
                36.0,
                shape::EXTRA_LARGE,
                28.0,
                16.0,
                typescale::HEADLINE_SMALL,
            ),
        };
        Spec {
            height,
            width: (!self.extended).then_some(height),
            min_width: height,
            padding,
            icon,
            gap,
            text,
            corners: [corner, corner, shape::FULL],
            inner: [corner; 3],
            hover_inner: false,
            outline: 1.0,
            elevation: [3.0, 4.0],
        }
    }
}

// A selected FAB is the close button of an open FAB menu, in the strong
// color of its set.
kinds! {
    /// A FAB in primary container colors.
    FAB_PRIMARY_CONTAINER = "Fab", primary_container => |s, on| if on {
        Paint::new(s.primary, s.on_primary)
    } else {
        Paint::new(s.primary_container, s.on_primary_container)
    };
    /// A FAB in secondary container colors.
    FAB_SECONDARY_CONTAINER = "SecondaryContainerFab", secondary_container => |s, on| if on {
        Paint::new(s.secondary, s.on_secondary)
    } else {
        Paint::new(s.secondary_container, s.on_secondary_container)
    };
    /// A FAB in tertiary container colors.
    FAB_TERTIARY_CONTAINER = "TertiaryContainerFab", tertiary_container => |s, on| if on {
        Paint::new(s.tertiary, s.on_tertiary)
    } else {
        Paint::new(s.tertiary_container, s.on_tertiary_container)
    };
    /// A FAB in primary colors.
    FAB_PRIMARY = "PrimaryFab", primary => |s, _on| Paint::new(s.primary, s.on_primary);
    /// A FAB in secondary colors.
    FAB_SECONDARY = "SecondaryFab", secondary => |s, _on| Paint::new(s.secondary, s.on_secondary);
    /// A FAB in tertiary colors.
    FAB_TERTIARY = "TertiaryFab", tertiary => |s, _on| Paint::new(s.tertiary, s.on_tertiary);
}

handle! {
    /// A floating action button, the screen's primary action; extended, it
    /// shows its label beside the icon.
    pub Fab(PressableControl): text, interactive, pressed, indicator
}

pressable_methods!(Fab);

impl Fab {
    /// Appends a regular FAB showing `icon`. The label names it, and an
    /// extended FAB shows it.
    pub fn new(parent: &Container, icon: Icon, label: &str) -> Result<Self> {
        Self::create(parent, icon, label, false)
    }

    /// Appends an extended FAB showing `icon` and `label`.
    pub fn extended(parent: &Container, icon: Icon, label: &str) -> Result<Self> {
        Self::create(parent, icon, label, true)
    }

    fn create(parent: &Container, icon: Icon, label: &str, extended: bool) -> Result<Self> {
        let look = Look::Fab(FabLook {
            color: FabColor::default(),
            size: FabSize::default(),
            extended,
        });
        pressable::add(parent, |fonts, theme| {
            let mut control = PressableControl::new(fonts, theme, look, label)?;
            control.icon = Some(icon);
            control.hide_label = !extended;
            Ok(control)
        })
        .map(Self)
    }

    pub(crate) fn relook(&self, change: impl FnOnce(&mut FabLook)) -> Result {
        let mut look = self.read(|c| match c.look {
            Look::Fab(look) => look,
            _ => unreachable!("a Fab handle holds a FAB"),
        })?;
        change(&mut look);
        pressable::relook(self, |c| {
            c.hide_label = !look.extended;
            c.set_look(Look::Fab(look));
        })
    }

    /// Changes the color set.
    pub fn set_color(&self, color: FabColor) -> Result {
        self.relook(|look| look.color = color)
    }
    /// Changes the size.
    pub fn set_size(&self, size: FabSize) -> Result {
        self.relook(|look| look.size = size)
    }
    /// Extends it to show the label, or collapses it to the icon.
    pub fn set_extended(&self, extended: bool) -> Result {
        self.relook(|look| look.extended = extended)
    }
}
