//! The inputs page: filled and outlined text fields, search, the date and
//! time pickers, docked and modal, and a carousel.

use aegle::{ColorSlot, Container, Result};
use am3::{tokens::typescale, *};

use crate::shell::{Say, section};

fn caption(parent: &Container, text: &str) -> Result<Text> {
    Text::new(
        parent,
        typescale::BODY_MEDIUM,
        Role::on_surface_variant,
        text,
    )
}

fn column(row: &Container) -> Result<Container> {
    let column = row.column()?;
    column.set_gap(16.0)?;
    column.set_width(320.0)?;
    Ok(column)
}

pub fn build(page: &Container, say: &Say) -> Result {
    let row = section(page, "文本框")?;
    row.set_align_items(Some(aegle::Align::Start))?;
    let filled = column(&row)?;
    let name = TextField::new(&filled, FieldStyle::Filled, "姓名")?;
    name.set_supporting(Some("可以用输入法输入中文"))?;
    let greeting = caption(&filled, "输入的姓名：")?;
    name.on_change(move |field| greeting.set_text(&format!("输入的姓名：{}", field.text()?)))?;
    let email = TextField::new(&filled, FieldStyle::Filled, "邮箱")?;
    email.set_leading_icon(Some(icons::mail()))?;
    email.set_clearable(true)?;
    let bio = TextField::multiline(&filled, FieldStyle::Filled, "简介")?;
    bio.set_counter(Some(120))?;

    let outlined = column(&row)?;
    let city = TextField::new(&outlined, FieldStyle::Outlined, "城市")?;
    city.set_leading_icon(Some(icons::search()))?;
    city.set_text("上海")?;
    city.set_clearable(true)?;
    let password = TextField::new(&outlined, FieldStyle::Outlined, "密码")?;
    password.set_password(true)?;
    password.set_supporting(Some("至少 8 个字符"))?;
    let wrong = Checkbox::new(&outlined, "标记为错误", false)?;
    wrong.on_change(move |c| password.set_error(c.is_checked()?))?;
    let note = TextField::new(&outlined, FieldStyle::Outlined, "备注")?;
    note.set_placeholder(Some("只读"))?;
    note.set_read_only(true)?;

    let row = section(page, "搜索")?;
    let search = Search::new(&row, "搜索城市")?;
    search.set_width(420.0)?;
    search.set_trailing_icon(Some(icons::mic()))?;
    let list = List::new(search.results())?;
    for city in ["里斯本", "京都", "瓦哈卡", "首尔"] {
        let item = list.clickable_item(city)?;
        item.leading_icon(icons::schedule())?;
        let say = say.clone();
        item.on_click(move |_| say(city))?;
    }

    let row = section(page, "日期选择器")?;
    row.set_align_items(Some(aegle::Align::Start))?;
    let docked = DatePicker::docked(&row)?;
    let say_day = say.clone();
    docked.on_change(move |picker| {
        let day = picker
            .selected()?
            .map(|day| day.to_string())
            .unwrap_or_default();
        say_day(&format!("日期 {day}"))
    })?;
    let side = column(&row)?;
    let open = Button::new(&side, ButtonStyle::Tonal, "打开日期对话框")?;
    open.set_icon(Some(icons::calendar()))?;
    let chosen = caption(&side, "选中的日期：")?;
    let modal = DatePicker::modal(&row)?;
    modal.on_confirm(move |picker| {
        let day = picker
            .selected()?
            .map(|day| day.to_string())
            .unwrap_or_default();
        chosen.set_text(&format!("选中的日期：{day}"))
    })?;
    open.on_click(move |_| modal.show())?;

    let row = section(page, "时间选择器")?;
    row.set_align_items(Some(aegle::Align::Start))?;
    let inline = TimePicker::inline(&row)?;
    inline.set_24_hour(true)?;
    inline.set_time(19, 30)?;
    let side = column(&row)?;
    let open = Button::new(&side, ButtonStyle::Tonal, "打开时间对话框")?;
    open.set_icon(Some(icons::schedule()))?;
    let chosen = caption(&side, "选中的时间：")?;
    let modal = TimePicker::modal(&row)?;
    modal.on_confirm(move |picker| {
        let (hour, minute) = picker.time()?;
        chosen.set_text(&format!("选中的时间：{hour} 时 {minute} 分"))
    })?;
    open.on_click(move |_| modal.show())?;

    let row = section(page, "轮播")?;
    let carousel = Carousel::new(&row, 200.0, 200.0)?;
    carousel.set_width(640.0)?;
    for (title, container, content) in [
        (
            "里斯本",
            Role::primary_container,
            Role::on_primary_container,
        ),
        (
            "京都",
            Role::tertiary_container,
            Role::on_tertiary_container,
        ),
        (
            "瓦哈卡",
            Role::secondary_container,
            Role::on_secondary_container,
        ),
        ("首尔", Role::primary_container, Role::on_primary_container),
        (
            "开普敦",
            Role::tertiary_container,
            Role::on_tertiary_container,
        ),
    ] {
        let item = carousel.item(true)?;
        // Bound to color tokens, the items follow theme changes.
        item.bind_color(ColorSlot::Background, container.token())?;
        Text::new(&item, typescale::TITLE_LARGE, content, title)?;
        let say = say.clone();
        item.on_click(move |_| say(title))?;
    }
    Ok(())
}
