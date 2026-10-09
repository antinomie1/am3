//! The navigation bar: three to five destinations along the bottom of a
//! compact window, on the surface container, 64 dp tall (expressive).

use aegle_ui::{Align, Container, Result};

use super::{Layout, NavItem};
use crate::{
    Icon,
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
};

kinds! { container
    /// A navigation bar's container.
    NAV_BAR = "NavigationBar", nav_bar => |s, _on| Paint::new(s.surface_container, s.on_surface);
}

/// A row of destinations, the first one selected.
#[derive(Clone)]
pub struct NavigationBar(Container);

impl std::ops::Deref for NavigationBar {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.0
    }
}

impl NavigationBar {
    /// Appends an empty bar, as wide as its parent allows.
    pub fn new(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&NAV_BAR, Part::Pane, 0.0, 0.0);
        let bar = surface::add(parent, control, 0.0)?;
        bar.set_direction(aegle_ui::Direction::Row)?;
        bar.set_align_items(Some(Align::Center))?;
        bar.set_align_self(Some(Align::Stretch))?;
        bar.set_height(64.0)?;
        bar.set_gap(8.0)?;
        bar.set_padding(crate::edges(8.0, 0.0, 8.0, 0.0))?;
        Ok(Self(bar))
    }

    /// Appends a destination; the first is selected.
    pub fn item(&self, icon: Icon, text: &str) -> Result<NavItem> {
        let item = NavItem::add(&self.0, self.0.id, Layout::Stacked, icon, text)?;
        item.set_grow(1.0)?;
        item.set_basis(0.0)?;
        if self.selected()?.is_none() {
            item.select()?;
        }
        Ok(item)
    }

    /// The index of the selected destination.
    pub fn selected(&self) -> Result<Option<usize>> {
        super::selected_index(&self.0)
    }
}
