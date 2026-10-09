//! Switching themes on a screen of components: every mode and another
//! seed recolor them from the new scheme without rebuilding or relayout,
//! editing state survives, a subtree can stay dark inside a light window,
//! and bound color tokens follow along.

mod common;

use aegle_ui::{Color, ColorSlot, Container, Result};
use am3::{
    Button, ButtonStyle, Card, CardStyle, Chip, ChipKind, FieldStyle, Mode, NavigationBar, Role,
    Scheme, Switch, TextField, icons, theme,
};
use common::{SEED, ui};

struct Screen {
    button: Button,
    card: Card,
    field: TextField,
    switch: Switch,
    chip: Chip,
    bar: NavigationBar,
}

fn screen(parent: &Container) -> Result<Screen> {
    let card = Card::new(parent, CardStyle::Elevated)?;
    let field = TextField::new(&card, FieldStyle::Outlined, "Name")?;
    let bar = NavigationBar::new(parent)?;
    bar.item(icons::mail(), "Mail")?;
    Ok(Screen {
        button: Button::new(&card, ButtonStyle::Filled, "Save")?,
        switch: Switch::new(&card, "Wi-Fi", true)?,
        chip: Chip::new(&card, ChipKind::Filter, "Starred")?,
        card,
        field,
        bar,
    })
}

/// The scheme controls draw a theme with.
fn scheme(seed: Color, mode: Mode) -> Scheme {
    Scheme::of(&theme(seed, mode))
}

/// Checks the screen's resting colors against `s`.
fn follows(screen: &Screen, s: &Scheme) -> Result {
    assert_eq!(screen.button.appearance()?.background, s.primary);
    assert_eq!(
        screen.card.appearance()?.background,
        s.surface_container_low
    );
    assert_eq!(screen.bar.appearance()?.background, s.surface_container);
    assert_eq!(screen.field.appearance()?.border_color, s.outline);
    assert_eq!(
        screen.switch.appearance()?.indicator,
        s.primary,
        "a selected track"
    );
    assert_eq!(screen.chip.appearance()?.border_color, s.outline_variant);
    Ok(())
}

#[test]
fn every_mode_and_seed_recolors_the_screen() -> Result {
    let ui = ui()?;
    let screen = screen(&ui.root())?;
    screen.field.set_text("Ada")?;
    ui.refresh()?;
    let size = screen.card.bounds()?;
    let teal = Color::rgb(0x00, 0x6A, 0x6A);
    for seed in [SEED, teal] {
        for mode in [
            Mode::Light,
            Mode::Dark,
            Mode::LightHighContrast,
            Mode::DarkHighContrast,
        ] {
            ui.set_theme(theme(seed, mode))?;
            ui.refresh()?;
            follows(&screen, &scheme(seed, mode))?;
            assert_eq!(screen.card.bounds()?, size, "colors alone change");
        }
    }
    assert_eq!(screen.field.text()?, "Ada", "editing state survives");
    Ok(())
}

#[test]
fn a_dark_subtree_in_a_light_window() -> Result {
    let ui = ui()?;
    let light = screen(&ui.root())?;
    let panel = ui.root().column()?;
    let dark = screen(&panel)?;
    panel.set_theme(Some(theme(SEED, Mode::Dark)))?;
    ui.refresh()?;
    follows(&light, &scheme(SEED, Mode::Light))?;
    follows(&dark, &scheme(SEED, Mode::Dark))?;
    // A new seed for the window leaves the snapshot alone.
    ui.set_theme(theme(Color::rgb(0x00, 0x6A, 0x6A), Mode::Light))?;
    follows(&dark, &scheme(SEED, Mode::Dark))?;
    Ok(())
}

#[test]
fn bound_tokens_follow_the_theme() -> Result {
    let ui = ui()?;
    let swatch = ui.root().column()?;
    swatch.bind_color(ColorSlot::Background, Role::tertiary_container.token())?;
    ui.refresh()?;
    let tertiary = |mode| scheme(SEED, mode).tertiary_container;
    assert_eq!(swatch.appearance()?.background, tertiary(Mode::Light));
    ui.set_theme(theme(SEED, Mode::Dark))?;
    assert_eq!(swatch.appearance()?.background, tertiary(Mode::Dark));
    Ok(())
}
