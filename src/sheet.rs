//! Sheets: supplementary content anchored to the bottom or the end of the
//! window. A modal sheet opens over a scrim and closes on Escape or a click
//! outside it; a standard one sits in the layout beside the main content.
//! Bottom sheets round their top corners by 28 dp and show a drag handle;
//! modal side sheets round their inner corners by 16 dp.

use aegle_layout::Style;
use aegle_ui::{
    Align, Container, ControlKind, Length, Node, Point, Result, Size,
    control::{Control, MeasureCx, PaintCx},
    scene::{Rect, RoundedRect},
};

use crate::{
    IconButton, IconStyle, Role, Scheme, Text, icons,
    overlay::{Enter, Layer, Place},
    surface::{self, Part, SHEET, STANDARD_SHEET, SurfaceControl},
    tokens::typescale,
};

/// The drag handle at the top of a bottom sheet: 32 × 4 dp in a 48 dp
/// tall touch area.
struct Handle;

impl Control for Handle {
    fn kind(&self) -> &'static ControlKind {
        &crate::glyph::ICON
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(Size::new(48.0, 36.0))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let rect = Rect::new((cx.size.width - 32.0) / 2.0, 16.0, 32.0, 4.0);
        let color = Scheme::of(cx.theme).on_surface_variant;
        cx.builder.fill(RoundedRect::new(rect, 2.0)?, color)?;
        Ok(())
    }
}

/// A bottom or side sheet.
#[derive(Clone)]
pub struct Sheet {
    layer: Option<Layer>,
    surface: Container,
    content: Container,
    enter: Enter,
}

impl std::ops::Deref for Sheet {
    type Target = Container;
    /// The sheet's content.
    fn deref(&self) -> &Container {
        &self.content
    }
}

impl Sheet {
    /// Creates a hidden modal bottom sheet in the theme of `owner`, up to
    /// 640 dp wide.
    pub fn bottom(owner: &Node) -> Result<Self> {
        let layer = Layer::new(owner, Place::Bottom, true, 0.0)?;
        let control = SurfaceControl::new(&SHEET, Part::Sheet, 28.0, 1.0);
        Self::bottom_in(&layer.host.clone(), control, Some(layer))
    }

    /// Appends a standard bottom sheet to `parent`, shown.
    pub fn standard_bottom(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&SHEET, Part::Pane, 28.0, 1.0);
        Self::bottom_in(parent, control, None)
    }

    fn bottom_in(
        parent: &Container,
        mut control: SurfaceControl,
        layer: Option<Layer>,
    ) -> Result<Self> {
        control.corners = [28.0, 28.0, 0.0, 0.0];
        let surface = surface::add(parent, control, 0.0)?;
        surface.set_width(Length::Percent(100.0))?;
        surface.set_max_width(640.0)?;
        surface.set_align_items(Some(Align::Stretch))?;
        surface.add(|_, _| {
            let style = Style {
                flex_shrink: 0.0,
                align_self: Some(Align::Center.items()),
                ..Default::default()
            };
            Ok((Box::new(Handle) as Box<dyn Control>, style))
        })?;
        let content = surface.column()?;
        content.set_padding(crate::edges(16.0, 0.0, 16.0, 24.0))?;
        Self {
            layer,
            surface,
            content,
            enter: Enter::Slide(Point::new(0.0, 160.0)),
        }
        .dismissible()
    }

    /// Lets Escape and the scrim close a modal sheet.
    fn dismissible(self) -> Result<Self> {
        if let Some(layer) = &self.layer {
            let closing = self.clone();
            layer.on_dismiss(move || closing.close())?;
        }
        Ok(self)
    }

    /// Creates a hidden modal side sheet at the end of the window, 360 dp
    /// wide, headed by `headline` and a close button.
    pub fn side(owner: &Node, headline: &str) -> Result<Self> {
        let layer = Layer::new(owner, Place::End, true, 0.0)?;
        let mut control = SurfaceControl::new(&SHEET, Part::Sheet, 16.0, 1.0);
        control.corners = [16.0, 0.0, 0.0, 16.0];
        Self::side_in(&layer.host.clone(), control, headline, Some(layer))
    }

    /// Appends a standard side sheet to `parent`, shown, for content beside
    /// the main region.
    pub fn standard_side(parent: &Container, headline: &str) -> Result<Self> {
        let control = SurfaceControl::new(&STANDARD_SHEET, Part::Pane, 0.0, 0.0);
        Self::side_in(parent, control, headline, None)
    }

    fn side_in(
        parent: &Container,
        control: SurfaceControl,
        headline: &str,
        layer: Option<Layer>,
    ) -> Result<Self> {
        let surface = surface::add(parent, control, 0.0)?;
        surface.set_size(360.0, Length::Percent(100.0))?;
        let header = surface.row()?;
        header.set_align_items(Some(Align::Center))?;
        header.set_padding(crate::edges(24.0, 12.0, 12.0, 4.0))?;
        Text::new(
            &header,
            typescale::TITLE_LARGE,
            Role::on_surface_variant,
            headline,
        )?
        .set_grow(1.0)?;
        let content = surface.column()?;
        content.set_padding(crate::edges(24.0, 8.0, 24.0, 24.0))?;
        content.set_grow(1.0)?;
        let sheet = Self {
            layer,
            surface,
            content,
            enter: Enter::Slide(Point::new(120.0, 0.0)),
        };
        let close = IconButton::new(&header, IconStyle::Standard, icons::close(), "Close")?;
        let closing = sheet.clone();
        close.on_click(move |_| closing.close())?;
        sheet.dismissible()
    }

    /// The sheet's surface, for sizing it.
    pub fn surface(&self) -> &Container {
        &self.surface
    }

    /// Shows a modal sheet above everything, or a standard one in place.
    pub fn show(&self) -> Result {
        match &self.layer {
            Some(layer) => layer.show(&self.surface, None, self.enter),
            None => self.surface.set_visible(true),
        }
    }

    /// Closes a modal sheet, or hides a standard one.
    pub fn close(&self) -> Result {
        match &self.layer {
            Some(layer) => layer.dismiss(),
            None => self.surface.set_visible(false),
        }
    }

    /// Whether it is shown.
    pub fn is_open(&self) -> Result<bool> {
        match &self.layer {
            Some(layer) => layer.is_open(),
            None => self
                .surface
                .change(|state, id| Ok(state.tree.get(id).unwrap().context.visible)),
        }
    }

    /// Adds a handler run after Escape or a click outside closes a modal
    /// sheet.
    pub fn on_dismiss(&self, callback: impl FnMut() -> Result + 'static) -> Result {
        match &self.layer {
            Some(layer) => layer.on_dismiss(callback),
            None => Ok(()),
        }
    }
}
