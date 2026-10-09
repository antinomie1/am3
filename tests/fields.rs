//! Text fields on Aegle's editor: typing reports changes, the label and
//! supporting text size the field, the clear button empties it, an error
//! switches its kind, and scrolling keeps the caret inside the text area
//! between the icons.

mod common;

use std::{cell::Cell, rc::Rc};

use aegle_ui::{Key, Modifiers, Point, PointerId, PointerKind, Result};
use am3::{FieldStyle, TextField, icons};
use common::{key, ui};

#[test]
fn typing_clearing_and_errors() -> Result {
    let ui = ui()?;
    let name = TextField::new(&ui.root(), FieldStyle::Filled, "Name")?;
    let changes = Rc::new(Cell::new(0));
    let seen = changes.clone();
    name.on_change(move |_| {
        seen.set(seen.get() + 1);
        Ok(())
    })?;
    ui.refresh()?;
    assert_eq!(name.bounds()?.size.height, 56.0);
    name.set_supporting(Some("As on your passport"))?;
    ui.refresh()?;
    assert_eq!(
        name.bounds()?.size.height,
        76.0,
        "supporting text adds 20 dp"
    );

    name.focus()?;
    ui.paste("Ada")?;
    ui.dispatch_callbacks()?;
    assert_eq!(name.text()?, "Ada");
    assert_eq!(changes.get(), 1);
    key(&ui, Key::Enter)?;
    assert_eq!(changes.get(), 2, "Enter submits");

    name.set_clearable(true)?;
    ui.refresh()?;
    let b = name.bounds()?;
    let at = Point::new(b.origin.x + b.size.width - 24.0, b.origin.y + 28.0);
    ui.pointer(
        PointerId(1),
        PointerKind::Down { clicks: 1 },
        at,
        Modifiers::default(),
    )?;
    ui.pointer(PointerId(1), PointerKind::Up, at, Modifiers::default())?;
    ui.dispatch_callbacks()?;
    assert_eq!(name.text()?, "", "the clear button empties it");
    assert_eq!(changes.get(), 3);

    assert_eq!(name.visual_state()?.kind.name, "FilledTextField");
    name.set_error(true)?;
    assert_eq!(name.visual_state()?.kind.name, "FilledTextFieldError");
    let s = am3::Scheme::of(&ui.theme()?);
    assert_eq!(name.appearance()?.border_color, s.error);
    Ok(())
}

#[test]
fn multiline_grows_and_the_caret_stays_visible() -> Result {
    let ui = ui()?;
    let bio = TextField::multiline(&ui.root(), FieldStyle::Outlined, "Bio")?;
    ui.refresh()?;
    let one = bio.bounds()?.size.height;
    bio.set_text("one\ntwo\nthree")?;
    ui.refresh()?;
    assert!(
        bio.bounds()?.size.height > one + 40.0,
        "three lines are taller"
    );

    let city = TextField::new(&ui.root(), FieldStyle::Outlined, "City")?;
    city.set_width(200.0)?;
    city.set_leading_icon(Some(icons::search()))?;
    city.set_clearable(true)?;
    city.focus()?;
    ui.paste("Llanfairpwllgwyngyll Llanfairpwllgwyngyll Llanfairpwllgwyngyll")?;
    ui.refresh()?;
    let (scroll, caret) = city.change(|state, id| {
        let element = &state.tree.get(id).unwrap().context;
        let caret = element.control.editor().unwrap().editor().ime_rect();
        Ok((element.scroll.x, caret.origin.x + caret.size.width))
    })?;
    assert!(scroll > 0.0, "a long line scrolls");
    // 200 dp less the 52 dp icon areas on each side.
    assert!(
        caret - scroll <= 96.0 + 0.5,
        "the caret stays left of the clear button"
    );
    Ok(())
}
