//! Markup elements for every component, named with an `Md` prefix so they
//! sit beside Aegle's built-in elements: `MdButton`, `MdTextField`,
//! `MdNavigationBar`, and so on. They are ordinary `element!` definitions,
//! the same path any control library takes; the compiled `ui!` path finds
//! each through `use am3::MdButton;`, the loader through [`elements`].
//!
//! Choice properties name enum variants in snake case (`style: tonal`);
//! icons are named as in [`crate::icons`] (`icon: "arrow_back"`), colors
//! by role (`color: "on_surface_variant"`).

mod actions;
mod containment;
mod inputs;
mod navigation;

use std::{any::Any, collections::HashMap, rc::Rc};

use aegle_loader::Elements;
use aegle_ui::{Container, Node, NodeId, Result, State, UiError};

pub use actions::*;
pub use containment::*;
pub use inputs::*;
pub use navigation::*;

use crate::{Icon, Role, icons};

/// Every am3 element, to load markup with:
/// `Program::load_with(path, &am3::elements())`. Combine with other
/// libraries' elements through [`Elements::with`]. Also registers the color
/// tokens markup names, see [`crate::color::register_tokens`].
pub fn elements() -> Elements {
    crate::color::register_tokens();
    navigation::add(inputs::add(containment::add(actions::add(Elements::new()))))
}

/// The bundled icon `name`; markup cannot check names, so an unknown one
/// fails the build of its element.
fn icon(name: &str) -> Result<Icon> {
    icons::named(name).ok_or_else(|| UiError::InvalidValue.into())
}

/// An icon, or none for an empty name.
fn maybe_icon(name: &str) -> Result<Option<Icon>> {
    if name.is_empty() {
        Ok(None)
    } else {
        icon(name).map(Some)
    }
}

/// The color role `name`, such as `on_surface`.
fn role(name: &str) -> Result<Role> {
    Role::ALL
        .iter()
        .copied()
        .find(|role| {
            let token = &role.token_name()["am3.color.".len()..];
            token.len() == name.len()
                && token
                    .bytes()
                    .zip(name.bytes())
                    .all(|(t, n)| t == n || t == b'-' && n == b'_')
        })
        .ok_or_else(|| UiError::InvalidValue.into())
}

/// Maps the names of a checked choice property onto values.
macro_rules! choices {
    ($($f:ident -> $t:ty { $($name:literal => $value:expr),* $(,)? })*) => {$(
        fn $f(name: &str) -> $t {
            match name {
                $($name => $value,)*
                _ => unreachable!("a checked choice"),
            }
        }
    )*};
}
use choices;

choices! {
    button_size -> crate::ButtonSize {
        "extra_small" => crate::ButtonSize::ExtraSmall,
        "small" => crate::ButtonSize::Small,
        "medium" => crate::ButtonSize::Medium,
        "large" => crate::ButtonSize::Large,
        "extra_large" => crate::ButtonSize::ExtraLarge,
    }
    button_style -> crate::ButtonStyle {
        "filled" => crate::ButtonStyle::Filled,
        "tonal" => crate::ButtonStyle::Tonal,
        "outlined" => crate::ButtonStyle::Outlined,
        "elevated" => crate::ButtonStyle::Elevated,
        "text" => crate::ButtonStyle::Text,
    }
    icon_style -> crate::IconStyle {
        "standard" => crate::IconStyle::Standard,
        "filled" => crate::IconStyle::Filled,
        "tonal" => crate::IconStyle::Tonal,
        "outlined" => crate::IconStyle::Outlined,
    }
    fab_color -> crate::FabColor {
        "primary_container" => crate::FabColor::PrimaryContainer,
        "secondary_container" => crate::FabColor::SecondaryContainer,
        "tertiary_container" => crate::FabColor::TertiaryContainer,
        "primary" => crate::FabColor::Primary,
        "secondary" => crate::FabColor::Secondary,
        "tertiary" => crate::FabColor::Tertiary,
    }
}

/// Badges set from markup, by the node they decorate.
#[derive(Default)]
struct Badges(HashMap<NodeId, crate::Badge>);

/// Shows a count on `node`'s badge, a dot for -1, or nothing for 0; the
/// badge is made once, by `make`.
fn badge(node: &Node, count: i64, make: impl FnOnce() -> Result<crate::Badge>) -> Result {
    let badge = match node.change(|state, id| Ok(state.ext::<Badges>().0.get(&id).cloned()))? {
        Some(badge) => badge,
        None => {
            let badge = make()?;
            let kept = badge.clone();
            node.change(|state, id| {
                state.install(&crate::HOOKS);
                state.ext::<Badges>().0.insert(id, kept);
                Ok(())
            })?;
            badge
        }
    };
    match count {
        0 => badge.hide(),
        -1 => badge.show_dot(),
        n => badge.show_count(n as u32),
    }
}

/// Composite handles by the node their child elements are created in, so
/// a child element (a tab, a list item) can reach its parent's handle.
#[derive(Default)]
struct Owners(HashMap<NodeId, Rc<dyn Any>>);

/// Records `handle` as the owner of children created in `node`.
fn adopt<H: Clone + 'static>(handle: H, node: &Node) -> Result<H> {
    let owned = handle.clone();
    node.change(|state, id| {
        state.install(&crate::HOOKS);
        state.ext::<Owners>().0.insert(id, Rc::new(owned));
        Ok(())
    })?;
    Ok(handle)
}

/// The handle owning children created in `parent`, if it is an `H`.
fn owner<H: Clone + 'static>(parent: &Container) -> Result<Option<H>> {
    parent.change(|state, id| {
        Ok(state
            .ext::<Owners>()
            .0
            .get(&id)
            .and_then(|owner| owner.downcast_ref::<H>())
            .cloned())
    })
}

/// The `H` owning `parent`; element specs place children only inside
/// their owners, so another parent is a broken invariant.
fn owned<H: Clone + 'static>(parent: &Container) -> Result<H> {
    Ok(owner(parent)?.expect("a child element inside its owner"))
}

/// Forgets a removed owner or badge.
pub(crate) fn removed(state: &mut State, id: NodeId) {
    fn get<T: 'static>(state: &mut State) -> Option<&mut T> {
        state
            .ext
            .get_mut(&std::any::TypeId::of::<T>())?
            .downcast_mut()
    }
    if let Some(owners) = get::<Owners>(state) {
        owners.0.remove(&id);
    }
    if let Some(badges) = get::<Badges>(state) {
        badges.0.remove(&id);
    }
}
