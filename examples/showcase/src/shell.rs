//! What the Rust and markup versions share: the controls the host wires to
//! the window (seed, dark, high contrast, reduced motion), and the menus and
//! tooltips that have no markup elements, attached to anchors either
//! version built.

use std::{cell::Cell, rc::Rc};

use aegle::{Color, Container, Preferences, Result, Window, Wrap};
use am3::{
    Button, IconButton, Menu, Mode, RichTooltip, Role, Segment, SplitButton, Switch, Text,
    set_tooltip, tokens::typescale,
};

/// Seed colors to choose from, in the order of the seed segments.
pub const SEEDS: [(&str, Color); 4] = [
    ("紫", Color::rgb(0x67, 0x50, 0xA4)),
    ("青", Color::rgb(0x00, 0x6A, 0x6A)),
    ("琥珀", Color::rgb(0x8B, 0x50, 0x00)),
    ("玫红", Color::rgb(0xB0, 0x30, 0x6A)),
];

/// Pages of the navigation rail: icon name and label.
pub const PAGES: [(&str, &str); 5] = [
    ("star", "按钮"),
    ("check", "选择"),
    ("keyboard", "输入"),
    ("bookmark", "容器与浮层"),
    ("explore", "导航"),
];

/// Reports an action in the window's status line.
pub type Say = Rc<dyn Fn(&str) -> Result>;

/// Handles the host wires, from either version.
pub struct Shell {
    pub window: Window,
    pub seeds: Vec<Segment>,
    pub dark: Switch,
    pub contrast: Switch,
    pub reduced: Switch,
    /// Opens a menu.
    pub menu: Button,
    /// Its menu part opens a menu.
    pub split: SplitButton,
    /// Shows a plain tooltip on hover.
    pub tooltip: IconButton,
    /// Shows a rich tooltip when clicked.
    pub rich: IconButton,
    pub say: Say,
}

#[derive(Clone, Copy)]
struct Look {
    seed: usize,
    dark: bool,
    contrast: bool,
}

impl Look {
    fn mode(self) -> Mode {
        match (self.dark, self.contrast) {
            (false, false) => Mode::Light,
            (true, false) => Mode::Dark,
            (false, true) => Mode::LightHighContrast,
            (true, true) => Mode::DarkHighContrast,
        }
    }
}

/// A titled section whose content wraps in a row.
pub fn section(page: &Container, title: &str) -> Result<Container> {
    let column = page.column()?;
    column.set_gap(12.0)?;
    Text::new(&column, typescale::TITLE_MEDIUM, Role::on_surface, title)?;
    let row = column.row()?;
    row.set_gap(12.0)?;
    row.set_wrap(Wrap::Wrap)?;
    row.set_align_items(Some(aegle::Align::Center))?;
    Ok(row)
}

/// Starts the switches at the system's preferences and applies every
/// change to the window, then adds the menus and tooltips.
pub fn wire(shell: Shell, system: Preferences) -> Result {
    let look = Rc::new(Cell::new(Look {
        seed: 0,
        dark: system.dark.unwrap_or(false),
        contrast: system.high_contrast.unwrap_or(false),
    }));
    shell.seeds[0].set_selected(Some(true))?;
    shell.dark.set_checked(look.get().dark)?;
    shell.contrast.set_checked(look.get().contrast)?;
    shell
        .reduced
        .set_checked(system.reduced_motion.unwrap_or(false))?;

    let window = shell.window.clone();
    let apply = Rc::new(move |change: &dyn Fn(&mut Look)| {
        let mut next = look.get();
        change(&mut next);
        look.set(next);
        window.set_theme(am3::theme(SEEDS[next.seed].1, next.mode()))
    });
    apply(&|_| {})?;
    for (index, segment) in shell.seeds.iter().enumerate() {
        let apply = apply.clone();
        segment.on_click(move |_| apply(&|look| look.seed = index))?;
    }
    let to_dark = apply.clone();
    shell.dark.on_change(move |switch| {
        let on = switch.is_checked()?;
        to_dark(&|look| look.dark = on)
    })?;
    shell.contrast.on_change(move |switch| {
        let on = switch.is_checked()?;
        apply(&|look| look.contrast = on)
    })?;
    let window = shell.window.clone();
    shell
        .reduced
        .on_change(move |switch| window.set_reduced_motion(switch.is_checked()?))?;
    extras(&shell)
}

/// The menus and tooltips, which have no markup elements.
fn extras(shell: &Shell) -> Result {
    let menu = Menu::new(&shell.menu)?;
    for (text, shortcut) in [("剪切", "Ctrl+X"), ("复制", "Ctrl+C"), ("粘贴", "Ctrl+V")] {
        let item = menu.item(text)?;
        item.set_shortcut(Some(shortcut))?;
        let say = shell.say.clone();
        item.on_click(move |_| say(text))?;
    }
    menu.separator()?;
    menu.check_item("显示标尺", true)?;
    let transform = menu.submenu("转换")?;
    for text in ["大写", "小写"] {
        let say = shell.say.clone();
        transform.item(text)?.on_click(move |_| say(text))?;
    }
    let shown = menu.clone();
    shell.menu.on_click(move |_| shown.show())?;

    let save = Menu::new(shell.split.menu())?;
    for text in ["保存为草稿", "保存并发送"] {
        let (say, split) = (shell.say.clone(), shell.split.clone());
        save.item(text)?.on_click(move |_| {
            split.set_menu_open(false)?;
            say(text)
        })?;
    }
    // The menu part is a toggle: open the menu with it, and keep them in
    // step when the menu was closed by a click elsewhere.
    let split = shell.split.clone();
    shell.split.menu().on_click(move |_| {
        let open = !save.is_shown()?;
        split.set_menu_open(open)?;
        if open { save.show() } else { save.hide() }
    })?;
    let say = shell.say.clone();
    shell.split.action().on_click(move |_| say("保存"))?;

    set_tooltip(&shell.tooltip, Some("加入书签"))?;
    let tip = RichTooltip::new(
        &shell.rich,
        Some("富工具提示"),
        "富工具提示说明一项功能，可以带一个操作。",
    )?;
    let hiding = tip.clone();
    tip.action("知道了")?.on_click(move |_| hiding.hide())?;
    shell.rich.on_click(move |_| tip.show())
}
