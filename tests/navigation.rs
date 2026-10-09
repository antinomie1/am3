//! Navigation components keep exactly one destination selected, the rail
//! relays out when expanded, a modal drawer closes after a choice, tabs
//! move their selection, a scrolled top app bar recolors, and skins set
//! for a kind follow controls whose kind changes.

mod common;

use aegle_ui::Result;
use am3::{
    AppBarStyle, IconStyle, NavigationBar, NavigationDrawer, NavigationRail, Scheme, TabStyle,
    Tabs, Toolbar, ToolbarColor, TopAppBar, icons,
};
use common::{click, ui};

#[test]
fn one_destination_is_selected() -> Result {
    let ui = ui()?;
    let bar = NavigationBar::new(&ui.root())?;
    let home = bar.item(icons::home(), "Home")?;
    let mail = bar.item(icons::mail(), "Mail")?;
    mail.badge()?.show_count(4)?;
    bar.item(icons::settings(), "Settings")?;
    assert_eq!(bar.selected()?, Some(0));
    click(&ui, &mail)?;
    assert_eq!(bar.selected()?, Some(1));
    assert!(!home.is_selected()? && mail.is_selected()?);
    assert!(mail.visual_state()?.checked);

    let rail = NavigationRail::new(&ui.root())?;
    let inbox = rail.item(icons::mail(), "Inbox")?;
    let outbox = rail.item(icons::send(), "Outbox")?;
    outbox.select()?;
    ui.refresh()?;
    assert!(!inbox.is_selected()?);
    assert_eq!(
        inbox.bounds()?.size.height,
        52.0,
        "indicator, gap and label"
    );
    rail.set_expanded(true)?;
    ui.refresh()?;
    assert_eq!(inbox.bounds()?.size.height, 56.0, "an inline pill");
    assert!(rail.bounds()?.size.width >= 220.0);
    assert_eq!(rail.selected()?, Some(1));
    Ok(())
}

#[test]
fn modal_drawer_closes_after_a_choice() -> Result {
    let ui = ui()?;
    let drawer = NavigationDrawer::modal(&ui.root())?;
    drawer.headline("Mail")?;
    drawer.item(icons::mail(), "Inbox")?;
    let starred = drawer.item(icons::star(), "Starred")?;
    drawer.show()?;
    ui.refresh()?;
    assert!(drawer.is_open()?);
    click(&ui, &starred)?;
    assert_eq!(drawer.selected()?, Some(1));
    assert!(!drawer.is_open()?);
    Ok(())
}

#[test]
fn tabs_select_one() -> Result {
    let ui = ui()?;
    let tabs = Tabs::new(&ui.root(), TabStyle::Primary)?;
    let flights = tabs.tab(Some(icons::send()), "Flights")?;
    let trips = tabs.tab(Some(icons::calendar()), "Trips")?;
    ui.refresh()?;
    assert_eq!(tabs.selected()?, Some(0));
    assert_eq!(flights.bounds()?.size.height, 64.0, "icon above label");
    assert_eq!(flights.bounds()?.size.width, trips.bounds()?.size.width);
    click(&ui, &trips)?;
    assert_eq!(tabs.selected()?, Some(1));
    assert!(!flights.is_selected()?);
    let secondary = Tabs::new(&ui.root(), TabStyle::Secondary)?;
    let tab = secondary.tab(Some(icons::send()), "Overview")?;
    ui.refresh()?;
    assert_eq!(
        tab.bounds()?.size.height,
        48.0,
        "secondary tabs show no icon"
    );
    Ok(())
}

#[test]
fn bars_recolor_and_skins_follow_kind_changes() -> Result {
    let ui = ui()?;
    let bar = TopAppBar::new(&ui.root(), AppBarStyle::Medium, "Title")?;
    bar.navigation(icons::arrow_back(), "Back")?;
    ui.refresh()?;
    assert_eq!(bar.bounds()?.size.height, 112.0);
    let s = Scheme::of(&ui.theme()?);
    let background = |n: &aegle_ui::Node| n.appearance().map(|a| a.background);
    assert_eq!(background(&bar)?, s.surface);
    bar.set_scrolled(true)?;
    assert_eq!(background(&bar)?, s.surface_container);

    let toolbar = Toolbar::floating(&ui.root(), ToolbarColor::Vibrant, false)?;
    let button = toolbar.icon_button(icons::add(), "Add")?;
    assert_eq!(button.appearance()?.indicator, s.on_primary_container);
    button.set_style(IconStyle::Filled)?;
    assert_eq!(
        background(&button)?,
        s.primary,
        "the filled kind's own skin"
    );
    button.set_style(IconStyle::Standard)?;
    assert_eq!(button.appearance()?.indicator, s.on_primary_container);
    Ok(())
}
