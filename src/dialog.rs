//! Dialogs: a modal 28 dp rounded surface over a scrim with an optional
//! icon, a headline, supporting content and text-button actions at its end.
//! Escape and a click on the scrim dismiss it; Tab stays inside while open,
//! and focus returns to where it was when it closes.

use aegle_ui::{Align, Container, Justify, Length, Node, Result};

use crate::{
    Button, ButtonStyle, Icon, IconView, Role, Text,
    overlay::{Enter, Layer, Place},
    surface::{self, DIALOG, Part, SurfaceControl},
    tokens::typescale,
};

/// A basic or full-screen dialog.
#[derive(Clone)]
pub struct Dialog {
    layer: Layer,
    surface: Container,
    content: Container,
    actions: Container,
    headline: Text,
}

impl std::ops::Deref for Dialog {
    type Target = Container;
    /// The content between the headline and the actions.
    fn deref(&self) -> &Container {
        &self.content
    }
}

impl Dialog {
    /// Creates a hidden dialog in the theme of `owner`, headed by `icon`
    /// (centered, in the secondary color) and `headline`.
    pub fn new(owner: &Node, icon: Option<Icon>, headline: &str) -> Result<Self> {
        let layer = Layer::new(owner, Place::Center, true, 24.0)?;
        let control = SurfaceControl::new(&DIALOG, Part::Dialog, 28.0, 0.0);
        let surface = surface::add(&layer.host, control, 24.0)?;
        surface.set_gap(16.0)?;
        surface.set_min_width(280.0)?;
        surface.set_max_width(560.0)?;
        if let Some(icon) = icon {
            IconView::new(&surface, icon, 24.0, Role::secondary)?
                .set_align_self(Some(Align::Center))?;
        }
        let title = Text::new(
            &surface,
            typescale::HEADLINE_SMALL,
            Role::on_surface,
            headline,
        )?;
        if icon_centered(&surface)? {
            title.set_align_self(Some(Align::Center))?;
        }
        let content = surface.column()?;
        content.set_gap(8.0)?;
        let actions = surface.row()?;
        actions.set_gap(8.0)?;
        actions.set_justify_content(Some(Justify::End))?;
        actions.set_margin(crate::edges(0.0, 8.0, 0.0, 0.0))?;
        actions.set_visible(false)?;
        let dialog = Self {
            layer,
            surface,
            content,
            actions,
            headline: title,
        };
        let closing = dialog.layer.clone();
        dialog.layer.on_dismiss(move || closing.dismiss())?;
        Ok(dialog)
    }

    /// Appends supporting text in body medium.
    pub fn supporting(&self, text: &str) -> Result<Text> {
        Text::new(
            &self.content,
            typescale::BODY_MEDIUM,
            Role::on_surface_variant,
            text,
        )
    }

    /// Appends a text-button action at the end; actions do not close the
    /// dialog themselves.
    pub fn action(&self, text: &str) -> Result<Button> {
        self.actions.set_visible(true)?;
        Button::new(&self.actions, ButtonStyle::Text, text)
    }

    /// The headline.
    pub fn headline(&self) -> &Text {
        &self.headline
    }

    /// The dialog's surface, for sizing it.
    pub fn surface(&self) -> &Container {
        &self.surface
    }

    /// Fills the window with square corners, the full-screen dialog of
    /// compact windows, or returns to a basic dialog.
    pub fn set_full_screen(&self, full: bool) -> Result {
        let (padding, radius) = if full { (0.0, 0.0) } else { (24.0, 28.0) };
        self.layer.host.set_padding(padding)?;
        let size = if full {
            Length::Percent(100.0)
        } else {
            Length::Auto
        };
        self.surface.set_size(size, size)?;
        self.surface.set_max_width(if full {
            Length::Auto
        } else {
            Length::Px(560.0)
        })?;
        self.surface.change(|state, id| {
            let control = state
                .control_as::<SurfaceControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?;
            control.corners = [radius; 4];
            state.tree.mark_dirty(id, aegle_ui::Dirty::PAINT)?;
            Ok(())
        })
    }

    /// Shows the dialog above everything, focusing its first control.
    pub fn show(&self) -> Result {
        self.layer.show(&self.surface, None, Enter::Scale)
    }

    /// Closes the dialog, returning focus to where it was.
    pub fn close(&self) -> Result {
        self.layer.dismiss()
    }

    /// Whether it is open.
    pub fn is_open(&self) -> Result<bool> {
        self.layer.is_open()
    }

    /// Adds a handler run after Escape or a click on the scrim closes it.
    pub fn on_dismiss(&self, callback: impl FnMut() -> Result + 'static) -> Result {
        self.layer.on_dismiss(callback)
    }

    /// Removes the dialog.
    pub fn remove(&self) -> Result {
        self.layer.host.remove()
    }
}

/// Whether the surface starts with an icon, which centers the headline.
fn icon_centered(surface: &Container) -> Result<bool> {
    surface.change(|state, id| {
        let first = state.tree.children(id)?.next();
        Ok(first.is_some_and(|c| state.control_as::<crate::glyph::IconControl>(c).is_some()))
    })
}
