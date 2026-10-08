//! Groups of buttons: a single-select group keeps exactly one selected, a
//! pressed button of a standard group reaches into its neighbors and
//! returns, split buttons report their menu, segments report their
//! selection, and a FAB menu opens and closes.

mod common;

use std::time::{Duration, Instant};

use aegle_ui::Result;
use am3::{
    ButtonGroup, ButtonSize, ButtonStyle, FabColor, FabMenu, GroupStyle, SegmentedButton,
    Selection, SplitButton, icons,
};
use common::{click, press, release, ui};

#[test]
fn single_select_groups_keep_one_selected() -> Result {
    let ui = ui()?;
    let group = ButtonGroup::new(&ui.root(), GroupStyle::Connected, ButtonSize::Small)?;
    let days: Vec<_> = ["Day", "Week", "Month"]
        .into_iter()
        .map(|d| group.button(ButtonStyle::Tonal, d))
        .collect::<Result<_>>()?;
    group.set_selection(Selection::Single)?;
    ui.refresh()?;
    let selected = || -> Result<Vec<_>> {
        days.iter()
            .map(|d| Ok(d.selected()? == Some(true)))
            .collect()
    };
    assert_eq!(selected()?, [true, false, false]);
    click(&ui, &days[2])?;
    assert_eq!(selected()?, [false, false, true]);
    // Choosing the chosen one changes nothing.
    click(&ui, &days[2])?;
    assert_eq!(selected()?, [false, false, true]);
    // Connected buttons sit 2 dp apart.
    let (a, b) = (days[0].bounds()?, days[1].bounds()?);
    assert_eq!(b.origin.x - (a.origin.x + a.size.width), 2.0);
    Ok(())
}

#[test]
fn standard_groups_bounce_and_return() -> Result {
    let ui = ui()?;
    let group = ButtonGroup::new(&ui.root(), GroupStyle::Standard, ButtonSize::Small)?;
    let left = group.button(ButtonStyle::Tonal, "A")?;
    let middle = group.button(ButtonStyle::Filled, "Middle")?;
    group.button(ButtonStyle::Tonal, "B")?;
    // The right edge of the left button's painting shows how far the
    // middle one's press reaches into it.
    let painted = |ui: &aegle_ui::Ui| -> Result<f32> {
        let mut right = 0.0f32;
        ui.visit_scenes(|visit| {
            if let aegle_ui::Visit::Scene {
                scene, transform, ..
            } = visit
                && let Some(b) = scene.bounds()
                && transform.map_point(b.origin).x < 30.0
            {
                let far = aegle_ui::Point::new(b.origin.x + b.size.width, b.origin.y);
                right = right.max(transform.map_point(far).x);
            }
            Ok(())
        })?;
        Ok(right)
    };
    ui.refresh()?;
    let rest = painted(&ui)?;
    let before = left.bounds()?.size;
    press(&ui, &middle)?;
    ui.refresh()?;
    ui.run_frame(Instant::now() + Duration::from_secs(1))?;
    ui.refresh()?;
    let pressed = painted(&ui)?;
    assert!(pressed < rest - 2.0, "{rest} → {pressed}");
    // The layout does not move: only the painting reaches.
    assert_eq!(left.bounds()?.size, before);
    release(&ui, &middle)?;
    ui.refresh()?;
    ui.run_frame(Instant::now() + Duration::from_secs(3))?;
    ui.refresh()?;
    let back = painted(&ui)?;
    assert!((back - rest).abs() < 0.5, "{rest} → {pressed} → {back}");
    Ok(())
}

#[test]
fn split_segments_and_fab_menus_report() -> Result {
    let ui = ui()?;
    let root = ui.root();
    let split = SplitButton::new(&root, ButtonStyle::Filled, "Save")?;
    let opened = std::rc::Rc::new(std::cell::Cell::new(None));
    let seen = opened.clone();
    split.menu().on_click(move |menu| {
        seen.set(menu.selected()?);
        Ok(())
    })?;
    ui.refresh()?;
    click(&ui, split.menu())?;
    assert_eq!(opened.get(), Some(true));
    // The menu part keeps its colors when open.
    assert_eq!(split.menu().visual_state()?.kind.name, "FilledButton");

    let segments = SegmentedButton::new(&root, true)?;
    let walk = segments.segment("Walk", None)?;
    segments.segment("Ride", None)?;
    let fly = segments.segment("Fly", None)?;
    ui.refresh()?;
    click(&ui, &walk)?;
    click(&ui, &fly)?;
    assert_eq!(segments.selected()?, [0, 2]);

    let menu = FabMenu::new(&root, icons::add(), "Create", FabColor::PrimaryContainer)?;
    let note = menu.item(icons::edit(), "Note")?;
    ui.refresh()?;
    assert!(note.bounds()?.size.width == 0.0 || !menu.is_open()?);
    click(&ui, menu.fab())?;
    assert!(menu.is_open()?);
    ui.advance_animations(Duration::from_secs(2))?;
    ui.refresh()?;
    assert!(note.bounds()?.size.width > 0.0);
    click(&ui, &note)?;
    assert!(!menu.is_open()?);
    Ok(())
}
