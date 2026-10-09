//! Selection controls and sliders: checkboxes leave the mixed state on
//! change, radio buttons stay exclusive under clicks and arrow keys,
//! switches report changes, and sliders follow drags, keys and steps, with
//! a range slider's handles kept in order.

mod common;

use std::{cell::Cell, rc::Rc};

use aegle_controls::KeyInput;
use aegle_ui::{Key, Modifiers, PointerId, PointerKind, Result};
use am3::{Checkbox, Radio, Slider, Switch};
use common::{click, ui};

fn key(ui: &aegle_ui::Ui, key: Key) -> Result {
    for pressed in [true, false] {
        ui.key(KeyInput {
            key,
            text: "",
            modifiers: Modifiers::default(),
            pressed,
            repeat: false,
        })?;
    }
    ui.dispatch_callbacks()?;
    ui.refresh()?;
    Ok(())
}

#[test]
fn checkboxes_radios_and_switches_change() -> Result {
    let ui = ui()?;
    let root = ui.root();
    let check = Checkbox::new(&root, "All", false)?;
    check.set_mixed(true)?;
    let radios: Vec<_> = ["S", "M", "L"]
        .into_iter()
        .map(|t| Radio::new(&root, t, t == "S"))
        .collect::<Result<_>>()?;
    let switch = Switch::new(&root, "Wi-Fi", false)?;
    let changes = Rc::new(Cell::new(0));
    let seen = changes.clone();
    switch.on_change(move |_| {
        seen.set(seen.get() + 1);
        Ok(())
    })?;
    ui.refresh()?;

    click(&ui, &check)?;
    assert!(check.is_checked()? && !check.is_mixed()?);

    let chosen = || -> Result<Vec<bool>> { radios.iter().map(|r| r.is_checked()).collect() };
    click(&ui, &radios[2])?;
    assert_eq!(chosen()?, [false, false, true]);
    // Arrow keys move the choice and wrap.
    key(&ui, Key::Down)?;
    assert_eq!(chosen()?, [true, false, false]);
    key(&ui, Key::Up)?;
    assert_eq!(chosen()?, [false, false, true]);

    click(&ui, &switch)?;
    click(&ui, &switch)?;
    assert_eq!(changes.get(), 2);
    assert!(!switch.is_checked()?);
    Ok(())
}

#[test]
fn sliders_follow_pointer_keys_and_steps() -> Result {
    let ui = ui()?;
    let root = ui.root();
    let slider = Slider::new(&root, 0.0, 100.0, 0.0)?;
    ui.refresh()?;
    let b = slider.bounds()?;
    let at = |f: f32| {
        aegle_ui::Point::new(
            b.origin.x + 2.0 + (b.size.width - 4.0) * f,
            b.origin.y + b.size.height / 2.0,
        )
    };
    let mods = Modifiers::default();
    ui.pointer(
        PointerId(1),
        PointerKind::Down { clicks: 1 },
        at(0.25),
        mods,
    )?;
    ui.pointer(PointerId(1), PointerKind::Move, at(0.75), mods)?;
    ui.pointer(PointerId(1), PointerKind::Up, at(0.75), mods)?;
    assert!((slider.value()? - 75.0).abs() < 0.5, "{}", slider.value()?);
    slider.set_step(10.0, true)?;
    key(&ui, Key::Right)?;
    assert_eq!(slider.value()?, 90.0);
    key(&ui, Key::Home)?;
    assert_eq!(slider.value()?, 0.0);

    // A range slider: a press takes the nearer handle, and the start never
    // passes the end.
    let range = Slider::new(&root, 0.0, 100.0, 80.0)?;
    range.set_range(true)?;
    range.set_start(20.0)?;
    ui.refresh()?;
    let b = range.bounds()?;
    let at = |f: f32| {
        aegle_ui::Point::new(
            b.origin.x + 2.0 + (b.size.width - 4.0) * f,
            b.origin.y + b.size.height / 2.0,
        )
    };
    ui.pointer(PointerId(1), PointerKind::Down { clicks: 1 }, at(0.3), mods)?;
    ui.pointer(PointerId(1), PointerKind::Move, at(0.95), mods)?;
    ui.pointer(PointerId(1), PointerKind::Up, at(0.95), mods)?;
    let (start, end) = (range.start()?.unwrap(), range.value()?);
    assert!(start <= end, "{start} > {end}");
    assert!((start - 95.0).abs() < 0.5 && (end - 95.0).abs() < 0.5);
    Ok(())
}
