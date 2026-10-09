//! Gallery shots of containment and communication components: cards,
//! dialogs, sheets, snackbars, menus, tooltips, lists, dividers and badges.

use std::time::{Duration, Instant};

use aegle_ui::{Container, Result, Ui};
use am3::{tokens::typescale, *};

use crate::{hover, pointer_mode, shots::Shot};

/// A card's content: a headline, supporting text and an action.
fn card(host: &Container, style: CardStyle) -> Result {
    let card = Card::new(host, style)?;
    card.set_width(200.0)?;
    Text::new(&card, typescale::TITLE_MEDIUM, Role::on_surface, "Headline")?;
    Text::new(
        &card,
        typescale::BODY_MEDIUM,
        Role::on_surface_variant,
        "Supporting text that explains the card.",
    )?;
    Button::new(&card, ButtonStyle::Filled, "Action").map(drop)
}

/// Page content for a layer to cover.
fn page(host: &Container) -> Result {
    Text::new(host, typescale::TITLE_LARGE, Role::on_surface, "Inbox")?;
    for line in ["Lunch on Friday?", "Your order shipped", "Weekly summary"] {
        Text::new(host, typescale::BODY_LARGE, Role::on_surface_variant, line)?;
    }
    Ok(())
}

fn dialog(ui: &Ui, host: &Container) -> Result {
    page(host)?;
    pointer_mode(ui)?;
    let dialog = Dialog::new(host, Some(icons::delete()), "Delete draft?")?;
    dialog.supporting("The draft will be removed from this device and can't be recovered.")?;
    dialog.action("Cancel")?;
    dialog.action("Delete")?;
    dialog.surface().set_max_width(300.0)?;
    dialog.show()
}

fn bottom_sheet(ui: &Ui, host: &Container) -> Result {
    page(host)?;
    pointer_mode(ui)?;
    let sheet = Sheet::bottom(host)?;
    let list = List::new(&sheet)?;
    list.set_background(aegle_ui::Color::TRANSPARENT)?;
    for (icon, text) in [
        (icons::share(), "Share"),
        (icons::download(), "Download"),
        (icons::delete(), "Delete"),
    ] {
        list.clickable_item(text)?.leading_icon(icon)?;
    }
    sheet.show()
}

fn side_sheet(ui: &Ui, host: &Container) -> Result {
    page(host)?;
    pointer_mode(ui)?;
    let sheet = Sheet::side(host, "Filters")?;
    sheet.surface().set_width(220.0)?;
    sheet.set_gap(12.0)?;
    for (text, on) in [("Unread", true), ("Starred", false), ("Attachments", true)] {
        Checkbox::new(&sheet, text, on)?;
    }
    sheet.show()
}

fn menu(ui: &Ui, host: &Container) -> Result {
    let button = Button::new(host, ButtonStyle::Tonal, "Edit")?;
    let menu = Menu::new(&button)?;
    menu.item("Cut")?.set_shortcut(Some("Ctrl+X"))?;
    menu.item("Copy")?.set_shortcut(Some("Ctrl+C"))?;
    menu.item("Paste")?.set_shortcut(Some("Ctrl+V"))?;
    menu.separator()?;
    menu.check_item("Show ruler", true)?;
    menu.submenu("Transform")?.item("Uppercase")?;
    ui.refresh()?;
    menu.show()?;
    hover(ui, &menu.item("Select all")?)
}

fn tooltip(ui: &Ui, host: &Container) -> Result {
    let button = IconButton::new(host, IconStyle::Standard, icons::bookmark(), "Save")?;
    set_tooltip(&button, Some("Save to favorites"))?;
    hover(ui, &button)?;
    ui.wake(Instant::now() + Duration::from_secs(1))
}

fn rich_tooltip(ui: &Ui, host: &Container) -> Result {
    let button = IconButton::new(host, IconStyle::Standard, icons::info(), "About")?;
    let tip = RichTooltip::new(
        &button,
        Some("Rich tooltip"),
        "Rich tooltips bring attention to a feature and can hold an action.",
    )?;
    tip.action("Learn more")?;
    ui.refresh()?;
    pointer_mode(ui)?;
    tip.show()
}

fn snackbar(_: &Ui, host: &Container, action: bool) -> Result {
    page(host)?;
    let bar = Snackbar::new(host, "Message archived")?;
    if action {
        bar.action("Undo")?;
        bar.closable()?;
    }
    bar.show(None)
}

fn list(_: &Ui, host: &Container) -> Result {
    let list = List::new(host)?;
    list.set_width(300.0)?;
    let item = list.clickable_item("Ali Connors")?;
    item.leading_icon(icons::person())?;
    item.supporting("Brunch this weekend?")?;
    item.trailing_text("15 min")?;
    Divider::new(&list)?.set_inset(56.0, 0.0)?;
    let item = list.item("Wi-Fi")?;
    item.leading_icon(icons::settings())?;
    Switch::new(item.trailing()?, "", true)?;
    Divider::new(&list)?.set_inset(56.0, 0.0)?;
    let item = list.item("Notifications")?;
    item.leading_icon(icons::notifications())?;
    item.supporting("Sounds, vibration")?;
    item.trailing_icon(icons::chevron_right()).map(drop)
}

fn badges(_: &Ui, host: &Container) -> Result {
    let row = host.row()?;
    row.set_gap(16.0)?;
    let dot = IconButton::new(&row, IconStyle::Standard, icons::mail(), "Mail")?;
    Badge::new(&dot)?.show_dot()?;
    let count = IconButton::new(&row, IconStyle::Standard, icons::notifications(), "Alerts")?;
    Badge::new(&count)?.show_count(3)?;
    let many = IconButton::new(&row, IconStyle::Tonal, icons::mail(), "Inbox")?;
    Badge::new(&many)?.show_count(1200)
}

pub(crate) const SURFACES: &[Shot] = &[
    (
        "cards",
        (230.0, 200.0),
        &[
            ("elevated", |_, h| card(h, CardStyle::Elevated)),
            ("filled", |_, h| card(h, CardStyle::Filled)),
            ("outlined", |_, h| card(h, CardStyle::Outlined)),
        ],
    ),
    ("dialogs", (380.0, 320.0), &[("basic, with icon", dialog)]),
    (
        "sheets",
        (360.0, 300.0),
        &[
            ("modal bottom sheet", bottom_sheet),
            ("modal side sheet", side_sheet),
        ],
    ),
    (
        "snackbars",
        (380.0, 170.0),
        &[
            ("single line", |ui, h| snackbar(ui, h, false)),
            ("with action and close", |ui, h| snackbar(ui, h, true)),
        ],
    ),
    ("menus", (300.0, 380.0), &[("menu", menu)]),
    (
        "tooltips",
        (300.0, 230.0),
        &[("plain", tooltip), ("rich", rich_tooltip)],
    ),
    ("lists", (340.0, 290.0), &[("list with dividers", list)]),
    ("badges", (240.0, 90.0), &[("small, count, 999+", badges)]),
];
