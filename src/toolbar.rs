//! Toolbars (M3 Expressive): a docked toolbar spans the bottom of the
//! window; a floating toolbar is a 64 dp pill, horizontal or vertical,
//! raised over content. Both come in the standard surface-container colors
//! or the vibrant primary-container colors.

use aegle_ui::{Align, Appearance, Color, Container, Justify, Result, Theme, VisualState};

use crate::{
    Button, ButtonStyle, Icon, IconButton, IconStyle, Scheme,
    skin::{self, Paint, kinds},
    surface::{self, Part, SurfaceControl},
};

kinds! { container
    /// A toolbar in the standard colors.
    TOOLBAR = "Toolbar", standard => |s, _on| Paint::new(s.surface_container, s.on_surface);
    /// A toolbar in the vibrant colors.
    VIBRANT_TOOLBAR = "VibrantToolbar", vibrant => |s, _on| {
        Paint::new(s.primary_container, s.on_primary_container)
    };
}

/// Standard icon buttons and text buttons on a vibrant toolbar.
fn vibrant_content(theme: &Theme, state: VisualState) -> Appearance {
    let s = &Scheme::of(theme);
    skin::pressable(
        theme,
        state,
        s,
        Paint::new(Color::TRANSPARENT, s.on_primary_container),
    )
}

/// The toolbar colors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarColor {
    /// On the surface container.
    #[default]
    Standard,
    /// On the primary container, for emphasis.
    Vibrant,
}

/// A docked or floating toolbar.
#[derive(Clone)]
pub struct Toolbar(Container);

impl std::ops::Deref for Toolbar {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.0
    }
}

impl Toolbar {
    /// Appends a 64 dp docked toolbar spanning its parent, its actions
    /// spread evenly.
    pub fn docked(parent: &Container, color: ToolbarColor) -> Result<Self> {
        let bar = Self::create(parent, color, 0.0, 0.0)?;
        bar.set_direction(aegle_ui::Direction::Row)?;
        bar.set_align_self(Some(Align::Stretch))?;
        bar.set_height(64.0)?;
        bar.set_justify_content(Some(Justify::SpaceEvenly))?;
        bar.set_padding(crate::edges(16.0, 0.0, 16.0, 0.0))?;
        Ok(bar)
    }

    /// Appends a floating pill toolbar, down a column if `vertical`.
    pub fn floating(parent: &Container, color: ToolbarColor, vertical: bool) -> Result<Self> {
        let bar = Self::create(parent, color, 32.0, 3.0)?;
        if !vertical {
            bar.set_direction(aegle_ui::Direction::Row)?;
            bar.set_height(64.0)?;
        } else {
            bar.set_width(64.0)?;
        }
        bar.set_gap(4.0)?;
        bar.set_padding(8.0)?;
        Ok(bar)
    }

    fn create(parent: &Container, color: ToolbarColor, radius: f32, level: f32) -> Result<Self> {
        let kind = match color {
            ToolbarColor::Standard => &TOOLBAR,
            ToolbarColor::Vibrant => &VIBRANT_TOOLBAR,
        };
        let bar = surface::add(
            parent,
            SurfaceControl::new(kind, Part::Pane, radius, level),
            0.0,
        )?;
        bar.set_align_items(Some(Align::Center))?;
        if color == ToolbarColor::Vibrant {
            bar.set_kind_skin(&crate::kinds::STANDARD_ICON_BUTTON, Some(vibrant_content))?;
            bar.set_kind_skin(&crate::kinds::TEXT_BUTTON, Some(vibrant_content))?;
        }
        Ok(Self(bar))
    }

    /// Appends a standard icon button.
    pub fn icon_button(&self, icon: Icon, label: &str) -> Result<IconButton> {
        IconButton::new(&self.0, IconStyle::Standard, icon, label)
    }

    /// Appends a button.
    pub fn button(&self, style: ButtonStyle, text: &str) -> Result<Button> {
        Button::new(&self.0, style, text)
    }
}
