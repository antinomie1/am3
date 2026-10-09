//! The Rust version's window: the full-height navigation rail, and beside
//! it the header with the theme controls over the scrolling page pane.

use std::rc::Rc;

use aegle::{Align, App, Color, ColorSlot, Container, Insets, Length, Result, WindowOptions};
use am3::{NavigationRail, Role, SegmentedButton, Switch, Text, icons, tokens::typescale};

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
    // The navigation sits on a darker surface than the page, as on
    // m3.material.io; the page is a lighter rounded pane without an outline.
    // Bound to tokens, both follow theme changes.
    window.bind_color(ColorSlot::Background, Role::surface_container.token())?;
    window.set_padding(0.0)?;

    let body = window.row()?;
    body.set_grow(1.0)?;
    // Without it the row grows to the page's height and never scrolls.
    body.set_min_height(0.0)?;
    let rail = NavigationRail::new(&body)?;
    // The rail shows the window's darker surface instead of its own.
    rail.set_background(Color::TRANSPARENT)?;
    let main = body.column()?;
    main.set_grow(1.0)?;
    main.set_min_height(0.0)?;
    main.set_padding(Insets {
        left: Length::Px(0.0),
        top: Length::Px(0.0),
        right: Length::Px(16.0),
        bottom: Length::Px(16.0),
    })?;

    let header = main.row()?;
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

    let scroll = am3::scroll_view(&main)?;
    // Aegle's scroll views don't shrink by default; this one must fit the
    // window to scroll.
    scroll.set_grow(1.0)?;
    scroll.set_shrink(1.0)?;
    scroll.set_min_height(0.0)?;
    scroll.set_padding(24.0)?;
    scroll.bind_color(ColorSlot::Background, Role::surface.token())?;
    scroll.set_radius(24.0)?;

    let say: Say = Rc::new(move |text| status.set_text(&format!("最近操作：{text}")));
    let page = |index: usize| -> Result<Container> {
        let page = scroll.column()?;
        page.set_gap(28.0)?;
        // A page keeps its content height and scrolls instead of shrinking
        // its sections into the viewport.
        page.set_shrink(0.0)?;
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
