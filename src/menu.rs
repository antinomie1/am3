//! Menus: Aegle's menus (keyboard navigation, submenus, check and radio
//! items, shortcut hints, placement) in Material's look. The popup is a
//! 16 dp rounded surface container with a level 2 shadow; items are 44 dp
//! tall body-large rows with 12 dp rounded state layers, inset 6 dp.

use aegle_ui::{Appearance, Node, Result, Theme, ThemeOverride, VisualState};
use aegle_widgets::{NodeMenu, kinds};

use crate::{
    Scheme,
    skin::{self, Paint},
    tokens::elevation,
};

fn panel(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        background: Scheme::of(theme).surface_container,
        border_width: 0.0,
        radius: 16.0,
        ..Appearance::base(theme, state)
    }
}

fn item(theme: &Theme, state: VisualState) -> Appearance {
    let s = &Scheme::of(theme);
    Appearance {
        radius: 12.0,
        indicator: if state.enabled {
            s.primary
        } else {
            crate::color::alpha(s.on_surface, crate::tokens::state::DISABLED_CONTENT)
        },
        // Opaque over the panel: Aegle composites in linear light, which
        // would make a translucent layer stronger than Material's.
        ..skin::pressable(
            theme,
            state,
            s,
            Paint::new(s.surface_container, s.on_surface),
        )
    }
}

/// Gives an Aegle menu popup Material's look.
fn style(menu: &aegle_widgets::Menu) -> Result {
    let theme = menu.theme()?;
    let s = Scheme::of(&theme);
    menu.set_kind_skin(&kinds::PANEL, Some(panel))?;
    menu.set_kind_skin(&kinds::MENU_ITEM, Some(item))?;
    // Body large items with 12 dp padding: 44 dp rows, in the host's scale.
    menu.set_theme_override(Some(ThemeOverride {
        font_size: Some(theme.font_size * 16.0 / 14.0),
        padding: Some(12.0),
        gap: Some(12.0),
        border: Some(s.outline_variant),
        muted: Some(s.on_surface_variant),
        ..Default::default()
    }))?;
    menu.set_shadow(elevation(2.0, s.shadow))
}

/// A Material menu: an [`aegle_widgets::Menu`] (items, check and radio
/// items, separators, [`aegle_widgets::Popup::show`]) whose submenus share
/// its look.
#[derive(Clone)]
pub struct Menu(aegle_widgets::Menu);

impl std::ops::Deref for Menu {
    type Target = aegle_widgets::Menu;
    fn deref(&self) -> &aegle_widgets::Menu {
        &self.0
    }
}

impl Menu {
    /// A hidden menu that opens below `anchor`, as from a button.
    pub fn new(anchor: &Node) -> Result<Self> {
        let menu = anchor.menu()?;
        style(&menu)?;
        Ok(Self(menu))
    }

    /// A hidden menu `anchor` opens at the pointer on a secondary click,
    /// a long press or the menu key.
    pub fn context(anchor: &Node) -> Result<Self> {
        let menu = anchor.context_menu()?;
        style(&menu)?;
        Ok(Self(menu))
    }

    /// Appends an item opening a submenu in the same look.
    pub fn submenu(&self, text: &str) -> Result<Menu> {
        let menu = self.0.submenu(text)?;
        style(&menu)?;
        Ok(Self(menu))
    }
}
