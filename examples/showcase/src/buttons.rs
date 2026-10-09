//! The buttons page: every button style, size and shape, toggle buttons,
//! icon buttons, FABs and the FAB menu, button groups and segmented buttons.

use std::{cell::Cell, rc::Rc};

use aegle::{Container, Result};
use am3::*;

use crate::shell::{Say, section};

fn said(say: &Say, text: &'static str) -> impl FnMut(Button) -> Result + 'static {
    let say = say.clone();
    move |_| say(text)
}

fn on_off(selected: Option<bool>) -> &'static str {
    if selected == Some(true) { "开" } else { "关" }
}

pub fn build(page: &Container, say: &Say) -> Result {
    let row = section(page, "常用按钮")?;
    for (style, text) in [
        (ButtonStyle::Elevated, "凸起"),
        (ButtonStyle::Filled, "填充"),
        (ButtonStyle::Tonal, "色调"),
        (ButtonStyle::Outlined, "描边"),
        (ButtonStyle::Text, "文字"),
    ] {
        Button::new(&row, style, text)?.on_click(said(say, text))?;
    }
    let add = Button::new(&row, ButtonStyle::Filled, "添加")?;
    add.set_icon(Some(icons::add()))?;
    add.on_click(said(say, "添加"))?;
    Button::new(&row, ButtonStyle::Filled, "不可用")?.set_enabled(false)?;

    let row = section(page, "尺寸与形状")?;
    for (size, text) in [
        (ButtonSize::ExtraSmall, "超小"),
        (ButtonSize::Small, "小"),
        (ButtonSize::Medium, "中"),
        (ButtonSize::Large, "大"),
        (ButtonSize::ExtraLarge, "超大"),
    ] {
        let button = Button::new(&row, ButtonStyle::Filled, text)?;
        button.set_size(size)?;
        button.on_click(said(say, text))?;
    }
    let square = Button::new(&row, ButtonStyle::Tonal, "方形")?;
    square.set_shape(ButtonShape::Square)?;
    square.on_click(said(say, "方形"))?;

    let row = section(page, "切换按钮")?;
    for (style, text) in [
        (ButtonStyle::Elevated, "凸起"),
        (ButtonStyle::Filled, "填充"),
        (ButtonStyle::Tonal, "色调"),
        (ButtonStyle::Outlined, "描边"),
    ] {
        let toggle = Button::new(&row, style, text)?;
        toggle.set_selected(Some(false))?;
        let say = say.clone();
        toggle.on_click(move |b| say(&format!("{text}切换按钮：{}", on_off(b.selected()?))))?;
    }

    let row = section(page, "图标按钮")?;
    for (style, label) in [
        (IconStyle::Standard, "标准"),
        (IconStyle::Filled, "填充"),
        (IconStyle::Tonal, "色调"),
        (IconStyle::Outlined, "描边"),
    ] {
        let button = IconButton::new(&row, style, icons::favorite(), label)?;
        button.set_selected(Some(false))?;
        let say = say.clone();
        button.on_click(move |b| say(&format!("{label}图标按钮：{}", on_off(b.selected()?))))?;
    }
    for (width, label) in [
        (IconWidth::Narrow, "窄"),
        (IconWidth::Default, "默认"),
        (IconWidth::Wide, "宽"),
    ] {
        let button = IconButton::new(&row, IconStyle::Tonal, icons::edit(), label)?;
        button.set_width(width)?;
    }
    let alerts = IconButton::new(&row, IconStyle::Standard, icons::notifications(), "通知")?;
    Badge::new(&alerts)?.show_count(3)?;

    let row = section(page, "悬浮操作按钮")?;
    let compose = Fab::new(&row, icons::edit(), "撰写")?;
    let composing = say.clone();
    compose.on_click(move |_| composing("撰写"))?;
    let medium = Fab::new(&row, icons::add(), "新建")?;
    medium.set_size(FabSize::Medium)?;
    medium.set_color(FabColor::Tertiary)?;
    let large = Fab::new(&row, icons::photo(), "拍照")?;
    large.set_size(FabSize::Large)?;
    // Clicking the extended FAB folds it to its icon and back.
    let extended = Fab::extended(&row, icons::edit(), "撰写")?;
    let folded = Rc::new(Cell::new(false));
    extended.on_click(move |fab| {
        folded.set(!folded.get());
        fab.set_extended(!folded.get())
    })?;
    let menu = FabMenu::new(&row, icons::add(), "新建", FabColor::PrimaryContainer)?;
    for (icon, text) in [
        (icons::mail(), "消息"),
        (icons::calendar(), "日程"),
        (icons::edit(), "笔记"),
    ] {
        let say = say.clone();
        menu.item(icon, text)?.on_click(move |_| say(text))?;
    }

    let row = section(page, "按钮组")?;
    let group = ButtonGroup::new(&row, GroupStyle::Standard, ButtonSize::Small)?;
    group.button(ButtonStyle::Tonal, "邮件")?;
    group.button(ButtonStyle::Filled, "收件箱")?;
    group.button(ButtonStyle::Tonal, "星标")?;
    let range = ButtonGroup::new(&row, GroupStyle::Connected, ButtonSize::Small)?;
    for text in ["日", "周", "月"] {
        range
            .button(ButtonStyle::Tonal, text)?
            .on_click(said(say, text))?;
    }
    range.set_selection(Selection::Single)?;

    let row = section(page, "分段按钮")?;
    let travel = SegmentedButton::new(&row, false)?;
    for text in ["步行", "骑行", "驾车"] {
        let segment = travel.segment(text, None)?;
        segment.set_selected(Some(text == "步行"))?;
        let say = say.clone();
        segment.on_click(move |_| say(text))?;
    }
    let kinds = SegmentedButton::new(&row, true)?;
    for (icon, text) in [
        (icons::mail(), "邮件"),
        (icons::chat(), "聊天"),
        (icons::notifications(), "通知"),
    ] {
        kinds
            .segment(text, Some(icon))?
            .set_selected(Some(text != "聊天"))?;
    }
    Ok(())
}
