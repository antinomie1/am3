//! Buttons: toggles flip and report, sizes relayout and restyle the label,
//! fixed-width components keep their width, and focus shows only after keys.

mod common;

use std::{cell::RefCell, rc::Rc};

use common::{click, ui};

use aegle_controls::KeyInput;
use aegle_ui::{Key, Modifiers, Result, Size};
use am3::{Button, ButtonSize, ButtonStyle, Fab, IconButton, IconStyle, icons};

#[test]
fn toggle_buttons_flip_and_report() -> Result {
    let ui = ui()?;
    let root = ui.root();
    root.set_align_items(Some(aegle_ui::Align::Start))?;
    let button = Button::new(&root, ButtonStyle::Tonal, "Bold")?;
    button.set_selected(Some(false))?;
    let clicks = Rc::new(RefCell::new(Vec::new()));
    let seen = clicks.clone();
    button.on_click(move |b| {
        seen.borrow_mut().push(b.selected()?);
        Ok(())
    })?;
    ui.refresh()?;
    assert_eq!(button.visual_state()?.kind.name, "TonalToggleButton");
    click(&ui, &button)?;
    click(&ui, &button)?;
    assert_eq!(*clicks.borrow(), [Some(true), Some(false)]);
    // A plain button activates without a selection.
    button.set_selected(None)?;
    click(&ui, &button)?;
    assert_eq!(clicks.borrow().last(), Some(&None));
    assert_eq!(button.visual_state()?.kind.name, "TonalButton");
    Ok(())
}

#[test]
fn sizes_relayout_and_restyle_the_label() -> Result {
    let ui = ui()?;
    let root = ui.root();
    root.set_align_items(Some(aegle_ui::Align::Start))?;
    let button = Button::new(&root, ButtonStyle::Filled, "Label")?;
    ui.refresh()?;
    let small = button.bounds()?.size;
    assert_eq!(small.height, 40.0);
    button.set_size(ButtonSize::Large)?;
    ui.refresh()?;
    let large = button.bounds()?.size;
    assert_eq!(large.height, 96.0);
    // 48 dp padding each side and a headline-small label.
    assert!(large.width > small.width + 64.0, "{small:?} → {large:?}");
    // The type role survives text changes.
    button.set_text("Label")?;
    ui.refresh()?;
    assert_eq!(button.bounds()?.size, large);

    let fab = Fab::new(&root, icons::add(), "New")?;
    let icon = IconButton::new(&root, IconStyle::Filled, icons::edit(), "Edit")?;
    ui.refresh()?;
    assert_eq!(fab.bounds()?.size, Size::new(56.0, 56.0));
    assert_eq!(icon.bounds()?.size, Size::new(40.0, 40.0));
    assert_eq!(icon.text()?, "Edit");
    fab.set_extended(true)?;
    ui.refresh()?;
    assert!(fab.bounds()?.size.width > 56.0);
    Ok(())
}

#[test]
fn focus_shows_after_keys_only() -> Result {
    let ui = ui()?;
    let root = ui.root();
    root.set_align_items(Some(aegle_ui::Align::Start))?;
    let button = Button::new(&root, ButtonStyle::Filled, "Save")?;
    ui.refresh()?;
    click(&ui, &button)?;
    assert!(button.is_focused()?);
    assert!(!button.visual_state()?.focused);
    ui.key(KeyInput {
        key: Key::Escape,
        text: "",
        modifiers: Modifiers::default(),
        pressed: true,
        repeat: false,
    })?;
    assert!(button.visual_state()?.focused);
    Ok(())
}
