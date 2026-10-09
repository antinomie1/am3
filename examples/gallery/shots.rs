//! What the gallery shows: each component in its variants and states.

use aegle_ui::{Container, Result};
use am3::*;

use crate::{Setup, hover, press};

pub(crate) type Shot = (&'static str, (f32, f32), &'static [(&'static str, Setup)]);

fn styles(host: &Container) -> Result<aegle_ui::Container> {
    let row = host.row()?;
    row.set_gap(8.0)?;
    Ok(row)
}

fn menu(host: &Container) -> Result<FabMenu> {
    host.set_align_items(Some(aegle_ui::Align::End))?;
    let m = FabMenu::new(host, icons::add(), "Create", FabColor::PrimaryContainer)?;
    m.item(icons::mail(), "Message")?;
    m.item(icons::calendar(), "Event")?;
    m.item(icons::edit(), "Note")?;
    Ok(m)
}

/// Indicators started apart, so the settled frame shows three shapes.
fn loading(ui: &aegle_ui::Ui, host: &Container, contained: bool) -> Result {
    let row = styles(host)?;
    let start = std::time::Instant::now();
    for delay in [0.1, 0.75, 1.4] {
        ui.run_frame(start + std::time::Duration::from_secs_f32(delay))?;
        LoadingIndicator::new(&row, contained)?;
        ui.refresh()?;
    }
    Ok(())
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
    (
        "fab-menu",
        (220.0, 340.0),
        &[
            ("closed", |_, h| menu(h).map(drop)),
            ("open", |_, h| menu(h)?.set_open(true)),
            ("open, tertiary", |_, h| {
                h.set_align_items(Some(aegle_ui::Align::End))?;
                let m = FabMenu::new(h, icons::add(), "Create", FabColor::TertiaryContainer)?;
                m.item(icons::photo(), "Photo")?;
                m.item(icons::mic(), "Recording")?;
                m.set_open(true)
            }),
        ],
    ),
    (
        "button-groups",
        (330.0, 110.0),
        &[
            ("standard", |_, h| {
                let g = ButtonGroup::new(h, GroupStyle::Standard, ButtonSize::Small)?;
                g.icon_button(IconStyle::Tonal, icons::mail(), "Mail")?;
                g.button(ButtonStyle::Filled, "Inbox")?;
                g.icon_button(IconStyle::Tonal, icons::star(), "Starred")?;
                Ok(())
            }),
            ("standard, middle pressed", |ui, h| {
                let g = ButtonGroup::new(h, GroupStyle::Standard, ButtonSize::Small)?;
                g.icon_button(IconStyle::Tonal, icons::mail(), "Mail")?;
                let b = g.button(ButtonStyle::Filled, "Inbox")?;
                g.icon_button(IconStyle::Tonal, icons::star(), "Starred")?;
                press(ui, &b)
            }),
            ("connected, single select", |_, h| {
                let g = ButtonGroup::new(h, GroupStyle::Connected, ButtonSize::Small)?;
                for day in ["Day", "Week", "Month"] {
                    g.button(ButtonStyle::Tonal, day)?;
                }
                g.set_selection(Selection::Single)
            }),
        ],
    ),
    (
        "split-buttons",
        (210.0, 96.0),
        &[
            ("filled", |_, h| {
                SplitButton::new(h, ButtonStyle::Filled, "Save").map(drop)
            }),
            ("tonal, hovered", |ui, h| {
                let s = SplitButton::new(h, ButtonStyle::Tonal, "Save")?;
                hover(ui, s.action())
            }),
            ("outlined, menu open", |_, h| {
                let s = SplitButton::new(h, ButtonStyle::Outlined, "Save")?;
                s.set_menu_open(true)
            }),
            ("elevated", |_, h| {
                SplitButton::new(h, ButtonStyle::Elevated, "Save").map(drop)
            }),
        ],
    ),
    (
        "segmented-buttons",
        (300.0, 96.0),
        &[
            ("single select", |_, h| {
                let s = SegmentedButton::new(h, false)?;
                for t in ["Day", "Week", "Month"] {
                    s.segment(t, None)?;
                }
                Ok(())
            }),
            ("multi select, icons", |_, h| {
                let s = SegmentedButton::new(h, true)?;
                s.segment("Walk", Some(icons::person()))?
                    .set_selected(Some(true))?;
                s.segment("Ride", Some(icons::home()))?;
                s.segment("Fly", Some(icons::send()))?
                    .set_selected(Some(true))?;
                Ok(())
            }),
        ],
    ),
    (
        "checkboxes",
        (150.0, 96.0),
        &[
            ("unchecked", |_, h| {
                Checkbox::new(h, "Label", false).map(drop)
            }),
            ("checked", |_, h| Checkbox::new(h, "Label", true).map(drop)),
            ("indeterminate", |_, h| {
                Checkbox::new(h, "Label", true)?.set_mixed(true)
            }),
            ("error", |_, h| {
                Checkbox::new(h, "Label", true)?.set_error(true)
            }),
            ("hovered", |ui, h| {
                hover(ui, &Checkbox::new(h, "Label", false)?)
            }),
            ("disabled", |_, h| {
                Checkbox::new(h, "Label", true)?.set_enabled(false)
            }),
        ],
    ),
    (
        "radio-buttons",
        (150.0, 96.0),
        &[
            ("unselected", |_, h| Radio::new(h, "Label", false).map(drop)),
            ("selected", |_, h| Radio::new(h, "Label", true).map(drop)),
            ("focused", |_, h| Radio::new(h, "Label", true)?.focus()),
            ("pressed", |ui, h| {
                press(ui, &Radio::new(h, "Label", false)?)
            }),
            ("disabled", |_, h| {
                Radio::new(h, "Label", true)?.set_enabled(false)
            }),
        ],
    ),
    (
        "switches",
        (150.0, 96.0),
        &[
            ("off", |_, h| Switch::new(h, "", false).map(drop)),
            ("on", |_, h| Switch::new(h, "", true).map(drop)),
            ("icons", |_, h| {
                let row = styles(h)?;
                for on in [false, true] {
                    let s = Switch::new(&row, "", on)?;
                    s.set_icons(Some(icons::check()), Some(icons::close()))?;
                }
                Ok(())
            }),
            ("pressed", |ui, h| press(ui, &Switch::new(h, "", true)?)),
            ("labelled", |_, h| Switch::new(h, "Wi-Fi", true).map(drop)),
            ("disabled", |_, h| {
                let row = styles(h)?;
                Switch::new(&row, "", false)?.set_enabled(false)?;
                Switch::new(&row, "", true)?.set_enabled(false)
            }),
        ],
    ),
    (
        "sliders",
        (260.0, 150.0),
        &[
            ("continuous", |_, h| {
                Slider::new(h, 0.0, 100.0, 40.0).map(drop)
            }),
            ("discrete, ticks", |_, h| {
                let s = Slider::new(h, 0.0, 10.0, 6.0)?;
                s.set_step(1.0, true)
            }),
            ("range", |_, h| {
                let s = Slider::new(h, 0.0, 100.0, 80.0)?;
                s.set_range(true)?;
                s.set_start(25.0)
            }),
            ("centered", |_, h| {
                let s = Slider::new(h, -50.0, 50.0, 20.0)?;
                s.set_centered(true)
            }),
            ("dragging", |ui, h| {
                h.set_padding(52.0)?;
                press(ui, &Slider::new(h, 0.0, 100.0, 50.0)?)
            }),
            ("disabled", |_, h| {
                Slider::new(h, 0.0, 100.0, 40.0)?.set_enabled(false)
            }),
        ],
    ),
    (
        "slider-sizes",
        (260.0, 140.0),
        &[
            ("S", |_, h| {
                Slider::new(h, 0.0, 1.0, 0.5)?.set_size(SliderSize::Small)
            }),
            ("M", |_, h| {
                Slider::new(h, 0.0, 1.0, 0.5)?.set_size(SliderSize::Medium)
            }),
            ("L", |_, h| {
                Slider::new(h, 0.0, 1.0, 0.5)?.set_size(SliderSize::Large)
            }),
            ("XL", |_, h| {
                Slider::new(h, 0.0, 1.0, 0.5)?.set_size(SliderSize::ExtraLarge)
            }),
        ],
    ),
    (
        "chips",
        (190.0, 96.0),
        &[
            ("assist", |_, h| {
                Chip::new(h, ChipKind::Assist, "Add to calendar")?.set_icon(Some(icons::calendar()))
            }),
            ("assist, elevated", |_, h| {
                let c = Chip::new(h, ChipKind::Assist, "Directions")?;
                c.set_icon(Some(icons::send()))?;
                c.set_elevated(true)
            }),
            ("filter", |_, h| {
                Chip::new(h, ChipKind::Filter, "Vegan").map(drop)
            }),
            ("filter, selected", |_, h| {
                Chip::new(h, ChipKind::Filter, "Vegan")?.set_selected(Some(true))
            }),
            ("input", |_, h| {
                Chip::new(h, ChipKind::Input, "Ada")?.set_icon(Some(icons::person()))
            }),
            ("suggestion", |_, h| {
                Chip::new(h, ChipKind::Suggestion, "Sounds good").map(drop)
            }),
        ],
    ),
    (
        "progress",
        (270.0, 110.0),
        &[
            ("linear 40 %", |_, h| {
                Progress::linear(h, Some(0.4)).map(drop)
            }),
            ("linear, wavy", |_, h| {
                Progress::linear(h, Some(0.6))?.set_wavy(true)
            }),
            ("linear, indeterminate", |_, h| {
                Progress::linear(h, None).map(drop)
            }),
            ("circular", |_, h| {
                let row = styles(h)?;
                Progress::circular(&row, Some(0.3))?;
                Progress::circular(&row, Some(0.7))?.set_wavy(true)?;
                Progress::circular(&row, None).map(drop)
            }),
        ],
    ),
    (
        "loading",
        (300.0, 100.0),
        &[
            ("loading indicator", |ui, h| loading(ui, h, false)),
            ("contained", |ui, h| loading(ui, h, true)),
        ],
    ),
];
