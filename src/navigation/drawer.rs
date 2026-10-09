//! The navigation drawer: destinations in 56 dp pills under optional
//! section headlines, 360 dp wide. A standard drawer sits in the layout; a
//! modal one slides in from the start edge over a scrim, closes on Escape
//! or a click outside, and closes after a destination is chosen.

use aegle_ui::{Align, Container, Node, Point, Result};

use super::{Layout, NavItem};
use crate::{
    Divider, Icon, Role, Text,
    overlay::{Enter, Layer, Place},
    surface::{self, Part, SHEET, SurfaceControl},
    tokens::typescale,
};

/// A standard or modal navigation drawer.
#[derive(Clone)]
pub struct NavigationDrawer {
    layer: Option<Layer>,
    surface: Container,
}

impl std::ops::Deref for NavigationDrawer {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.surface
    }
}

impl NavigationDrawer {
    /// Appends a standard drawer to `parent`.
    pub fn standard(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&SHEET, Part::Pane, 0.0, 0.0);
        Self::create(parent, control, None)
    }

    /// Creates a hidden modal drawer in the theme of `owner`.
    pub fn modal(owner: &Node) -> Result<Self> {
        let layer = Layer::new(owner, Place::Start, true, 0.0)?;
        let mut control = SurfaceControl::new(&SHEET, Part::Sheet, 16.0, 1.0);
        control.corners = [0.0, 16.0, 16.0, 0.0];
        let drawer = Self::create(&layer.host.clone(), control, Some(layer))?;
        let closing = drawer.clone();
        drawer
            .layer
            .as_ref()
            .expect("modal")
            .on_dismiss(move || closing.close())?;
        Ok(drawer)
    }

    fn create(parent: &Container, control: SurfaceControl, layer: Option<Layer>) -> Result<Self> {
        let surface = surface::add(parent, control, 12.0)?;
        surface.set_size(360.0, aegle_ui::Length::Percent(100.0))?;
        surface.set_align_items(Some(Align::Stretch))?;
        Ok(Self { layer, surface })
    }

    /// Appends a section headline.
    pub fn headline(&self, text: &str) -> Result<Text> {
        let headline = Text::new(
            &self.surface,
            typescale::TITLE_SMALL,
            Role::on_surface_variant,
            text,
        )?;
        headline.set_margin(crate::edges(16.0, 18.0, 16.0, 18.0))?;
        Ok(headline)
    }

    /// Appends a destination; the first is selected. Choosing one closes a
    /// modal drawer.
    pub fn item(&self, icon: Icon, text: &str) -> Result<NavItem> {
        let group = self.surface.id;
        let item = NavItem::add(&self.surface, group, Layout::Inline, icon, text)?;
        if super::selected_index(&self.surface)?.is_none() {
            item.select()?;
        }
        if self.layer.is_some() {
            let drawer = self.clone();
            item.on_click(move |_| drawer.close())?;
        }
        Ok(item)
    }

    /// Appends a divider between sections.
    pub fn divider(&self) -> Result<Divider> {
        let divider = Divider::new(&self.surface)?;
        divider.set_inset(16.0, 16.0)?;
        Ok(divider)
    }

    /// Shows a modal drawer above everything, or a standard one in place.
    pub fn show(&self) -> Result {
        match &self.layer {
            Some(layer) => {
                let slide = Enter::Slide(Point::new(-120.0, 0.0));
                layer.show(&self.surface, None, slide)
            }
            None => self.surface.set_visible(true),
        }
    }

    /// Closes a modal drawer, or hides a standard one.
    pub fn close(&self) -> Result {
        match &self.layer {
            Some(layer) => layer.dismiss(),
            None => self.surface.set_visible(false),
        }
    }

    /// Whether a modal drawer is open.
    pub fn is_open(&self) -> Result<bool> {
        match &self.layer {
            Some(layer) => layer.is_open(),
            None => Ok(true),
        }
    }

    /// The index of the selected destination.
    pub fn selected(&self) -> Result<Option<usize>> {
        super::selected_index(&self.surface)
    }
}
