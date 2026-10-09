//! An interactive window of the am3 components, built in Rust or in markup.
//!
//! A navigation rail pages through buttons, selection, inputs, containers
//! and overlays, and navigation; every control is live: dialogs and sheets
//! open and close, pickers pick and text fields take IME input. The top row
//! switches the seed color, dark and high-contrast themes and reduced motion
//! on the running window.
//!
//! By default the interface is built with the Rust API (`frame.rs` and one
//! module per page). `--markup` builds the same interface from
//! `showcase.aegle` and the page components it imports, compiled by Aegle's
//! `ui!`. Both versions hand the same controls to `shell::wire`, which also
//! adds the menus and tooltips that have no markup elements.
//!
//! `cargo run -p am3-showcase [-- --markup]`
mod buttons;
mod frame;
mod inputs;
mod navigation;
mod selection;
mod shell;
mod surfaces;

use std::rc::Rc;

use aegle::{App, AppOptions, Result};
use am3::Mode;

fn main() -> Result {
    let seed = shell::SEEDS[0].1;
    let app = App::with_options(AppOptions {
        app_id: "org.am3.showcase".into(),
        theme: am3::theme(seed, Mode::Light),
        dark_theme: Some(am3::theme(seed, Mode::Dark)),
        high_contrast_theme: Some(am3::theme(seed, Mode::DarkHighContrast)),
        ..Default::default()
    })?;
    let shell = if std::env::args().any(|arg| arg == "--markup") {
        markup(&app)?
    } else {
        frame::build(&app)?
    };
    shell::wire(shell, app.preferences())?;
    app.run()
}

/// The same interface from `showcase.aegle`.
fn markup(app: &App) -> Result<shell::Shell> {
    use aegle::prelude::{Column, Row};
    use am3::{
        MdAction, MdBottomAppBar, MdButton, MdButtonGroup, MdCard, MdCarousel, MdCarouselItem,
        MdCheckbox, MdChip, MdDatePicker, MdDialog, MdDialogAction, MdDivider, MdFab, MdFabMenu,
        MdFabMenuItem, MdGroupButton, MdIconButton, MdList, MdListItem, MdLoadingIndicator,
        MdNavItem, MdNavigationBar, MdNavigationDrawer, MdNavigationRail, MdProgress, MdRadio,
        MdScrollView, MdSearch, MdSegment, MdSegmentedButton, MdSheet, MdSlider, MdSnackbar,
        MdSnackbarAction, MdSplitButton, MdSwitch, MdTab, MdTabs, MdText, MdTextField,
        MdTimePicker, MdToolbar, MdTopAppBar,
    };
    let view = aegle::ui!(app, "showcase.aegle")?;
    let last = view.last.clone();
    Ok(shell::Shell {
        window: view.root.clone(),
        seeds: vec![view.seed0, view.seed1, view.seed2, view.seed3],
        dark: view.dark,
        contrast: view.contrast,
        reduced: view.reduced,
        menu: view.menu,
        split: view.split,
        tooltip: view.tooltip,
        rich: view.rich,
        say: Rc::new(move |text| last.set(text.to_owned())),
    })
}
