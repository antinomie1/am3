//! Gallery shots of navigation and app structure: navigation bars, rails
//! and drawers, tabs, app bars and toolbars.

use aegle_ui::{Container, Result, Ui};
use am3::*;

use crate::{hover, shots::Shot};

fn bar(_: &Ui, host: &Container) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let bar = NavigationBar::new(host)?;
    bar.item(icons::home(), "Home")?;
    let mail = bar.item(icons::mail(), "Mail")?;
    mail.badge()?.show_count(3)?;
    let chat = bar.item(icons::notifications(), "Alerts")?;
    chat.badge()?.show_dot()?;
    bar.item(icons::settings(), "Settings").map(drop)
}

fn rail(_: &Ui, host: &Container, expanded: bool) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let row = host.row()?;
    row.set_grow(1.0)?;
    let rail = NavigationRail::new(&row)?;
    let header = rail.header()?;
    IconButton::new(header, IconStyle::Standard, icons::menu(), "Menu")?;
    Fab::new(header, icons::edit(), "Compose")?.set_color(FabColor::TertiaryContainer)?;
    rail.item(icons::mail(), "Inbox")?;
    rail.item(icons::send(), "Outbox")?;
    rail.item(icons::star(), "Favorites")?;
    rail.set_expanded(expanded)
}

fn drawer(_: &Ui, host: &Container) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let row = host.row()?;
    row.set_grow(1.0)?;
    let drawer = NavigationDrawer::standard(&row)?;
    drawer.set_width(280.0)?;
    drawer.headline("Mail")?;
    drawer.item(icons::mail(), "Inbox")?;
    drawer.item(icons::send(), "Outbox")?;
    drawer.item(icons::star(), "Favorites")?;
    drawer.divider()?;
    drawer.headline("Labels")?;
    drawer.item(icons::bookmark(), "Saved").map(drop)
}

fn tabs(ui: &Ui, host: &Container, style: TabStyle, icons: bool) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let tabs = Tabs::new(host, style)?;
    let icon = |i: Icon| icons.then_some(i);
    tabs.tab(icon(icons::send()), "Flights")?;
    let trips = tabs.tab(icon(icons::calendar()), "Trips")?;
    tabs.tab(icon(icons::explore()), "Explore")?;
    trips.select()?;
    ui.refresh()?;
    Ok(())
}

fn top(_: &Ui, host: &Container, style: AppBarStyle) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let bar = TopAppBar::new(host, style, "Title")?;
    bar.navigation(icons::arrow_back(), "Back")?;
    bar.action(icons::bookmark(), "Save")?;
    bar.action(icons::more_vert(), "More")?;
    if style == AppBarStyle::Medium {
        bar.subtitle("Subtitle")?;
    }
    Ok(())
}

fn bottom(_: &Ui, host: &Container) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let bar = BottomAppBar::new(host)?;
    for (icon, label) in [
        (icons::search(), "Search"),
        (icons::delete(), "Delete"),
        (icons::share(), "Share"),
    ] {
        bar.action(icon, label)?;
    }
    bar.fab(icons::add(), "New").map(drop)
}

fn docked(ui: &Ui, host: &Container, color: ToolbarColor) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let bar = Toolbar::docked(host, color)?;
    bar.icon_button(icons::arrow_back(), "Back")?;
    bar.icon_button(icons::arrow_forward(), "Forward")?;
    let add = bar.icon_button(icons::add(), "Add")?;
    add.set_style(IconStyle::Filled)?;
    bar.icon_button(icons::share(), "Share")?;
    hover(ui, &bar.icon_button(icons::more_vert(), "More")?)
}

fn floating(_: &Ui, host: &Container, color: ToolbarColor, vertical: bool) -> Result {
    let bar = Toolbar::floating(host, color, vertical)?;
    for (icon, label) in [
        (icons::edit(), "Edit"),
        (icons::photo(), "Photo"),
        (icons::mic(), "Voice"),
    ] {
        bar.icon_button(icon, label)?;
    }
    let toggle = bar.icon_button(icons::keyboard(), "Keyboard")?;
    toggle.set_selected(Some(true))?;
    Ok(())
}

pub(crate) const NAVIGATION: &[Shot] = &[
    (
        "navigation-bar",
        (380.0, 110.0),
        &[("navigation bar with badges", bar)],
    ),
    (
        "navigation-rail",
        (300.0, 420.0),
        &[
            ("collapsed", |ui, h| rail(ui, h, false)),
            ("expanded", |ui, h| rail(ui, h, true)),
        ],
    ),
    (
        "navigation-drawer",
        (320.0, 400.0),
        &[("standard drawer", drawer)],
    ),
    (
        "tabs",
        (360.0, 110.0),
        &[
            ("primary, icons", |ui, h| {
                tabs(ui, h, TabStyle::Primary, true)
            }),
            ("primary", |ui, h| tabs(ui, h, TabStyle::Primary, false)),
            ("secondary", |ui, h| tabs(ui, h, TabStyle::Secondary, false)),
        ],
    ),
    (
        "top-app-bars",
        (360.0, 170.0),
        &[
            ("small", |ui, h| top(ui, h, AppBarStyle::Small)),
            ("center-aligned", |ui, h| {
                top(ui, h, AppBarStyle::CenterAligned)
            }),
            ("medium flexible", |ui, h| top(ui, h, AppBarStyle::Medium)),
            ("large flexible", |ui, h| top(ui, h, AppBarStyle::Large)),
        ],
    ),
    (
        "bottom-app-bar",
        (380.0, 120.0),
        &[("bottom app bar", bottom)],
    ),
    (
        "toolbars",
        (380.0, 110.0),
        &[
            ("docked", |ui, h| docked(ui, h, ToolbarColor::Standard)),
            ("docked, vibrant", |ui, h| {
                docked(ui, h, ToolbarColor::Vibrant)
            }),
        ],
    ),
    (
        "floating-toolbars",
        (300.0, 330.0),
        &[
            ("floating", |ui, h| {
                floating(ui, h, ToolbarColor::Standard, false)
            }),
            ("floating, vibrant, vertical", |ui, h| {
                floating(ui, h, ToolbarColor::Vibrant, true)
            }),
        ],
    ),
];
