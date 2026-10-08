//! What the gallery shows: each component in its variants and states.

use aegle_ui::{Container, Result};
use am3::{
    Button, ButtonShape, ButtonSize, ButtonStyle, Fab, FabColor, FabSize, IconButton, IconStyle,
    IconWidth, icons,
};

use crate::{Setup, hover, press};

type Shot = (&'static str, (f32, f32), &'static [(&'static str, Setup)]);

fn styles(host: &Container) -> Result<aegle_ui::Container> {
    let row = host.row()?;
    row.set_gap(8.0)?;
    Ok(row)
}

pub(crate) const ALL: &[Shot] = &[
    (
        "buttons",
        (150.0, 96.0),
        &[
            ("elevated", |_, h| {
                Button::new(h, ButtonStyle::Elevated, "Elevated").map(drop)
            }),
            ("filled", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Filled").map(drop)
            }),
            ("tonal", |_, h| {
                Button::new(h, ButtonStyle::Tonal, "Tonal").map(drop)
            }),
            ("outlined", |_, h| {
                Button::new(h, ButtonStyle::Outlined, "Outlined").map(drop)
            }),
            ("text", |_, h| {
                Button::new(h, ButtonStyle::Text, "Text").map(drop)
            }),
            ("with icon", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Add")?.set_icon(Some(icons::add()))
            }),
        ],
    ),
    (
        "button-states",
        (130.0, 96.0),
        &[
            ("enabled", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Button").map(drop)
            }),
            ("hovered", |ui, h| {
                hover(ui, &Button::new(h, ButtonStyle::Filled, "Button")?)
            }),
            ("focused", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Button")?.focus()
            }),
            ("pressed", |ui, h| {
                press(ui, &Button::new(h, ButtonStyle::Filled, "Button")?)
            }),
            ("disabled", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Button")?.set_enabled(false)
            }),
        ],
    ),
    (
        "button-sizes",
        (300.0, 180.0),
        &[
            ("XS · S · M", |_, h| {
                let row = styles(h)?;
                row.set_align_items(Some(aegle_ui::Align::Center))?;
                for size in [
                    ButtonSize::ExtraSmall,
                    ButtonSize::Small,
                    ButtonSize::Medium,
                ] {
                    let b = Button::new(&row, ButtonStyle::Filled, "Label")?;
                    b.set_size(size)?;
                }
                Ok(())
            }),
            ("L", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Label")?.set_size(ButtonSize::Large)
            }),
            ("XL", |_, h| {
                Button::new(h, ButtonStyle::Filled, "Label")?.set_size(ButtonSize::ExtraLarge)
            }),
        ],
    ),
    (
        "toggle-buttons",
        (300.0, 96.0),
        &[
            ("unselected", |_, h| {
                let row = styles(h)?;
                for style in [
                    ButtonStyle::Elevated,
                    ButtonStyle::Filled,
                    ButtonStyle::Tonal,
                    ButtonStyle::Outlined,
                ] {
                    Button::new(&row, style, "Off")?.set_selected(Some(false))?;
                }
                Ok(())
            }),
            ("selected", |_, h| {
                let row = styles(h)?;
                for style in [
                    ButtonStyle::Elevated,
                    ButtonStyle::Filled,
                    ButtonStyle::Tonal,
                    ButtonStyle::Outlined,
                ] {
                    Button::new(&row, style, "On")?.set_selected(Some(true))?;
                }
                Ok(())
            }),
            ("square shape", |_, h| {
                let row = styles(h)?;
                for style in [
                    ButtonStyle::Elevated,
                    ButtonStyle::Filled,
                    ButtonStyle::Tonal,
                    ButtonStyle::Outlined,
                ] {
                    Button::new(&row, style, "Sq")?.set_shape(ButtonShape::Square)?;
                }
                Ok(())
            }),
        ],
    ),
    (
        "icon-buttons",
        (220.0, 96.0),
        &[
            ("standard · filled · tonal · outlined", |_, h| {
                let row = styles(h)?;
                for style in [
                    IconStyle::Standard,
                    IconStyle::Filled,
                    IconStyle::Tonal,
                    IconStyle::Outlined,
                ] {
                    IconButton::new(&row, style, icons::favorite(), "Favorite")?;
                }
                Ok(())
            }),
            ("toggles, selected", |_, h| {
                let row = styles(h)?;
                for style in [
                    IconStyle::Standard,
                    IconStyle::Filled,
                    IconStyle::Tonal,
                    IconStyle::Outlined,
                ] {
                    IconButton::new(&row, style, icons::favorite(), "Favorite")?
                        .set_selected(Some(true))?;
                }
                Ok(())
            }),
            ("narrow · default · wide", |_, h| {
                let row = styles(h)?;
                for width in [IconWidth::Narrow, IconWidth::Default, IconWidth::Wide] {
                    IconButton::new(&row, IconStyle::Tonal, icons::edit(), "Edit")?
                        .set_width(width)?;
                }
                Ok(())
            }),
        ],
    ),
    (
        "fabs",
        (170.0, 150.0),
        &[
            ("FAB", |_, h| {
                Fab::new(h, icons::edit(), "Compose").map(drop)
            }),
            ("medium", |_, h| {
                Fab::new(h, icons::edit(), "Compose")?.set_size(FabSize::Medium)
            }),
            ("large", |_, h| {
                Fab::new(h, icons::edit(), "Compose")?.set_size(FabSize::Large)
            }),
            ("tertiary", |_, h| {
                Fab::new(h, icons::add(), "New")?.set_color(FabColor::Tertiary)
            }),
            ("extended", |_, h| {
                Fab::extended(h, icons::edit(), "Compose").map(drop)
            }),
        ],
    ),
];
