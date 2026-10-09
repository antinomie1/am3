//! Elements of navigation and app structure: the navigation bar, rail and
//! drawer with their `MdNavItem` destinations, tabs, app bars and
//! toolbars with their `MdAction` icon buttons.

use aegle_loader::{Elements, element};
use aegle_ui::{Container, Result, UiError};

use super::{adopt, badge, choices, icon, maybe_icon, owned, owner};
use crate::{
    AppBarStyle, BottomAppBar, IconButton, NavItem, NavigationBar, NavigationDrawer,
    NavigationRail, Tab, TabStyle, Tabs, Toolbar, ToolbarColor, TopAppBar,
};

choices! {
    app_bar_style -> AppBarStyle {
        "small" => AppBarStyle::Small,
        "center_aligned" => AppBarStyle::CenterAligned,
        "medium" => AppBarStyle::Medium,
        "large" => AppBarStyle::Large,
    }
}

/// A destination of the bar, rail or drawer owning `parent`.
fn nav_item(parent: &Container, icon: &str, text: &str) -> Result<NavItem> {
    let icon = super::icon(icon)?;
    if let Some(bar) = owner::<NavigationBar>(parent)? {
        bar.item(icon, text)
    } else if let Some(rail) = owner::<NavigationRail>(parent)? {
        rail.item(icon, text)
    } else if let Some(drawer) = owner::<NavigationDrawer>(parent)? {
        drawer.item(icon, text)
    } else {
        Err(UiError::InvalidValue.into())
    }
}

/// An icon button of the app bar or toolbar owning `parent`.
fn action(parent: &Container, icon: &str, label: &str, navigation: bool) -> Result<IconButton> {
    let icon = super::icon(icon)?;
    if let Some(bar) = owner::<TopAppBar>(parent)? {
        if navigation {
            bar.navigation(icon, label)
        } else {
            bar.action(icon, label)
        }
    } else if let Some(bar) = owner::<BottomAppBar>(parent)? {
        bar.action(icon, label)
    } else if let Some(toolbar) = owner::<Toolbar>(parent)? {
        toolbar.icon_button(icon, label)
    } else {
        Err(UiError::InvalidValue.into())
    }
}

fn drawer(parent: &Container, modal: bool, headline: &str) -> Result<NavigationDrawer> {
    let drawer = if modal {
        NavigationDrawer::modal(parent)?
    } else {
        NavigationDrawer::standard(parent)?
    };
    if !headline.is_empty() {
        drawer.headline(headline)?;
    }
    adopt(drawer.clone(), &drawer)
}

fn top_app_bar(parent: &Container, title: &str, style: &str, subtitle: &str) -> Result<TopAppBar> {
    let bar = TopAppBar::new(parent, app_bar_style(style), title)?;
    if !subtitle.is_empty() {
        bar.subtitle(subtitle)?;
    }
    adopt(bar.clone(), &bar)
}

fn toolbar(parent: &Container, floating: bool, vertical: bool, vibrant: bool) -> Result<Toolbar> {
    let color = if vibrant {
        ToolbarColor::Vibrant
    } else {
        ToolbarColor::Standard
    };
    let toolbar = if floating {
        Toolbar::floating(parent, color, vertical)?
    } else {
        Toolbar::docked(parent, color)?
    };
    adopt(toolbar.clone(), &toolbar)
}

/// A selected index as markup reads it: -1 for none.
fn index(selected: Option<usize>) -> i64 {
    selected.map_or(-1, |i| i as i64)
}

element! {
    /// A navigation bar of `MdNavItem` destinations.
    pub MdNavigationBar(NavigationBar) {
        layout flex;
        children only MdNavItem;
        create |parent| {
            let bar = NavigationBar::new(parent)?;
            adopt(bar.clone(), &bar)
        };
        get selected: int => |bar| bar.selected().map(index);
    }
    /// A navigation rail of `MdNavItem` destinations.
    pub MdNavigationRail(NavigationRail) {
        layout flex;
        children only MdNavItem;
        create |parent, expanded: bool = false| {
            let rail = NavigationRail::new(parent)?;
            rail.set_expanded(expanded)?;
            adopt(rail.clone(), &rail)
        };
        set expanded: bool => |rail, on| rail.set_expanded(on);
        get selected: int => |rail| rail.selected().map(index);
    }
    /// A standard or modal navigation drawer of `MdNavItem` destinations;
    /// a modal one shows while `open`.
    pub MdNavigationDrawer(NavigationDrawer) {
        layout flex;
        children only MdNavItem;
        create |parent, modal: bool = false, headline: line = ""| drawer(parent, modal, headline);
        set open: bool => |drawer, open| if open { drawer.show() } else { drawer.close() };
        get selected: int => |drawer| drawer.selected().map(index);
    }
    /// A destination of a navigation bar, rail or drawer; `badge` shows a
    /// count, a dot for -1, or nothing for 0.
    pub MdNavItem(NavItem) {
        create |parent, icon: line, text: line| nav_item(parent, icon, text);
        set selected_icon: line => |item, name| item.set_selected_icon(maybe_icon(name)?);
        set selected: bool => |item, on| if on { item.select() } else { Ok(()) };
        set badge: int(-1, 1000000) => |item, count| badge(item, count, || item.badge());
        event clicked => |item, run| item.on_click(move |_| run());
        get selected: bool => |item| item.is_selected();
    }
    /// A row of `MdTab` tabs.
    pub MdTabs(Tabs) {
        layout flex;
        children only MdTab;
        create |parent, style: choice(primary, secondary) = "primary"| {
            let style = if style == "secondary" { TabStyle::Secondary } else { TabStyle::Primary };
            let tabs = Tabs::new(parent, style)?;
            adopt(tabs.clone(), &tabs)
        };
        get selected: int => |tabs| tabs.selected().map(index);
    }
    /// A tab of an `MdTabs` row.
    pub MdTab(Tab) {
        parent MdTabs;
        create |parent, text: line, icon: line = ""| owned::<Tabs>(parent)?.tab(maybe_icon(icon)?, text);
        set selected: bool => |tab, on| if on { tab.select() } else { Ok(()) };
        event clicked => |tab, run| tab.on_click(move |_| run());
        get selected: bool => |tab| tab.is_selected();
    }
    /// A top app bar with `MdAction` buttons.
    pub MdTopAppBar(TopAppBar) {
        layout flex;
        children only MdAction;
        create |parent, title: line = "", style: choice(small, center_aligned, medium, large) = "small", subtitle: line = ""| top_app_bar(parent, title, style, subtitle);
        set title: line => |bar, text| bar.title().set_text(text);
        set scrolled: bool => |bar, on| bar.set_scrolled(on);
    }
    /// A bottom app bar of `MdAction` buttons.
    pub MdBottomAppBar(BottomAppBar) {
        layout flex;
        children only MdAction;
        create |parent| {
            let bar = BottomAppBar::new(parent)?;
            adopt(bar.clone(), &bar)
        };
    }
    /// A docked or floating toolbar of `MdAction` buttons.
    pub MdToolbar(Toolbar) {
        layout flex;
        children only MdAction;
        create |parent, floating: bool = false, vertical: bool = false, vibrant: bool = false| toolbar(parent, floating, vertical, vibrant);
    }
    /// An icon button of an app bar or toolbar; `navigation` puts it at a
    /// top app bar's start.
    pub MdAction(IconButton) {
        create |parent, icon: line, label: line, navigation: bool = false| action(parent, icon, label, navigation);
        set icon: line => |button, name| button.set_icon(Some(icon(name)?));
        event clicked => |button, run| button.on_click(move |_| run());
    }
}

pub(super) fn add(elements: Elements) -> Elements {
    elements
        .with::<MdNavigationBar>()
        .with::<MdNavigationRail>()
        .with::<MdNavigationDrawer>()
        .with::<MdNavItem>()
        .with::<MdTabs>()
        .with::<MdTab>()
        .with::<MdTopAppBar>()
        .with::<MdBottomAppBar>()
        .with::<MdToolbar>()
        .with::<MdAction>()
}
