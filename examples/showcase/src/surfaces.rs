//! The containers and overlays page: cards, a list, a dialog, sheets, a
//! snackbar, menus, tooltips, badges and progress indicators.

use aegle::{Color, Container, Result};
use am3::{tokens::typescale, *};

use crate::shell::{Say, section};

/// Anchors for the menus and tooltips `shell::wire` adds.
pub struct Anchors {
    pub menu: Button,
    pub split: SplitButton,
    pub tooltip: IconButton,
    pub rich: IconButton,
}

fn card(row: &Container, style: CardStyle, title: &str, say: &Say) -> Result {
    let card = Card::new(row, style)?;
    card.set_width(220.0)?;
    Text::new(&card, typescale::TITLE_MEDIUM, Role::on_surface, title)?;
    Text::new(
        &card,
        typescale::BODY_MEDIUM,
        Role::on_surface_variant,
        "卡片把相关的内容与操作放在一起。",
    )?;
    let say = say.clone();
    let title = title.to_owned();
    Button::new(&card, ButtonStyle::Filled, "操作")?.on_click(move |_| say(&title))
}

pub fn build(page: &Container, say: &Say) -> Result<Anchors> {
    let row = section(page, "卡片")?;
    card(&row, CardStyle::Elevated, "凸起卡片", say)?;
    card(&row, CardStyle::Filled, "填充卡片", say)?;
    card(&row, CardStyle::Outlined, "描边卡片", say)?;
    let clickable = Card::clickable(&row, CardStyle::Outlined)?;
    clickable.set_width(220.0)?;
    Text::new(
        &clickable,
        typescale::TITLE_MEDIUM,
        Role::on_surface,
        "可点击卡片",
    )?;
    Text::new(
        &clickable,
        typescale::BODY_MEDIUM,
        Role::on_surface_variant,
        "整张卡片是一个按钮。",
    )?;
    let clicked = say.clone();
    clickable.on_click(move |_| clicked("可点击卡片"))?;

    let row = section(page, "列表")?;
    let list = List::new(&row)?;
    list.set_width(420.0)?;
    // The pane's surface shows through.
    list.set_background(Color::TRANSPARENT)?;
    for (headline, supporting, icon, trailing) in [
        ("Ada Lovelace", "周五一起吃午饭？", "person", "15 分钟"),
        ("邮件", "3 封未读", "mail", "刚刚"),
        ("设置", "声音、振动", "settings", ""),
    ] {
        let item = list.clickable_item(headline)?;
        item.supporting(supporting)?;
        item.leading_icon(icons::named(icon).expect("a bundled icon"))?;
        if trailing.is_empty() {
            item.trailing_icon(icons::chevron_right())?;
        } else {
            item.trailing_text(trailing)?;
        }
        let say = say.clone();
        item.on_click(move |_| say(headline))?;
    }

    let row = section(page, "对话框、表与提示条")?;
    let dialog = Dialog::new(&row, Some(icons::delete()), "删除草稿？")?;
    dialog.supporting("草稿会从这台设备上删除，无法恢复。")?;
    let closing = dialog.clone();
    dialog.action("取消")?.on_click(move |_| closing.close())?;
    let (closing, deleted) = (dialog.clone(), say.clone());
    dialog.action("删除")?.on_click(move |_| {
        closing.close()?;
        deleted("删除草稿")
    })?;
    let open = Button::new(&row, ButtonStyle::Tonal, "对话框")?;
    open.on_click(move |_| dialog.show())?;

    let bottom = Sheet::bottom(&row)?;
    let actions = List::new(&bottom)?;
    actions.set_background(Color::TRANSPARENT)?;
    for (icon, text) in [
        (icons::share(), "分享"),
        (icons::download(), "下载"),
        (icons::delete(), "删除"),
    ] {
        let item = actions.clickable_item(text)?;
        item.leading_icon(icon)?;
        let (say, closing) = (say.clone(), bottom.clone());
        item.on_click(move |_| {
            closing.close()?;
            say(text)
        })?;
    }
    let open = Button::new(&row, ButtonStyle::Tonal, "底部表")?;
    open.on_click(move |_| bottom.show())?;

    let side = Sheet::side(&row, "筛选")?;
    side.set_gap(12.0)?;
    for (text, on) in [("未读", true), ("星标", false), ("有附件", true)] {
        Checkbox::new(&side, text, on)?;
    }
    let open = Button::new(&row, ButtonStyle::Tonal, "侧边表")?;
    open.on_click(move |_| side.show())?;

    let bar = Snackbar::new(&row, "邮件已归档")?;
    let (undone, closing) = (say.clone(), bar.clone());
    bar.action("撤销")?.on_click(move |_| {
        closing.close()?;
        undone("撤销归档")
    })?;
    bar.closable()?;
    let open = Button::new(&row, ButtonStyle::Tonal, "提示条")?;
    open.on_click(move |_| bar.show(Some(snackbar::SHORT)))?;

    let row = section(page, "菜单与工具提示")?;
    let menu = Button::new(&row, ButtonStyle::Outlined, "编辑")?;
    menu.set_icon(Some(icons::edit()))?;
    let split = SplitButton::new(&row, ButtonStyle::Filled, "保存")?;
    let tooltip = IconButton::new(&row, IconStyle::Standard, icons::bookmark(), "书签")?;
    let rich = IconButton::new(&row, IconStyle::Standard, icons::info(), "说明")?;

    let row = section(page, "徽章")?;
    let dot = IconButton::new(&row, IconStyle::Standard, icons::mail(), "邮件")?;
    Badge::new(&dot)?.show_dot()?;
    let count = IconButton::new(&row, IconStyle::Standard, icons::notifications(), "通知")?;
    Badge::new(&count)?.show_count(3)?;
    let many = IconButton::new(&row, IconStyle::Tonal, icons::chat(), "聊天")?;
    Badge::new(&many)?.show_count(1200)?;

    let row = section(page, "进度指示器")?;
    let column = row.column()?;
    column.set_gap(16.0)?;
    column.set_width(420.0)?;
    let linear = Progress::linear(&column, Some(0.4))?;
    let wavy = Progress::linear(&column, Some(0.4))?;
    wavy.set_wavy(true)?;
    let progress = Slider::new(&column, 0.0, 1.0, 0.4)?;
    progress.on_change(move |s| {
        let value = s.value()? as f32;
        linear.set_value(Some(value))?;
        wavy.set_value(Some(value))
    })?;
    Progress::linear(&column, None)?;
    Progress::circular(&row, None)?;
    LoadingIndicator::new(&row, false)?;
    LoadingIndicator::new(&row, true)?;

    Ok(Anchors {
        menu,
        split,
        tooltip,
        rich,
    })
}
