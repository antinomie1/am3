//! Navigation between top-level destinations: the navigation bar, the
//! navigation rail (collapsed or expanded) and the navigation drawer
//! (standard or modal). All three hold [`NavItem`]s, exactly one of which
//! is selected.

mod bar;
mod drawer;
mod item;
mod rail;

use aegle_ui::{
    Appearance, Color, Container, Control as _, NodeId, Result, State, Theme, VisualState, handle,
};

pub use bar::{NAV_BAR, NavigationBar};
pub use drawer::NavigationDrawer;
use item::Layout;
pub use item::NavItemControl;
pub use rail::{NAV_RAIL, NavigationRail};

use crate::{
    Badge, Icon, Scheme,
    color::{alpha, blend},
    skin::layer_opacity,
    tokens::state,
};

fn colors(theme: &Theme, state: VisualState, label: Color) -> Appearance {
    let s = Scheme::of(theme);
    let on = state.checked;
    let opacity = layer_opacity(state);
    let (icon, label) = if !state.enabled {
        let off = alpha(s.on_surface, state::DISABLED_CONTENT);
        (off, off)
    } else if on {
        (s.on_secondary_container, label)
    } else {
        (s.on_surface_variant, s.on_surface_variant)
    };
    Appearance {
        background: if on && opacity > 0.0 {
            blend(s.secondary_container, s.on_secondary_container, opacity)
        } else {
            s.secondary_container
        },
        border_color: if !on && opacity > 0.0 {
            alpha(s.on_surface, opacity)
        } else {
            Color::TRANSPARENT
        },
        foreground: label,
        indicator: icon,
        focus_color: s.secondary,
        focus_width: 0.0,
        ..Appearance::base(theme, state)
    }
}

fn stacked(theme: &Theme, state: VisualState) -> Appearance {
    colors(theme, state, Scheme::of(theme).secondary)
}

fn inline(theme: &Theme, state: VisualState) -> Appearance {
    colors(theme, state, Scheme::of(theme).on_secondary_container)
}

/// A destination of a navigation bar or collapsed rail.
pub static NAV_ITEM: aegle_ui::ControlKind = aegle_ui::ControlKind {
    name: "NavigationItem",
    skin: stacked,
    accepts: crate::skin::PRESSABLE,
    container: false,
};
/// A destination of a navigation drawer or expanded rail.
pub static NAV_DRAWER_ITEM: aegle_ui::ControlKind = aegle_ui::ControlKind {
    name: "NavigationDrawerItem",
    skin: inline,
    accepts: crate::skin::PRESSABLE,
    container: false,
};

/// Deselects the other items of the navigation component of `id`.
fn deselect_others(state: &mut State, id: NodeId) -> Result {
    let Some(group) = state.control_as::<NavItemControl>(id).and_then(|c| c.group) else {
        return Ok(());
    };
    state.rebuild_order();
    let items: Vec<_> = state
        .order
        .iter()
        .copied()
        .filter(|&n| n != id && state.contains(group, n))
        .collect();
    for item in items {
        if let Some(other) = state.control_as::<NavItemControl>(item)
            && other.selected
        {
            other.selected = false;
            let dirty = aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS;
            state.tree.mark_dirty(item, dirty)?;
        }
    }
    Ok(())
}

handle! {
    /// A destination in a navigation bar, rail or drawer.
    pub NavItem(NavItemControl): text, interactive, pressed, indicator
}

impl NavItem {
    pub(crate) fn add(
        parent: &Container,
        group: NodeId,
        layout: Layout,
        icon: Icon,
        text: &str,
    ) -> Result<Self> {
        let node = parent.add(|state, theme| {
            let mut control = NavItemControl::new(&state.fonts, theme, layout, icon, text)?;
            control.group = Some(group);
            let style = aegle_layout::Style {
                flex_shrink: 0.0,
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn aegle_ui::Control>, style))
        })?;
        crate::effects(&node)?;
        Ok(Self(node))
    }

    /// Selects it, deselecting the other destinations.
    pub fn select(&self) -> Result {
        self.change(|state, id| {
            state
                .control_as::<NavItemControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?
                .selected = true;
            state.dirty_visual_state(id)?;
            state.tree.mark_dirty(id, aegle_ui::Dirty::SEMANTICS)?;
            deselect_others(state, id)
        })
    }

    /// Whether it is the selected destination.
    pub fn is_selected(&self) -> Result<bool> {
        self.read(|c| c.selected)
    }

    /// The icon shown while selected, typically the filled variant.
    pub fn set_selected_icon(&self, icon: Option<Icon>) -> Result {
        self.update(|c| c.selected_icon = icon)
    }

    /// A hidden badge on its icon.
    pub fn badge(&self) -> Result<Badge> {
        let icon = self.read(|c| c.icon_at.clone())?;
        Badge::placed(self, Some(icon))
    }

    /// Activates it through the normal enabled checks.
    pub fn activate(&self) -> Result {
        self.change(|state, id| state.dispatch(id, aegle_ui::control::Input::Activate))
    }

    /// Adds a handler run after each activation.
    pub fn on_click(&self, mut callback: impl FnMut(NavItem) -> Result + 'static) -> Result {
        self.change(|state, id| state.on_action(id, move |node| callback(NavItem(node))))
    }
}

/// The index of the selected item among the items inside `group`.
fn selected_index(group: &Container) -> Result<Option<usize>> {
    group.change(|state, id| {
        state.rebuild_order();
        let items: Vec<_> = state
            .order
            .iter()
            .copied()
            .filter(|&n| state.contains(id, n))
            .collect();
        let mut index = 0;
        for item in items {
            if let Some(c) = state.control_as::<NavItemControl>(item) {
                if c.selected {
                    return Ok(Some(index));
                }
                index += 1;
            }
        }
        Ok(None)
    })
}

/// Changes how the items inside `group` lay out, reshaping their labels.
fn set_layout(group: &Container, layout: Layout) -> Result {
    group.change(|state, id| {
        state.rebuild_order();
        let items: Vec<_> = state
            .order
            .iter()
            .copied()
            .filter(|&n| state.contains(id, n))
            .collect();
        let fonts = state.fonts.clone();
        for item in items {
            if state.control_as::<NavItemControl>(item).is_none() {
                continue;
            }
            state.control_as::<NavItemControl>(item).unwrap().layout = layout;
            let style = state.text_style(item);
            let control = state.control_as::<NavItemControl>(item).unwrap();
            fonts
                .borrow_mut()
                .restyle(control.paragraph_mut().unwrap(), &style)?;
            state.tree.mark_dirty(item, aegle_ui::Dirty::ALL)?;
        }
        Ok(())
    })
}
