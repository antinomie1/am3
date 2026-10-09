//! The selection page: checkboxes, radio buttons, switches, sliders and
//! chips.

use aegle::{Container, Result};
use am3::*;

use crate::shell::{Say, section};

pub fn build(page: &Container, say: &Say) -> Result {
    let row = section(page, "复选框")?;
    for (text, checked) in [("已读", true), ("星标", false)] {
        let checkbox = Checkbox::new(&row, text, checked)?;
        let say = say.clone();
        checkbox.on_change(move |c| {
            say(&format!(
                "{text}：{}",
                if c.is_checked()? { "勾选" } else { "取消" }
            ))
        })?;
    }
    Checkbox::new(&row, "部分", true)?.set_mixed(true)?;
    Checkbox::new(&row, "必填", false)?.set_error(true)?;
    Checkbox::new(&row, "不可用", true)?.set_enabled(false)?;

    let row = section(page, "单选按钮")?;
    for (text, checked) in [("小杯", false), ("中杯", true), ("大杯", false)] {
        let radio = Radio::new(&row, text, checked)?;
        let say = say.clone();
        radio.on_change(move |_| say(text))?;
    }

    let row = section(page, "开关")?;
    for (text, on) in [("Wi-Fi", true), ("蓝牙", false)] {
        let switch = Switch::new(&row, text, on)?;
        let say = say.clone();
        switch.on_change(move |s| {
            say(&format!(
                "{text}：{}",
                if s.is_checked()? { "开" } else { "关" }
            ))
        })?;
    }
    let plane = Switch::new(&row, "飞行模式", false)?;
    plane.set_icons(Some(icons::check()), Some(icons::close()))?;
    Switch::new(&row, "不可用", true)?.set_enabled(false)?;

    let row = section(page, "滑块")?;
    let column = row.column()?;
    column.set_gap(16.0)?;
    column.set_width(420.0)?;
    let label = Text::new(
        &column,
        tokens::typescale::BODY_MEDIUM,
        Role::on_surface_variant,
        "音量：40",
    )?;
    let volume = Slider::new(&column, 0.0, 100.0, 40.0)?;
    volume.on_change(move |s| label.set_text(&format!("音量：{}", s.value()? as i64)))?;
    Slider::new(&column, 0.0, 10.0, 6.0)?.set_step(1.0, true)?;
    let centered = Slider::new(&column, -50.0, 50.0, 20.0)?;
    centered.set_centered(true)?;
    let large = Slider::new(&column, 0.0, 1.0, 0.5)?;
    large.set_size(SliderSize::Large)?;

    let row = section(page, "纸片")?;
    let calendar = Chip::new(&row, ChipKind::Assist, "加入日程")?;
    calendar.set_leading(Some(icons::calendar()))?;
    let directions = Chip::new(&row, ChipKind::Assist, "路线")?;
    directions.set_leading(Some(icons::send()))?;
    directions.set_elevated(true)?;
    for (text, selected) in [("素食", true), ("无麸质", false)] {
        Chip::new(&row, ChipKind::Filter, text)?.set_selected(Some(selected))?;
    }
    for name in ["Ada", "Grace"] {
        let chip = Chip::new(&row, ChipKind::Input, name)?;
        chip.set_leading(Some(icons::person()))?;
        let say = say.clone();
        chip.on_remove(move |chip| {
            chip.set_visible(false)?;
            say(&format!("移除 {name}"))
        })?;
    }
    let suggestion = Chip::new(&row, ChipKind::Suggestion, "听起来不错")?;
    let say = say.clone();
    suggestion.on_click(move |_| say("听起来不错"))
}
