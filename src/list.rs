//! Lists: a column of items, each a row of an optional leading element, a
//! body-large headline with optional supporting text, and an optional
//! trailing element. Items are 56 dp tall, 72 dp with supporting text; a
//! clickable item is a button with a state layer and ripple.

use aegle_ui::{Align, Color, Container, Result};

use crate::{
    Icon, IconView, Role, Text,
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
    tokens::typescale,
};

kinds! { container
    /// A list's container.
    LIST = "List", list => |s, _on| Paint::new(s.surface, s.on_surface);
    /// A list item.
    LIST_ITEM = "ListItem", list_item => |s, _on| Paint::new(Color::TRANSPARENT, s.on_surface);
}

/// A vertical list.
#[derive(Clone)]
pub struct List(Container);

impl std::ops::Deref for List {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.0
    }
}

impl List {
    /// Appends an empty list with 8 dp padding above and below.
    pub fn new(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&LIST, Part::List, 0.0, 0.0);
        let list = surface::add(parent, control, 0.0)?;
        list.set_padding(crate::edges(0.0, 8.0, 0.0, 8.0))?;
        list.set_gap(0.0)?;
        Ok(Self(list))
    }

    /// Appends an item showing `headline`.
    pub fn item(&self, headline: &str) -> Result<ListItem> {
        ListItem::create(self, headline, false)
    }

    /// Appends an item that is a button.
    pub fn clickable_item(&self, headline: &str) -> Result<ListItem> {
        ListItem::create(self, headline, true)
    }
}

/// An item of a [`List`].
#[derive(Clone)]
pub struct ListItem {
    row: Container,
    leading: Container,
    texts: Container,
    trailing: Container,
}

impl std::ops::Deref for ListItem {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.row
    }
}

impl ListItem {
    fn create(list: &Container, headline: &str, clickable: bool) -> Result<Self> {
        let mut control = SurfaceControl::new(&LIST_ITEM, Part::ListItem, 0.0, 0.0);
        if clickable {
            control = control.clickable();
        }
        let row = surface::add(list, control, 0.0)?;
        row.set_direction(aegle_ui::Direction::Row)?;
        row.set_align_items(Some(Align::Center))?;
        row.set_gap(16.0)?;
        row.set_min_height(56.0)?;
        row.set_padding(crate::edges(16.0, 8.0, 24.0, 8.0))?;
        let leading = row.row()?;
        leading.set_visible(false)?;
        let texts = row.column()?;
        texts.set_gap(0.0)?;
        texts.set_grow(1.0)?;
        texts.set_shrink(1.0)?;
        Text::new(&texts, typescale::BODY_LARGE, Role::on_surface, headline)?;
        let trailing = row.row()?;
        trailing.set_align_items(Some(Align::Center))?;
        trailing.set_visible(false)?;
        Ok(Self {
            row,
            leading,
            texts,
            trailing,
        })
    }

    /// Adds supporting text below the headline; the item grows to 72 dp.
    pub fn supporting(&self, text: &str) -> Result<Text> {
        self.row.set_min_height(72.0)?;
        Text::new(
            &self.texts,
            typescale::BODY_MEDIUM,
            Role::on_surface_variant,
            text,
        )
    }

    /// The leading slot, for an avatar, image, checkbox or radio button.
    pub fn leading(&self) -> Result<&Container> {
        self.leading.set_visible(true)?;
        Ok(&self.leading)
    }

    /// The trailing slot, for a switch, checkbox or icon button.
    pub fn trailing(&self) -> Result<&Container> {
        self.trailing.set_visible(true)?;
        Ok(&self.trailing)
    }

    /// Puts a 24 dp on-surface-variant icon in the leading slot.
    pub fn leading_icon(&self, icon: Icon) -> Result<IconView> {
        IconView::new(self.leading()?, icon, 24.0, Role::on_surface_variant)
    }

    /// Puts a 24 dp on-surface-variant icon in the trailing slot.
    pub fn trailing_icon(&self, icon: Icon) -> Result<IconView> {
        IconView::new(self.trailing()?, icon, 24.0, Role::on_surface_variant)
    }

    /// Puts label-small supporting text in the trailing slot.
    pub fn trailing_text(&self, text: &str) -> Result<Text> {
        Text::new(
            self.trailing()?,
            typescale::LABEL_SMALL,
            Role::on_surface_variant,
            text,
        )
    }

    /// Adds a handler run after each activation of a clickable item.
    pub fn on_click(&self, mut callback: impl FnMut(ListItem) -> Result + 'static) -> Result {
        let item = self.clone();
        self.row
            .change(|state, id| state.on_action(id, move |_| callback(item.clone())))
    }

    /// Activates a clickable item through the normal enabled checks.
    pub fn activate(&self) -> Result {
        self.row
            .change(|state, id| state.dispatch(id, aegle_ui::control::Input::Activate))
    }
}
