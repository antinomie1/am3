//! The navigation rail: destinations down the start edge of a medium or
//! larger window. Collapsed it is 96 dp wide with labels under the
//! indicators; expanded it is at least 220 dp wide with inline items, the
//! expressive replacement for a standard drawer.

use aegle_ui::{Align, Container, Result};

use super::{Layout, NavItem};
use crate::{
    Icon,
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
};

kinds! { container
    /// A navigation rail's container.
    NAV_RAIL = "NavigationRail", nav_rail => |s, _on| Paint::new(s.surface, s.on_surface);
}

/// A column of destinations with an optional header for a menu button and
/// a FAB.
#[derive(Clone)]
pub struct NavigationRail {
    rail: Container,
    expanded: std::rc::Rc<std::cell::Cell<bool>>,
    header: Container,
    items: Container,
}

impl std::ops::Deref for NavigationRail {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.rail
    }
}

impl NavigationRail {
    /// Appends a collapsed rail, as tall as its parent allows.
    pub fn new(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&NAV_RAIL, Part::Pane, 0.0, 0.0);
        let rail = surface::add(parent, control, 0.0)?;
        rail.set_align_self(Some(Align::Stretch))?;
        rail.set_padding(crate::edges(0.0, 44.0, 0.0, 16.0))?;
        rail.set_gap(40.0)?;
        let header = rail.column()?;
        header.set_gap(8.0)?;
        header.set_visible(false)?;
        let items = rail.column()?;
        items.set_gap(4.0)?;
        let rail = Self {
            rail,
            expanded: Default::default(),
            header,
            items,
        };
        rail.set_expanded(false)?;
        Ok(rail)
    }

    /// The header above the destinations, for a menu button and a FAB.
    pub fn header(&self) -> Result<&Container> {
        self.header.set_visible(true)?;
        Ok(&self.header)
    }

    /// Appends a destination; the first is selected.
    pub fn item(&self, icon: Icon, text: &str) -> Result<NavItem> {
        let layout = if self.expanded.get() {
            Layout::Inline
        } else {
            Layout::Stacked
        };
        let item = NavItem::add(&self.items, self.rail.id, layout, icon, text)?;
        if self.selected()?.is_none() {
            item.select()?;
        }
        Ok(item)
    }

    /// Expands it to show inline items at least 220 dp wide, or collapses
    /// it to 96 dp.
    pub fn set_expanded(&self, expanded: bool) -> Result {
        self.expanded.set(expanded);
        let (width, align, pad) = if expanded {
            (aegle_ui::Length::Auto, Align::Stretch, 20.0)
        } else {
            (aegle_ui::Length::Px(96.0), Align::Center, 0.0)
        };
        self.rail.set_width(width)?;
        self.rail
            .set_min_width(if expanded { 220.0 } else { 96.0 })?;
        self.rail.set_align_items(Some(align))?;
        self.header.set_align_items(Some(if expanded {
            Align::Start
        } else {
            Align::Center
        }))?;
        self.header.set_padding(crate::edges(pad, 0.0, pad, 0.0))?;
        self.items.set_align_items(Some(align))?;
        self.items.set_padding(crate::edges(pad, 0.0, pad, 0.0))?;
        let layout = if expanded {
            Layout::Inline
        } else {
            Layout::Stacked
        };
        super::set_layout(&self.items, layout)
    }

    /// Whether it is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded.get()
    }

    /// The index of the selected destination.
    pub fn selected(&self) -> Result<Option<usize>> {
        super::selected_index(&self.items)
    }
}
