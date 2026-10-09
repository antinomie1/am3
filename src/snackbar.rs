//! Snackbars: a brief message at the bottom of the window, with an optional
//! action and close button, on the inverse surface. They do not block input,
//! and close after a timeout, through their action or close button, or when
//! the application closes them.

use std::time::Duration;

use aegle_ui::{Align, Appearance, Color, Container, Node, Result, Theme, VisualState};

use crate::{
    Button, ButtonStyle, IconButton, IconStyle, Role, Scheme, Text, icons,
    overlay::{Enter, Layer, Place},
    skin::{self, Paint},
    surface::{self, Part, SNACKBAR, SurfaceControl},
    tokens::typescale,
};

/// How long a snackbar with no action shows.
pub const SHORT: Duration = Duration::from_secs(4);
/// How long a snackbar with an action shows.
pub const LONG: Duration = Duration::from_secs(10);

fn action_skin(theme: &Theme, state: VisualState) -> Appearance {
    let s = &Scheme::of(theme);
    skin::pressable(
        theme,
        state,
        s,
        Paint::new(Color::TRANSPARENT, s.inverse_primary),
    )
}

fn close_skin(theme: &Theme, state: VisualState) -> Appearance {
    let s = &Scheme::of(theme);
    skin::pressable(
        theme,
        state,
        s,
        Paint::new(Color::TRANSPARENT, s.inverse_on_surface),
    )
}

/// A brief message with an optional action.
#[derive(Clone)]
pub struct Snackbar {
    layer: Layer,
    surface: Container,
    text: Text,
}

impl std::ops::Deref for Snackbar {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.surface
    }
}

impl Snackbar {
    /// Creates a hidden snackbar showing `text`, in the theme of `owner`.
    pub fn new(owner: &Node, text: &str) -> Result<Self> {
        let layer = Layer::new(owner, Place::Bottom, false, 16.0)?;
        let control = SurfaceControl::new(&SNACKBAR, Part::Snackbar, 4.0, 3.0);
        let surface = surface::add(&layer.host, control, 0.0)?;
        surface.set_direction(aegle_ui::Direction::Row)?;
        surface.set_align_items(Some(Align::Center))?;
        surface.set_min_height(48.0)?;
        surface.set_max_width(600.0)?;
        surface.set_gap(4.0)?;
        surface.set_padding(crate::edges(16.0, 0.0, 8.0, 0.0))?;
        surface.set_kind_skin(&crate::kinds::TEXT_BUTTON, Some(action_skin))?;
        surface.set_kind_skin(&crate::kinds::STANDARD_ICON_BUTTON, Some(close_skin))?;
        let text = Text::new(
            &surface,
            typescale::BODY_MEDIUM,
            Role::inverse_on_surface,
            text,
        )?;
        text.set_grow(1.0)?;
        text.set_shrink(1.0)?;
        text.set_margin(crate::edges(0.0, 14.0, 8.0, 14.0))?;
        let snackbar = Self {
            layer,
            surface,
            text,
        };
        let closing = snackbar.layer.clone();
        snackbar.layer.on_dismiss(move || closing.dismiss())?;
        Ok(snackbar)
    }

    /// Replaces the message.
    pub fn set_text(&self, text: &str) -> Result {
        self.text.set_text(text)
    }

    /// Appends an action in the inverse primary color; activating it also
    /// closes the snackbar.
    pub fn action(&self, text: &str) -> Result<Button> {
        let button = Button::new(&self.surface, ButtonStyle::Text, text)?;
        let layer = self.layer.clone();
        button.on_click(move |_| layer.dismiss())?;
        Ok(button)
    }

    /// Appends a close button.
    pub fn closable(&self) -> Result<IconButton> {
        let button = IconButton::new(&self.surface, IconStyle::Standard, icons::close(), "Close")?;
        let layer = self.layer.clone();
        button.on_click(move |_| layer.dismiss())?;
        Ok(button)
    }

    /// The snackbar's surface.
    pub fn surface(&self) -> &Container {
        &self.surface
    }

    /// Shows it above the content for `duration` ([`SHORT`] or [`LONG`]),
    /// or until closed for `None`.
    pub fn show(&self, duration: Option<Duration>) -> Result {
        let rise = Enter::Slide(aegle_ui::Point::new(0.0, 24.0));
        self.layer.show(&self.surface, duration, rise)
    }

    /// Closes it.
    pub fn close(&self) -> Result {
        self.layer.dismiss()
    }

    /// Whether it is shown.
    pub fn is_open(&self) -> Result<bool> {
        self.layer.is_open()
    }

    /// Adds a handler run after the timeout closes it.
    pub fn on_timeout(&self, callback: impl FnMut() -> Result + 'static) -> Result {
        self.layer.on_dismiss(callback)
    }
}
