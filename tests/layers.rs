//! Modal layers and the components on them: focus moves in and back, Tab
//! stays inside, Escape and the scrim dismiss, input below is blocked, a
//! menu over a dialog closes first, and snackbars time out without
//! blocking anything.

mod common;

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use aegle_ui::{Key, Modifiers, Point, Result};
use am3::{Button, ButtonStyle, Card, CardStyle, Dialog, List, Menu, Sheet, Snackbar};
use common::{click, click_at, key, key_with, ui};

fn counter() -> (Rc<Cell<u32>>, Rc<Cell<u32>>) {
    let count = Rc::new(Cell::new(0));
    (count.clone(), count)
}

#[test]
fn dialog_traps_focus_and_dismisses() -> Result {
    let ui = ui()?;
    let opener = Button::new(&ui.root(), ButtonStyle::Filled, "Open")?;
    let (clicks, seen) = counter();
    opener.on_click(move |_| {
        clicks.set(clicks.get() + 1);
        Ok(())
    })?;
    let dialog = Dialog::new(&opener, None, "Discard draft?")?;
    dialog.supporting("Changes are lost.")?;
    let cancel = dialog.action("Cancel")?;
    let discard = dialog.action("Discard")?;
    let (dismissed, dismissals) = counter();
    dialog.on_dismiss(move || {
        dismissed.set(dismissed.get() + 1);
        Ok(())
    })?;

    opener.focus()?;
    dialog.show()?;
    ui.refresh()?;
    assert!(dialog.is_open()?);
    assert!(cancel.is_focused()?, "focus moves to the first control");
    key(&ui, Key::Tab)?;
    assert!(discard.is_focused()?);
    key(&ui, Key::Tab)?;
    assert!(cancel.is_focused()?, "Tab wraps inside the dialog");
    key_with(
        &ui,
        Key::Tab,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    )?;
    assert!(discard.is_focused()?);

    // Text inside the surface is not the scrim.
    click_at(&ui, common::center(dialog.surface())?)?;
    assert!(dialog.is_open()?);

    key(&ui, Key::Escape)?;
    assert!(!dialog.is_open()?);
    assert_eq!(dismissals.get(), 1);
    assert!(opener.is_focused()?, "focus returns to where it was");

    // A click on the scrim over the opener dismisses without reaching it.
    dialog.show()?;
    click(&ui, &opener)?;
    assert!(!dialog.is_open()?);
    assert_eq!(dismissals.get(), 2);
    assert_eq!(seen.get(), 0, "the scrim blocks the control below");

    // Closed (after its fade), the opener is reachable again.
    ui.advance_animations(Duration::from_secs(1))?;
    ui.dispatch_callbacks()?;
    click(&ui, &opener)?;
    assert_eq!(seen.get(), 1);
    Ok(())
}

#[test]
fn menu_over_dialog_closes_first() -> Result {
    let ui = ui()?;
    let dialog = Dialog::new(&ui.root(), None, "Sort")?;
    let by = dialog.action("Sort by")?;
    let menu = Menu::new(&by)?;
    menu.item("Name")?;
    menu.item("Date")?;
    dialog.show()?;
    ui.refresh()?;
    menu.show()?;
    ui.refresh()?;
    assert!(menu.is_shown()?);
    key(&ui, Key::Escape)?;
    assert!(!menu.is_shown()?, "Escape closes the menu");
    assert!(dialog.is_open()?, "and leaves the dialog open");
    key(&ui, Key::Escape)?;
    assert!(!dialog.is_open()?);
    Ok(())
}

#[test]
fn sheets_open_and_close() -> Result {
    let ui = ui()?;
    let bottom = Sheet::bottom(&ui.root())?;
    Button::new(&bottom, ButtonStyle::Text, "Share")?;
    bottom.show()?;
    // Past its entrance, which starts on the first frame.
    ui.refresh()?;
    ui.advance_animations(Duration::from_secs(1))?;
    ui.refresh()?;
    assert!(bottom.is_open()?);
    let sheet = bottom.surface().bounds()?;
    assert!(
        (sheet.origin.y + sheet.size.height - 600.0).abs() < 0.5,
        "a bottom sheet sits on the window's bottom edge"
    );
    click_at(&ui, Point::new(300.0, 10.0))?;
    assert!(!bottom.is_open()?, "a click above it dismisses it");

    let side = Sheet::side(&ui.root(), "Filters")?;
    side.show()?;
    ui.refresh()?;
    ui.advance_animations(Duration::from_secs(2))?;
    ui.refresh()?;
    let bounds = side.surface().bounds()?;
    assert!((bounds.origin.x + bounds.size.width - 600.0).abs() < 0.5);
    assert!((bounds.size.height - 600.0).abs() < 0.5, "full height");
    key(&ui, Key::Escape)?;
    assert!(!side.is_open()?);

    let standard = Sheet::standard_side(&ui.root(), "Details")?;
    assert!(standard.is_open()?);
    standard.close()?;
    assert!(!standard.is_open()?);
    Ok(())
}

#[test]
fn snackbar_times_out_without_blocking() -> Result {
    let ui = ui()?;
    let below = Button::new(&ui.root(), ButtonStyle::Filled, "Below")?;
    let (clicks, seen) = counter();
    below.on_click(move |_| {
        clicks.set(clicks.get() + 1);
        Ok(())
    })?;
    let bar = Snackbar::new(&ui.root(), "Archived")?;
    let undo = bar.action("Undo")?;
    let (timeouts, timed_out) = counter();
    bar.on_timeout(move || {
        timeouts.set(timeouts.get() + 1);
        Ok(())
    })?;
    bar.show(Some(am3::snackbar::LONG))?;
    ui.refresh()?;
    click(&ui, &below)?;
    assert_eq!(seen.get(), 1, "a snackbar does not block input");
    assert!(bar.is_open()?);
    ui.wake(Instant::now() + Duration::from_secs(11))?;
    ui.dispatch_callbacks()?;
    assert!(!bar.is_open()?);
    assert_eq!(timed_out.get(), 1);

    bar.show(None)?;
    click(&ui, &undo)?;
    assert!(!bar.is_open()?, "its action closes it");
    Ok(())
}

#[test]
fn clickable_cards_and_list_items_activate() -> Result {
    let ui = ui()?;
    let card = Card::clickable(&ui.root(), CardStyle::Outlined)?;
    card.set_size(120.0, 80.0)?;
    let (clicks, seen) = counter();
    card.on_click(move |_| {
        clicks.set(clicks.get() + 1);
        Ok(())
    })?;
    click(&ui, &card)?;
    assert_eq!(seen.get(), 1);
    let list = List::new(&ui.root())?;
    let item = list.clickable_item("Inbox")?;
    let still = list.item("Static")?;
    let (picks, picked) = counter();
    item.on_click(move |_| {
        picks.set(picks.get() + 1);
        Ok(())
    })?;
    click(&ui, &item)?;
    click(&ui, &still)?;
    assert_eq!(picked.get(), 1);
    assert_eq!(item.bounds()?.size.height, 56.0);
    item.supporting("3 new")?;
    ui.refresh()?;
    assert_eq!(item.bounds()?.size.height, 72.0);
    Ok(())
}
