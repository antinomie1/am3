//! Search: a 56 dp search bar, and the docked search view that opens under
//! it with results while there is a query. The bar is a text field in the
//! search style; the view is an Aegle popup (placed under the bar, closed
//! by Escape or a press outside) holding a 28 dp rounded surface.

use aegle_ui::{Appearance, Container, Result, Theme, VisualState};
use aegle_widgets::{NodePopup, Popup};

use crate::{
    FieldStyle, TextField, icons,
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
};

kinds! { container
    /// The docked search view under a search bar.
    SEARCH_VIEW = "SearchView", search_view => |s, _on| {
        Paint::new(s.surface_container_high, s.on_surface)
    };
}

fn clear(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        border_width: 0.0,
        ..Appearance::base(theme, state)
    }
}

/// A search bar with a docked view of results.
#[derive(Clone)]
pub struct Search {
    bar: TextField,
    popup: Popup,
    results: Container,
}

impl std::ops::Deref for Search {
    type Target = TextField;
    fn deref(&self) -> &TextField {
        &self.bar
    }
}

impl Search {
    /// Appends a search bar showing `placeholder` while empty, 360 to
    /// 720 dp wide.
    pub fn new(parent: &Container, placeholder: &str) -> Result<Self> {
        let bar = TextField::new(parent, FieldStyle::Search, "")?;
        bar.set_leading_icon(Some(icons::search()))?;
        bar.set_placeholder(Some(placeholder))?;
        bar.set_accessible_label(placeholder)?;
        bar.set_min_width(360.0)?;
        bar.set_max_width(720.0)?;
        let popup = bar.popup()?;
        popup.set_skin(Some(clear))?;
        popup.set_padding(0.0)?;
        let view = SurfaceControl::new(&SEARCH_VIEW, Part::Pane, 28.0, 0.0);
        let results = surface::add(&popup, view, 0.0)?;
        results.set_padding(crate::edges(0.0, 8.0, 0.0, 8.0))?;
        results.set_margin(crate::edges(0.0, 4.0, 0.0, 0.0))?;
        let search = Self {
            bar,
            popup,
            results,
        };
        let shown = search.clone();
        search.bar.on_change(move |bar| {
            if bar.text()?.is_empty() {
                shown.popup.hide()
            } else {
                shown.show_results()
            }
        })?;
        Ok(search)
    }

    /// The search bar.
    pub fn bar(&self) -> &TextField {
        &self.bar
    }

    /// The view's content, for suggestions and results such as list items.
    pub fn results(&self) -> &Container {
        &self.results
    }

    /// Opens the view under the bar, keeping focus in the bar.
    pub fn show_results(&self) -> Result {
        if !self.popup.is_shown()? {
            self.popup.show()?;
            self.bar.focus()?;
        }
        Ok(())
    }

    /// Closes the view.
    pub fn hide_results(&self) -> Result {
        self.popup.hide()
    }

    /// Whether the view is open.
    pub fn results_shown(&self) -> Result<bool> {
        self.popup.is_shown()
    }
}
