//! The FAB menu: a FAB that opens a short list of related actions above
//! itself. Open, the FAB turns into a round close button in its set's
//! strong color, and the items rise in one after another.

use std::time::Duration;

use aegle_ui::{Align, Container, ControlKind, Result, handle};

use crate::{
    ButtonShape, ButtonSize, ButtonStyle, Fab, FabColor, Icon,
    button::ButtonLook,
    icons,
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
};

pub(crate) fn item_spec() -> Spec {
    let look = ButtonLook {
        style: ButtonStyle::Filled,
        size: ButtonSize::Medium,
        shape: ButtonShape::Round,
        split: false,
    };
    Spec {
        elevation: [0.0, 0.0],
        ..look.spec()
    }
}

pub(crate) fn item_kind(color: FabColor) -> &'static ControlKind {
    match color {
        FabColor::PrimaryContainer | FabColor::Primary => &FAB_MENU_ITEM_PRIMARY,
        FabColor::SecondaryContainer | FabColor::Secondary => &FAB_MENU_ITEM_SECONDARY,
        FabColor::TertiaryContainer | FabColor::Tertiary => &FAB_MENU_ITEM_TERTIARY,
    }
}

kinds! {
    /// An item of a FAB menu in primary colors.
    FAB_MENU_ITEM_PRIMARY = "FabMenuItem", primary => |s, _on| {
        Paint::new(s.primary_container, s.on_primary_container)
    };
    /// An item of a FAB menu in secondary colors.
    FAB_MENU_ITEM_SECONDARY = "SecondaryFabMenuItem", secondary => |s, _on| {
        Paint::new(s.secondary_container, s.on_secondary_container)
    };
    /// An item of a FAB menu in tertiary colors.
    FAB_MENU_ITEM_TERTIARY = "TertiaryFabMenuItem", tertiary => |s, _on| {
        Paint::new(s.tertiary_container, s.on_tertiary_container)
    };
}

handle! {
    /// One action of a [`FabMenu`]; activating it closes the menu.
    pub FabMenuItem(PressableControl): text, interactive, pressed, indicator
}

pressable_methods!(FabMenuItem);

/// A FAB with a menu of up to six related actions.
#[derive(Clone)]
pub struct FabMenu {
    column: Container,
    items: Container,
    fab: Fab,
    color: FabColor,
}

impl std::ops::Deref for FabMenu {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.column
    }
}

impl FabMenu {
    /// Appends a closed menu whose FAB shows `icon` and is named `label`.
    pub fn new(parent: &Container, icon: Icon, label: &str, color: FabColor) -> Result<Self> {
        let column = parent.column()?;
        column.set_align_items(Some(Align::End))?;
        column.set_gap(8.0)?;
        let items = column.column()?;
        items.set_align_items(Some(Align::End))?;
        items.set_gap(4.0)?;
        items.set_visible(false)?;
        let fab = Fab::new(&column, icon, label)?;
        fab.set_color(color)?;
        pressable::relook(&fab, |c| {
            c.selected = Some(false);
            c.selected_icon = Some(icons::close());
        })?;
        let menu = Self {
            column,
            items,
            fab,
            color,
        };
        let shown = menu.clone();
        menu.fab
            .on_click(move |fab| shown.show(fab.selected()? == Some(true)))?;
        Ok(menu)
    }

    /// Appends an item.
    pub fn item(&self, icon: Icon, text: &str) -> Result<FabMenuItem> {
        let color = self.color;
        let item = pressable::add(&self.items, |fonts, theme| {
            let mut control = PressableControl::new(fonts, theme, Look::FabItem(color), text)?;
            control.icon = Some(icon);
            Ok(control)
        })
        .map(FabMenuItem)?;
        let menu = self.clone();
        item.on_click(move |_| menu.set_open(false))?;
        Ok(item)
    }

    /// The FAB that opens and closes the menu.
    pub fn fab(&self) -> &Fab {
        &self.fab
    }

    /// Opens or closes the menu.
    pub fn set_open(&self, open: bool) -> Result {
        self.fab.set_selected(Some(open))?;
        self.show(open)
    }

    /// Whether the menu is open.
    pub fn is_open(&self) -> Result<bool> {
        Ok(self.fab.selected()? == Some(true))
    }

    fn show(&self, open: bool) -> Result {
        self.items.set_visible(open)?;
        #[cfg(feature = "motion")]
        if open {
            self.rise()?;
        }
        Ok(())
    }

    /// The items fade and rise in, the one nearest the FAB first.
    #[cfg(feature = "motion")]
    fn rise(&self) -> Result {
        use aegle_ui::{Animate, Animation, Point};
        let items: Vec<aegle_ui::NodeId> = self
            .items
            .change(|state, id| Ok(state.tree.children(id)?.collect()))?;
        let count = items.len();
        let spring = crate::tokens::motion::FAST_SPATIAL.transition();
        let fade = crate::tokens::motion::DEFAULT_EFFECTS.transition();
        for (index, id) in items.into_iter().enumerate() {
            let node = aegle_ui::Node {
                state: self.items.state.clone(),
                id,
            };
            let delay = Duration::from_millis(30 * (count - 1 - index) as u64);
            let rise = Animation::tween(Point::new(0.0, 16.0), Point::default(), spring)?;
            node.animate(Animate::Offset(rise.delay(delay)))?;
            node.animate(Animate::Opacity(
                Animation::tween(0.0, 1.0, fade)?.delay(delay),
            ))?;
        }
        Ok(())
    }
}
