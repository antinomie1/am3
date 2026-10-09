//! Chips and indicators: a filter chip toggles and widens for its check,
//! an input chip's remove icon and Delete key remove without clicking, and
//! progress indicators keep their values and size for the wave.

mod common;

use std::{cell::RefCell, rc::Rc};

use aegle_controls::KeyInput;
use aegle_ui::{Key, Modifiers, Point, PointerId, PointerKind, Result};
use am3::{Chip, ChipKind, LoadingIndicator, Progress};
use common::{click, ui};

#[test]
fn chips_toggle_and_remove() -> Result {
    let ui = ui()?;
    let root = ui.root();
    let filter = Chip::new(&root, ChipKind::Filter, "Vegan")?;
    let input = Chip::new(&root, ChipKind::Input, "Ada")?;
    let log = Rc::new(RefCell::new(Vec::new()));
    let (clicks, removals) = (log.clone(), log.clone());
    input.on_click(move |_| {
        clicks.borrow_mut().push("click");
        Ok(())
    })?;
    input.on_remove(move |_| {
        removals.borrow_mut().push("remove");
        Ok(())
    })?;
    ui.refresh()?;
    let narrow = filter.bounds()?.size.width;
    click(&ui, &filter)?;
    assert_eq!(filter.selected()?, Some(true));
    assert!(filter.bounds()?.size.width > narrow + 10.0);

    // The label clicks; the trailing icon removes.
    let b = input.bounds()?;
    let mods = Modifiers::default();
    for x in [b.origin.x + 12.0, b.origin.x + b.size.width - 12.0] {
        let at = Point::new(x, b.origin.y + b.size.height / 2.0);
        ui.pointer(PointerId(1), PointerKind::Down { clicks: 1 }, at, mods)?;
        ui.pointer(PointerId(1), PointerKind::Up, at, mods)?;
        ui.dispatch_callbacks()?;
    }
    input.focus()?;
    ui.key(KeyInput {
        key: Key::Delete,
        text: "",
        modifiers: mods,
        pressed: true,
        repeat: false,
    })?;
    ui.dispatch_callbacks()?;
    assert_eq!(*log.borrow(), ["click", "remove", "remove"]);
    Ok(())
}

#[test]
fn indicators_keep_values_and_fit_waves() -> Result {
    let ui = ui()?;
    let root = ui.root();
    let linear = Progress::linear(&root, Some(1.5))?;
    let circular = Progress::circular(&root, None)?;
    LoadingIndicator::new(&root, true)?;
    ui.refresh()?;
    assert_eq!(linear.value()?, Some(1.0));
    assert_eq!(circular.value()?, None);
    assert_eq!(linear.bounds()?.size.height, 4.0);
    linear.set_wavy(true)?;
    circular.set_wavy(true)?;
    ui.refresh()?;
    assert_eq!(linear.bounds()?.size.height, 10.0);
    assert_eq!(circular.bounds()?.size.width, 48.0);
    Ok(())
}
