//! Gallery shots of input components: text fields, search, and the date
//! and time pickers.

use aegle_ui::{Container, Result, Ui};
use am3::*;

use crate::{pointer_mode, shots::Shot};

fn fields(_: &Ui, host: &Container, style: FieldStyle) -> Result {
    let column = host.column()?;
    column.set_gap(16.0)?;
    TextField::new(&column, style, "Name")?;
    let focused = TextField::new(&column, style, "Email")?;
    focused.set_text("ada@example.com")?;
    focused.set_supporting(Some("We never share it"))?;
    focused.focus()?;
    let error = TextField::new(&column, style, "Password")?;
    error.set_text("hunter2")?;
    error.set_password(true)?;
    error.set_error(true)?;
    error.set_supporting(Some("At least 12 characters"))?;
    let search = TextField::new(&column, style, "City")?;
    search.set_leading_icon(Some(icons::search()))?;
    search.set_text("Lisbon")?;
    search.set_clearable(true)?;
    let bio = TextField::multiline(&column, style, "Bio")?;
    bio.set_text("Mathematician and writer,\nknown for her notes on the Analytical Engine.")?;
    bio.set_counter(Some(120))
}

fn search(ui: &Ui, host: &Container) -> Result {
    let search = Search::new(host, "Search mail")?;
    search.bar().set_trailing_icon(Some(icons::mic()))?;
    let list = List::new(search.results())?;
    list.set_background(aegle_ui::Color::TRANSPARENT)?;
    for text in [
        "Lunch on Friday?",
        "Lunar eclipse photos",
        "Lund conference",
    ] {
        list.clickable_item(text)?.leading_icon(icons::schedule())?;
    }
    ui.refresh()?;
    search.bar().focus()?;
    ui.paste("Lun")?;
    ui.dispatch_callbacks()
}

const DAY: Date = match Date::new(2026, 8, 17) {
    Some(day) => day,
    None => unreachable!(),
};

fn docked_date(_: &Ui, host: &Container, range: bool) -> Result {
    let picker = DatePicker::docked(host)?;
    picker.set_today(DAY)?;
    if range {
        picker.set_selected_range(DAY.add_months(0), Date::from_days(DAY.days() + 5))
    } else {
        picker.set_selected(Some(Date::from_days(DAY.days() + 3)))
    }
}

fn modal_date(ui: &Ui, host: &Container) -> Result {
    let picker = DatePicker::modal(host)?;
    picker.set_today(DAY)?;
    picker.set_selected(Some(DAY))?;
    picker.on_confirm(|_| Ok(()))?;
    pointer_mode(ui)?;
    picker.show()
}

fn time(_: &Ui, host: &Container, h24: bool) -> Result {
    let picker = TimePicker::inline(host)?;
    picker.set_time(if h24 { 19 } else { 7 }, 30)?;
    picker.set_24_hour(h24)
}

fn carousel(_: &Ui, host: &Container) -> Result {
    host.set_align_items(Some(aegle_ui::Align::Stretch))?;
    let s = Scheme::of(&host.theme()?);
    let carousel = Carousel::new(host, 180.0, 200.0)?;
    for (title, color, role) in [
        ("Lisbon", s.primary_container, Role::on_primary_container),
        ("Kyoto", s.tertiary_container, Role::on_tertiary_container),
        (
            "Oaxaca",
            s.secondary_container,
            Role::on_secondary_container,
        ),
    ] {
        let item = carousel.item(true)?;
        item.set_background(color)?;
        Text::new(&item, tokens::typescale::TITLE_LARGE, role, title)?;
    }
    carousel.scroll_to(60.0)
}

pub(crate) const INPUTS: &[Shot] = &[
    (
        "text-fields",
        (330.0, 520.0),
        &[
            ("filled", |ui, h| fields(ui, h, FieldStyle::Filled)),
            ("outlined", |ui, h| fields(ui, h, FieldStyle::Outlined)),
        ],
    ),
    ("search", (420.0, 330.0), &[("search bar and view", search)]),
    (
        "date-pickers",
        (390.0, 560.0),
        &[
            ("docked", |ui, h| docked_date(ui, h, false)),
            ("docked, range", |ui, h| docked_date(ui, h, true)),
            ("modal", modal_date),
        ],
    ),
    (
        "time-pickers",
        (380.0, 520.0),
        &[
            ("12-hour", |ui, h| time(ui, h, false)),
            ("24-hour", |ui, h| time(ui, h, true)),
        ],
    ),
    ("carousel", (420.0, 260.0), &[("uncontained", carousel)]),
];
