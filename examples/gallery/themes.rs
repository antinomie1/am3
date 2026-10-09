//! The gallery shot of color schemes: one small screen under several
//! seeds and in high contrast, light over dark.

use aegle_ui::{Container, Result, Ui};
use am3::*;

use crate::shots::Shot;

/// A small screen in `seed`'s scheme, in the row's light or dark mode.
fn themed(ui: &Ui, host: &Container, seed: aegle_ui::Color, high_contrast: bool) -> Result {
    let dark = ui.theme()?.background == theme(crate::SEED, Mode::Dark).background;
    let mode = match (dark, high_contrast) {
        (false, false) => Mode::Light,
        (true, false) => Mode::Dark,
        (false, true) => Mode::LightHighContrast,
        (true, true) => Mode::DarkHighContrast,
    };
    ui.set_theme(theme(seed, mode))?;
    let card = Card::new(host, CardStyle::Filled)?;
    card.set_width(210.0)?;
    Text::new(
        &card,
        tokens::typescale::TITLE_MEDIUM,
        Role::on_surface,
        "Trip to Lisbon",
    )?;
    let row = card.row()?;
    row.set_gap(8.0)?;
    Chip::new(&row, ChipKind::Filter, "Flights")?.set_selected(Some(true))?;
    Chip::new(&row, ChipKind::Filter, "Hotels")?;
    Slider::new(&card, 0.0, 1.0, 0.6)?;
    Switch::new(&card, "Alerts", true)?;
    let row = card.row()?;
    row.set_gap(8.0)?;
    Button::new(&row, ButtonStyle::Filled, "Book")?;
    Button::new(&row, ButtonStyle::Tonal, "Share")?;
    Fab::new(host, icons::edit(), "Edit").map(drop)
}

pub(crate) const THEMES: &[Shot] = &[(
    "color-schemes",
    (250.0, 400.0),
    &[
        ("#6750A4", |ui, h| {
            themed(ui, h, aegle_ui::Color::rgb(0x67, 0x50, 0xA4), false)
        }),
        ("#006A6A", |ui, h| {
            themed(ui, h, aegle_ui::Color::rgb(0x00, 0x6A, 0x6A), false)
        }),
        ("#8B5000", |ui, h| {
            themed(ui, h, aegle_ui::Color::rgb(0x8B, 0x50, 0x00), false)
        }),
        ("#6750A4, high contrast", |ui, h| {
            themed(ui, h, aegle_ui::Color::rgb(0x67, 0x50, 0xA4), true)
        }),
    ],
)];
