//! am3's elements through Aegle's runtime loader: a screen built from
//! markup with states bound to properties and handlers writing them,
//! child elements reaching their parent components, `id` fields typed as
//! am3 handles, and misuse caught before or while building.

mod common;

use aegle_loader::{Data, Program};
use aegle_ui::{Result, Ui};
use am3::{
    Button, DatePicker, Dialog, NavItem, NavigationBar, Slider, Tabs, TextField, TimePicker,
};
use common::ui;

const SCREEN: &str = r#"
Column {
    state page: int = 0
    state name: string = "Ada"
    state confirming: bool = false
    state volume: float = 0.25
    MdTopAppBar {
        title: "Inbox"
        MdAction { icon: "menu"; label: "Menu"; navigation: true }
        MdAction { icon: "search"; label: "Search" }
    }
    MdTabs {
        id: tabs
        MdTab { text: "Mail"; icon: "mail" }
        MdTab { text: "Chat"; selected: page == 1 }
    }
    MdTextField { id: field; label: "Name"; style: outlined; text: name; supporting: "Hi " + name }
    MdSlider { id: slider; value: volume; step: 0.25; ticks: true }
    MdCard {
        style: elevated
        MdText { text: "Card"; typescale: title_medium }
        MdButton { id: save; text: "Save"; style: tonal; on clicked { confirming = true } }
    }
    MdList {
        MdListItem { headline: "Wi-Fi"; supporting: "Connected"; leading: "settings" }
        MdListItem { headline: "Alerts"; clickable: true; on clicked { page = 1 } }
    }
    MdSegmentedButton {
        MdSegment { text: "Day" }
        MdSegment { text: "Week" }
    }
    MdChip { text: "Starred"; kind: filter; selected: true }
    MdCheckbox { text: "Remember"; checked: true }
    MdDatePicker { id: date; selected: "2026-08-17" }
    MdTimePicker { id: time; h24: true; hour: 19; minute: 30 }
    MdCarousel {
        MdCarouselItem { MdText { text: "Lisbon" } }
    }
    MdNavigationBar {
        id: bar
        MdNavItem { id: mail; icon: "mail"; text: "Mail"; badge: 3; on clicked { page = 0 } }
        MdNavItem { id: chat; icon: "chat"; text: "Chat"; badge: -1; on clicked { page = 1 } }
    }
    MdDialog {
        id: dialog
        headline: "Discard draft?"
        open: confirming
        on dismissed { confirming = false }
        MdText { text: "It can't be recovered." }
        MdDialogAction { text: "Cancel"; on clicked { confirming = false } }
    }
}
"#;

fn load(ui: &Ui, source: &str) -> Result<aegle_loader::View> {
    let source = source.to_owned();
    let program =
        Program::from_sources("main.aegle", &am3::elements(), &mut |_| Ok(source.clone()))?;
    program.build(&ui.root())
}

fn typed<T: Clone + 'static>(view: &aegle_loader::View, id: &str) -> T {
    view.handle(id)
        .and_then(|h| h.typed::<T>())
        .expect("an id of that handle type")
}

#[test]
fn a_screen_from_markup() -> Result {
    let ui = ui()?;
    ui.resize(aegle_ui::Size::new(900.0, 2400.0))?;
    let view = load(&ui, SCREEN)?;
    ui.refresh()?;

    let field: TextField = typed(&view, "field");
    assert_eq!(field.text()?, "Ada");
    view.set("name", Data::String("Grace".into()))?;
    assert_eq!(field.text()?, "Grace", "bound text follows the state");
    assert_eq!(typed::<Slider>(&view, "slider").value()?, 0.25);

    // Child elements were created through their parents.
    let bar: NavigationBar = typed(&view, "bar");
    assert_eq!(bar.selected()?, Some(0));
    let tabs: Tabs = typed(&view, "tabs");
    assert_eq!(tabs.selected()?, Some(0));
    typed::<NavItem>(&view, "chat").activate()?;
    ui.dispatch_callbacks()?;
    assert_eq!(view.get("page"), Some(Data::Int(1)));
    assert_eq!(bar.selected()?, Some(1));
    assert_eq!(tabs.selected()?, Some(1), "the tab follows the page");

    // A handler opens the dialog through a bound state; its action closes it.
    let dialog: Dialog = typed(&view, "dialog");
    assert!(!dialog.is_open()?);
    typed::<Button>(&view, "save").activate()?;
    ui.dispatch_callbacks()?;
    assert!(dialog.is_open()?);
    common::key(&ui, aegle_ui::Key::Escape)?;
    assert_eq!(view.get("confirming"), Some(Data::Bool(false)));

    let date: DatePicker = typed(&view, "date");
    assert_eq!(
        date.selected()?.map(|d| d.to_string()),
        Some("2026-08-17".into())
    );
    assert_eq!(typed::<TimePicker>(&view, "time").time()?, (19, 30));
    Ok(())
}

#[test]
fn misuse_is_reported() -> Result {
    let ui = ui()?;
    let check = |source: &str| {
        Program::from_sources("main.aegle", &am3::elements(), &mut |_| {
            Ok(source.to_owned())
        })
        .err()
        .map(|error| error.to_string())
    };
    let outside = check("Column { MdTab { text: \"Lost\" } }").expect("a tab outside tabs");
    assert!(outside.contains("MdTab"), "{outside}");
    let choice = check("Column { MdButton { style: shiny } }").expect("an unknown style");
    assert!(choice.contains("shiny"), "{choice}");
    // Icon names are checked while building.
    assert!(
        load(
            &ui,
            "Column { MdIconButton { icon: \"nope\"; label: \"x\" } }"
        )
        .is_err()
    );
    Ok(())
}
