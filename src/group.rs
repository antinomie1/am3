//! Button groups: a row of buttons and icon buttons of one size. In a
//! standard group a pressed button briefly widens into its neighbors; in a
//! connected group the buttons join with 2 dp gaps and square inner corners,
//! and a selected one turns fully round.

use aegle_ui::{Align, Container, Result};

use crate::{Button, ButtonSize, ButtonStyle, Icon, IconButton, IconStyle, PressableControl};

/// How the buttons of a group sit together.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupStyle {
    /// Spaced buttons that bounce against each other when pressed.
    #[default]
    Standard,
    /// Buttons joined into one shape, 2 dp apart.
    Connected,
}

/// How the buttons of a group are selected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    /// Plain buttons: activation only.
    #[default]
    None,
    /// Toggle buttons with exactly one selected.
    Single,
    /// Independent toggle buttons.
    Multiple,
}

/// A row of buttons that changes their shape together.
#[derive(Clone)]
pub struct ButtonGroup {
    row: Container,
    style: GroupStyle,
    size: ButtonSize,
    selection: std::rc::Rc<std::cell::Cell<Selection>>,
}

impl std::ops::Deref for ButtonGroup {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.row
    }
}

impl ButtonGroup {
    /// Appends an empty group to `parent`; add buttons with
    /// [`ButtonGroup::button`] and [`ButtonGroup::icon_button`].
    pub fn new(parent: &Container, style: GroupStyle, size: ButtonSize) -> Result<Self> {
        let row = parent.row()?;
        row.set_align_items(Some(Align::Center))?;
        let gap = match (style, size) {
            (GroupStyle::Connected, _) => 2.0,
            (_, ButtonSize::ExtraSmall) => 18.0,
            (_, ButtonSize::Small) => 12.0,
            _ => 8.0,
        };
        row.set_gap(gap)?;
        Ok(Self {
            row,
            style,
            size,
            selection: Default::default(),
        })
    }

    /// The container of the buttons.
    pub fn container(&self) -> &Container {
        &self.row
    }

    /// Appends a button of the group's size.
    pub fn button(&self, style: ButtonStyle, text: &str) -> Result<Button> {
        let button = Button::new(&self.row, style, text)?;
        button.set_size(self.size)?;
        self.arrange()?;
        Ok(button)
    }

    /// Appends an icon button of the group's size.
    pub fn icon_button(&self, style: IconStyle, icon: Icon, label: &str) -> Result<IconButton> {
        let button = IconButton::new(&self.row, style, icon, label)?;
        button.set_size(self.size)?;
        self.arrange()?;
        Ok(button)
    }

    /// Makes the buttons plain, single-select or multi-select toggles; the
    /// first button starts selected in a single-select group.
    pub fn set_selection(&self, selection: Selection) -> Result {
        self.selection.set(selection);
        self.row.change(|state, id| {
            let children: Vec<_> = state.tree.children(id)?.collect();
            for (index, child) in children.into_iter().enumerate() {
                if let Some(c) = state.control_as::<PressableControl>(child) {
                    c.exclusive = selection == Selection::Single;
                    c.selected = match selection {
                        Selection::None => None,
                        Selection::Single => Some(index == 0),
                        Selection::Multiple => Some(c.selected == Some(true)),
                    };
                    state
                        .tree
                        .mark_dirty(child, aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS)?;
                }
            }
            Ok(())
        })
    }

    /// Joins connected buttons to their neighbors, lets standard ones
    /// bounce, and gives new buttons the group's selection.
    fn arrange(&self) -> Result {
        let (style, selection) = (self.style, self.selection.get());
        self.row.change(|state, id| {
            let children: Vec<_> = state.tree.children(id)?.collect();
            let last = children.len().saturating_sub(1);
            for (index, child) in children.into_iter().enumerate() {
                if let Some(c) = state.control_as::<PressableControl>(child) {
                    let connected = style == GroupStyle::Connected;
                    c.joined = [connected && index > 0, connected && index < last];
                    c.bounce = !connected;
                    c.exclusive = selection == Selection::Single;
                    if selection != Selection::None && c.selected.is_none() {
                        c.selected = Some(false);
                    }
                    state.tree.mark_dirty(child, aegle_ui::Dirty::PAINT)?;
                }
            }
            Ok(())
        })
    }
}
