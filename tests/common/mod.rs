//! A headless Ui under an am3 theme with Aegle's bundled test font.

#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc, sync::Arc};

use aegle_text::{Blob, GenericFamily, TextSystem};
use aegle_ui::{Modifiers, Node, Point, PointerId, PointerKind, Result, Size, Ui};
use am3::Mode;

pub const SEED: aegle_ui::Color = aegle_ui::Color::rgb(0x67, 0x50, 0xA4);

pub fn ui() -> Result<Ui> {
    let mut fonts = TextSystem::new();
    let font = include_bytes!("../../../aegle/tests/assets/aegle-test-cjk.otf");
    let families = fonts.register_fonts(Blob::new(Arc::new(font.as_slice())))?;
    let ids = families.iter().map(|(id, _)| *id);
    fonts
        .collection_mut()
        .set_generic_families(GenericFamily::SansSerif, ids);
    let ui = Ui::with_fonts(Rc::new(RefCell::new(fonts)), am3::theme(SEED, Mode::Light))?;
    ui.resize(Size::new(600.0, 600.0))?;
    ui.root().set_align_items(Some(aegle_ui::Align::Start))?;
    Ok(ui)
}

pub fn center(node: &Node) -> Result<Point> {
    let b = node.bounds()?;
    Ok(Point::new(
        b.origin.x + b.size.width / 2.0,
        b.origin.y + b.size.height / 2.0,
    ))
}

pub fn press(ui: &Ui, node: &Node) -> Result {
    ui.refresh()?;
    let at = center(node)?;
    ui.pointer(
        PointerId(1),
        PointerKind::Down { clicks: 1 },
        at,
        Modifiers::default(),
    )
}

pub fn release(ui: &Ui, node: &Node) -> Result {
    let at = center(node)?;
    ui.pointer(PointerId(1), PointerKind::Up, at, Modifiers::default())?;
    ui.dispatch_callbacks()?;
    ui.refresh()?;
    Ok(())
}

pub fn click(ui: &Ui, node: &Node) -> Result {
    press(ui, node)?;
    release(ui, node)
}
