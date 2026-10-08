//! Split buttons: an action button joined to a menu button. Hovering,
//! focusing or pressing a part rounds its inner corners; opening the menu
//! selects the menu part, which turns round with its arrow turned over.

use aegle_ui::{Align, Container, Result};

use crate::{
    Button, ButtonSize, ButtonStyle, icons,
    pressable::{self, Look},
};

/// An action and a menu of related actions, in one of the button styles.
#[derive(Clone)]
pub struct SplitButton {
    row: Container,
    action: Button,
    menu: Button,
}

impl std::ops::Deref for SplitButton {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.row
    }
}

impl SplitButton {
    /// Appends a small split button whose action reads `text`. The menu
    /// part is a toggle: show a menu when it turns selected, and deselect it
    /// with [`SplitButton::set_menu_open`] when the menu closes.
    pub fn new(parent: &Container, style: ButtonStyle, text: &str) -> Result<Self> {
        let row = parent.row()?;
        row.set_gap(2.0)?;
        row.set_align_items(Some(Align::Center))?;
        let action = Button::new(&row, style, text)?;
        let menu = Button::new(&row, style, "More options")?;
        let split = Self { row, action, menu };
        split.join()?;
        Ok(split)
    }

    fn join(&self) -> Result {
        for (part, menu) in [(&self.action, false), (&self.menu, true)] {
            let mut look = part.look()?;
            look.split = true;
            pressable::relook(part, |c| {
                c.set_look(Look::Button(look));
                c.joined = [menu, !menu];
                if menu {
                    c.hide_label = true;
                    c.turn = true;
                    c.trailing = Some(icons::expand_more());
                    c.selected = Some(false);
                }
            })?;
        }
        Ok(())
    }

    /// The action part.
    pub fn action(&self) -> &Button {
        &self.action
    }

    /// The menu part; its `on_click` reports each open and close.
    pub fn menu(&self) -> &Button {
        &self.menu
    }

    /// Shows the menu part open or closed.
    pub fn set_menu_open(&self, open: bool) -> Result {
        self.menu.set_selected(Some(open))
    }

    /// Changes the size of both parts.
    pub fn set_size(&self, size: ButtonSize) -> Result {
        self.action.relook(|look| look.size = size)?;
        self.menu.relook(|look| look.size = size)
    }

    /// Changes the color style of both parts.
    pub fn set_style(&self, style: ButtonStyle) -> Result {
        self.action.relook(|look| look.style = style)?;
        self.menu.relook(|look| look.style = style)
    }
}
