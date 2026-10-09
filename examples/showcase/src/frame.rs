//! The Rust version's window: the header with the theme controls, the
//! navigation rail and the scrolling page area.

use std::rc::Rc;

use aegle::{Align, App, Container, Result, Widgets, WindowOptions};
use am3::{Divider, NavigationRail, Role, SegmentedButton, Switch, Text, icons, tokens::typescale};

use crate::shell::{PAGES, SEEDS, Say, Shell};

pub fn build(app: &App) -> Result<Shell> {
    let window = app.window_with_options(
        "am3 控件展示",
        WindowOptions {
            width: 1180,
            height: 820,
            ..Default::default()
        },
    )?;

    let header = window.row()?;
    header.set_padding(16.0)?;
    header.set_gap(16.0)?;
    header.set_align_items(Some(Align::Center))?;
    let titles = header.column()?;
    titles.set_grow(1.0)?;
    Text::new(
        &titles,
        typescale::TITLE_LARGE,
        Role::on_surface,
        "am3 控件展示",
    )?;
    let status = Text::new(
        &titles,
        typescale::BODY_SMALL,
        Role::on_surface_variant,
        "最近操作：无",
    )?;
    let seed = SegmentedButton::new(&header, false)?;
    let seeds = SEEDS
        .iter()
        .map(|(name, _)| seed.segment(name, None))
        .collect::<Result<Vec<_>>>()?;
    let dark = Switch::new(&header, "深色", false)?;
    let contrast = Switch::new(&header, "高对比", false)?;
    let reduced = Switch::new(&header, "减少动态效果", false)?;
    Divider::new(&window)?;

    let body = window.row()?;
    body.set_grow(1.0)?;
    let rail = NavigationRail::new(&body)?;
    let scroll = body.scroll_view()?;
    scroll.set_grow(1.0)?;
    scroll.set_padding(24.0)?;

    let say: Say = Rc::new(move |text| status.set_text(&format!("最近操作：{text}")));
    let page = |index: usize| -> Result<Container> {
        let page = scroll.column()?;
        page.set_gap(28.0)?;
        page.set_visible(index == 0)?;
        Ok(page)
    };
    let pages = [page(0)?, page(1)?, page(2)?, page(3)?, page(4)?];
    crate::buttons::build(&pages[0], &say)?;
    crate::selection::build(&pages[1], &say)?;
    crate::inputs::build(&pages[2], &say)?;
    let anchors = crate::surfaces::build(&pages[3], &say)?;
    crate::navigation::build(&pages[4], &say, &rail)?;

    for (index, (icon, label)) in PAGES.into_iter().enumerate() {
        let item = rail.item(icons::named(icon).expect("a bundled icon"), label)?;
        let pages = pages.clone();
        item.on_click(move |_| {
            for (other, page) in pages.iter().enumerate() {
                page.set_visible(other == index)?;
            }
            Ok(())
        })?;
    }

    Ok(Shell {
        window,
        seeds,
        dark,
        contrast,
        reduced,
        menu: anchors.menu,
        split: anchors.split,
        tooltip: anchors.tooltip,
        rich: anchors.rich,
        say,
    })
}
