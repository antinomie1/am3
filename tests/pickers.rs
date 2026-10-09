//! Date and time pickers, search and carousels: calendar arithmetic, days
//! chosen by pointer and keyboard (single and range), the dial's rings in
//! 12- and 24-hour time, the search view following the bar's text, and
//! the carousel turning the wheel into horizontal scrolling.

mod common;

use std::time::Instant;

use aegle_ui::{Key, Modifiers, Point, Result};
use am3::{Carousel, Date, DatePicker, Search, TimePicker};
use common::{click_at, key, ui};

#[test]
fn date_arithmetic() {
    let day = Date::new(2026, 8, 17).unwrap();
    assert_eq!(Date::from_days(day.days()), day);
    assert_eq!(Date::from_days(0).to_string(), "1970-01-01");
    assert_eq!(day.weekday(), 1, "a Monday");
    assert_eq!(day.headline(), "Mon, Aug 17");
    assert_eq!(Date::new(2024, 2, 29).map(|d| d.weekday()), Some(4));
    assert_eq!(Date::new(2026, 2, 29), None);
    let end = Date::new(2026, 1, 31).unwrap();
    assert_eq!(end.add_months(1).to_string(), "2026-02-28", "clamped");
    assert_eq!(end.add_months(-13).to_string(), "2024-12-31");
}

/// The center of `day`'s cell in a calendar showing its month.
fn cell(picker: &DatePicker, day: u8) -> Result<Point> {
    let b = picker.calendar().bounds()?;
    let first = Date::new(2026, 2, 1).unwrap();
    let index = i32::from(first.weekday()) + i32::from(day) - 1;
    Ok(Point::new(
        b.origin.x + (index % 7) as f32 * 48.0 + 24.0,
        b.origin.y + 40.0 + (index / 7) as f32 * 48.0 + 24.0,
    ))
}

#[test]
fn days_by_pointer_and_keyboard() -> Result {
    let ui = ui()?;
    let picker = DatePicker::docked(&ui.root())?;
    picker.show_month(Date::new(2026, 2, 1).unwrap())?;
    let picks = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = picks.clone();
    picker.on_change(move |_| {
        seen.set(seen.get() + 1);
        Ok(())
    })?;
    ui.refresh()?;
    click_at(&ui, cell(&picker, 10)?)?;
    assert_eq!(picker.selected()?, Date::new(2026, 2, 10));
    assert_eq!(picks.get(), 1);

    // The cursor starts on the selection; the arrows move it by days and
    // weeks, PageDown by a month, and Enter chooses.
    picker.calendar().focus()?;
    key(&ui, Key::Right)?;
    key(&ui, Key::Down)?;
    key(&ui, Key::Enter)?;
    assert_eq!(picker.selected()?, Date::new(2026, 2, 18));
    key(&ui, Key::PageDown)?;
    key(&ui, Key::Enter)?;
    assert_eq!(picker.selected()?, Date::new(2026, 3, 18));
    assert_eq!(picks.get(), 3);

    picker.show_month(Date::new(2026, 2, 1).unwrap())?;
    picker.set_range(true)?;
    click_at(&ui, cell(&picker, 12)?)?;
    assert_eq!(picker.range()?, None, "a start awaits its end");
    click_at(&ui, cell(&picker, 9)?)?;
    assert_eq!(picker.range()?, None, "an earlier day restarts the range");
    click_at(&ui, cell(&picker, 12)?)?;
    let (start, end) = (Date::new(2026, 2, 9), Date::new(2026, 2, 12));
    assert_eq!(picker.range()?, start.zip(end));
    Ok(())
}

#[test]
fn modal_headline_follows_the_selection() -> Result {
    let ui = ui()?;
    let picker = DatePicker::modal(&ui.root())?;
    picker.set_selected(Date::new(2026, 8, 17))?;
    picker.show()?;
    ui.refresh()?;
    let headline = picker.headline().unwrap();
    assert_eq!(headline.text()?, "Mon, Aug 17");
    picker.calendar().focus()?;
    key(&ui, Key::Right)?;
    key(&ui, Key::Enter)?;
    assert_eq!(headline.text()?, "Tue, Aug 18");
    picker.set_range(true)?;
    key(&ui, Key::Right)?;
    key(&ui, Key::Enter)?;
    assert_eq!(headline.text()?, "Aug 18 – Aug 19");
    Ok(())
}

/// A point on the dial at `hour` o'clock, `radius` dp from its center.
fn on_dial(picker: &TimePicker, hour: f32, radius: f32) -> Result<Point> {
    let b = picker.dial().bounds()?;
    let angle = hour / 12.0 * std::f32::consts::TAU;
    Ok(Point::new(
        b.origin.x + 128.0 + radius * angle.sin(),
        b.origin.y + 128.0 - radius * angle.cos(),
    ))
}

#[test]
fn dial_rings_and_modes() -> Result {
    let ui = ui()?;
    let picker = TimePicker::inline(&ui.root())?;
    picker.set_time(19, 30)?;
    ui.refresh()?;
    // A 12-hour hour keeps its period; choosing it moves on to minutes.
    click_at(&ui, on_dial(&picker, 3.0, 100.0)?)?;
    assert_eq!(picker.time()?, (15, 30));
    click_at(&ui, on_dial(&picker, 9.0, 100.0)?)?;
    assert_eq!(picker.time()?, (15, 45));
    picker.dial().focus()?;
    key(&ui, Key::Up)?;
    assert_eq!(picker.time()?, (15, 50), "minutes step by five");

    Ok(())
}

#[test]
fn dial_rings_in_24_hour_time() -> Result {
    let ui = ui()?;
    ui.resize(aegle_ui::Size::new(800.0, 600.0))?;
    ui.root().set_direction(aegle_ui::Direction::Row)?;
    // The outer ring holds 00–11, the inner 12–23.
    for (radius, hour) in [(100.0, 0), (68.0, 12)] {
        let picker = TimePicker::inline(&ui.root())?;
        picker.set_24_hour(true)?;
        picker.set_time(7, 0)?;
        ui.refresh()?;
        click_at(&ui, on_dial(&picker, 0.0, radius)?)?;
        assert_eq!(picker.time()?.0, hour);
    }
    Ok(())
}

#[test]
fn search_view_follows_the_text() -> Result {
    let ui = ui()?;
    let search = Search::new(&ui.root(), "Search")?;
    ui.refresh()?;
    search.bar().focus()?;
    ui.paste("lis")?;
    ui.dispatch_callbacks()?;
    assert!(search.results_shown()?);
    assert!(search.bar().is_focused()?, "typing continues in the bar");
    search.bar().set_text("")?;
    key(&ui, Key::Backspace)?;
    search.hide_results()?;
    assert!(!search.results_shown()?);
    Ok(())
}

#[test]
fn carousel_scrolls_sideways() -> Result {
    let ui = ui()?;
    ui.root().set_align_items(Some(aegle_ui::Align::Stretch))?;
    let carousel = Carousel::new(&ui.root(), 200.0, 160.0)?;
    for _ in 0..5 {
        carousel.item(true)?;
    }
    ui.refresh()?;
    let b = carousel.bounds()?;
    let inside = Point::new(b.origin.x + 100.0, b.origin.y + 80.0);
    ui.wheel(
        inside,
        Point::new(0.0, 120.0),
        Modifiers::default(),
        Instant::now(),
    )?;
    ui.refresh()?;
    assert_eq!(carousel.offset()?, 120.0, "vertical wheel scrolls sideways");
    // 5 items of 200 dp, 8 dp apart, with 16 dp padding: 1064 dp.
    carousel.scroll_to(5000.0)?;
    assert_eq!(carousel.offset()?, 1064.0 - b.size.width);
    Ok(())
}
