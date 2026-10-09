//! The navigation page: tabs, app bars, toolbars, a navigation bar, a
//! modal navigation drawer, and expanding the window's own rail.

use aegle::{Align, Container, Result};
use am3::{tokens::typescale, *};

use crate::shell::{Say, section};

/// A fixed-width column for components that stretch across their parent.
fn frame(row: &Container, width: f32) -> Result<Container> {
    let column = row.column()?;
    column.set_gap(12.0)?;
    column.set_width(width)?;
    column.set_align_items(Some(Align::Stretch))?;
    Ok(column)
}

pub fn build(page: &Container, say: &Say, rail: &NavigationRail) -> Result {
    let row = section(page, "标签页")?;
    let column = frame(&row, 420.0)?;
    let tabs = Tabs::new(&column, TabStyle::Primary)?;
    for (icon, text) in [
        (icons::send(), "航班"),
        (icons::calendar(), "行程"),
        (icons::explore(), "探索"),
    ] {
        let say = say.clone();
        tabs.tab(Some(icon), text)?.on_click(move |_| say(text))?;
    }
    let tabs = Tabs::new(&column, TabStyle::Secondary)?;
    for text in ["概览", "规格", "评价"] {
        tabs.tab(None, text)?;
    }

    let row = section(page, "顶部应用栏")?;
    let column = frame(&row, 420.0)?;
    let small = TopAppBar::new(&column, AppBarStyle::Small, "收件箱")?;
    small.navigation(icons::menu(), "菜单")?;
    small.action(icons::search(), "搜索")?;
    small.action(icons::more_vert(), "更多")?;
    let centered = TopAppBar::new(&column, AppBarStyle::CenterAligned, "居中标题")?;
    centered.navigation(icons::arrow_back(), "返回")?;
    centered.action(icons::person(), "账户")?;
    let medium = TopAppBar::new(&column, AppBarStyle::Medium, "中号标题")?;
    medium.subtitle("副标题")?;
    medium.navigation(icons::arrow_back(), "返回")?;
    medium.action(icons::bookmark(), "书签")?;

    let row = section(page, "底部应用栏与工具栏")?;
    let column = frame(&row, 420.0)?;
    let bottom = BottomAppBar::new(&column)?;
    for (icon, label) in [
        (icons::search(), "搜索"),
        (icons::delete(), "删除"),
        (icons::share(), "分享"),
    ] {
        let say = say.clone();
        bottom.action(icon, label)?.on_click(move |_| say(label))?;
    }
    let docked = Toolbar::docked(&column, ToolbarColor::Standard)?;
    for (icon, label) in [
        (icons::arrow_back(), "后退"),
        (icons::arrow_forward(), "前进"),
        (icons::refresh(), "刷新"),
        (icons::share(), "分享"),
    ] {
        docked.icon_button(icon, label)?;
    }
    let floating = Toolbar::floating(&row, ToolbarColor::Vibrant, false)?;
    for (icon, label) in [
        (icons::edit(), "编辑"),
        (icons::photo(), "图片"),
        (icons::mic(), "语音"),
    ] {
        floating.icon_button(icon, label)?;
    }

    let row = section(page, "导航栏")?;
    let column = frame(&row, 420.0)?;
    let bar = NavigationBar::new(&column)?;
    bar.item(icons::home(), "首页")?;
    bar.item(icons::mail(), "邮件")?.badge()?.show_count(3)?;
    bar.item(icons::notifications(), "通知")?
        .badge()?
        .show_dot()?;
    bar.item(icons::settings(), "设置")?;

    let row = section(page, "导航抽屉与栏杆")?;
    let drawer = NavigationDrawer::modal(&row)?;
    drawer.headline("邮件")?;
    for (icon, text) in [
        (icons::mail(), "收件箱"),
        (icons::send(), "已发送"),
        (icons::star(), "星标"),
    ] {
        let (say, closing) = (say.clone(), drawer.clone());
        drawer.item(icon, text)?.on_click(move |_| {
            closing.close()?;
            say(text)
        })?;
    }
    let open = Button::new(&row, ButtonStyle::Tonal, "打开导航抽屉")?;
    open.set_icon(Some(icons::menu()))?;
    open.on_click(move |_| drawer.show())?;
    let expand = Switch::new(&row, "展开左侧导航栏杆", false)?;
    let rail = rail.clone();
    expand.on_change(move |s| rail.set_expanded(s.is_checked()?))?;
    Text::new(
        &row,
        typescale::BODY_MEDIUM,
        Role::on_surface_variant,
        "展开后栏杆显示为行内标签。",
    )?;
    Ok(())
}
