//! Elements of the action and selection components: buttons and their
//! groups, FABs, checkboxes, radio buttons, switches, sliders, chips and
//! the progress and loading indicators.

use aegle_loader::{Elements, element};
use aegle_ui::Result;

use super::{
    adopt, badge, button_size, button_style, choices, fab_color, icon, icon_style, maybe_icon,
    owned,
};
use crate::{
    Badge, Button, ButtonGroup, ButtonShape, Checkbox, Chip, ChipKind, Fab, FabMenu, FabMenuItem,
    FabSize, GroupStyle, IconButton, IconWidth, LoadingIndicator, Progress, Radio, Segment,
    SegmentedButton, Selection, Slider, SliderSize, SplitButton, Switch, icons,
};

choices! {
    chip_kind -> ChipKind {
        "assist" => ChipKind::Assist,
        "filter" => ChipKind::Filter,
        "input" => ChipKind::Input,
        "suggestion" => ChipKind::Suggestion,
    }
    slider_size -> SliderSize {
        "extra_small" => SliderSize::ExtraSmall,
        "small" => SliderSize::Small,
        "medium" => SliderSize::Medium,
        "large" => SliderSize::Large,
        "extra_large" => SliderSize::ExtraLarge,
    }
}

fn group(
    parent: &aegle_ui::Container,
    style: &str,
    size: &str,
    selection: &str,
) -> Result<ButtonGroup> {
    let style = if style == "connected" {
        GroupStyle::Connected
    } else {
        GroupStyle::Standard
    };
    let group = ButtonGroup::new(parent, style, button_size(size))?;
    group.set_selection(match selection {
        "single" => Selection::Single,
        "multiple" => Selection::Multiple,
        _ => Selection::None,
    })?;
    adopt(group.clone(), &group)
}

fn slider(
    parent: &aegle_ui::Container,
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    ticks: bool,
) -> Result<Slider> {
    let slider = Slider::new(parent, min, max, value)?;
    if step > 0.0 {
        slider.set_step(step, ticks)?;
    }
    Ok(slider)
}

element! {
    /// A Material button; `selected` makes it a toggle.
    pub MdButton(Button) {
        create |parent, text: line = "", style: choice(filled, tonal, outlined, elevated, text) = "filled"| Button::new(parent, button_style(style), text);
        set text: line => |button, text| button.set_text(text);
        set style: choice(filled, tonal, outlined, elevated, text) => |button, style| button.set_style(button_style(style));
        set size: choice(extra_small, small, medium, large, extra_large) => |button, size| button.set_size(button_size(size));
        set square: bool => |button, on| button.set_shape(if on { ButtonShape::Square } else { ButtonShape::Round });
        set icon: line => |button, name| button.set_icon(maybe_icon(name)?);
        set selected: bool => |button, on| button.set_selected(Some(on));
        event clicked => |button, run| button.on_click(move |_| run());
        get selected: bool => |button| button.selected().map(|s| s == Some(true));
    }
    /// An icon button, labelled for assistive technology.
    pub MdIconButton(IconButton) {
        create |parent, icon: line, label: line, style: choice(standard, filled, tonal, outlined) = "standard"| IconButton::new(parent, icon_style(style), super::icon(icon)?, label);
        set icon: line => |button, name| button.set_icon(Some(icon(name)?));
        set style: choice(standard, filled, tonal, outlined) => |button, style| button.set_style(icon_style(style));
        set size: choice(extra_small, small, medium, large, extra_large) => |button, size| button.set_size(button_size(size));
        set width: choice(narrow, default, wide) => |button, width| button.set_width(match width {
            "narrow" => IconWidth::Narrow,
            "wide" => IconWidth::Wide,
            _ => IconWidth::Default,
        });
        set selected: bool => |button, on| button.set_selected(Some(on));
        set badge: int(-1, 1000000) => |button, count| badge(button, count, || Badge::new(button));
        event clicked => |button, run| button.on_click(move |_| run());
        get selected: bool => |button| button.selected().map(|s| s == Some(true));
    }
    /// A floating action button, extended with its label shown.
    pub MdFab(Fab) {
        create |parent, icon: line, label: line, extended: bool = false| if extended { Fab::extended(parent, super::icon(icon)?, label) } else { Fab::new(parent, super::icon(icon)?, label) };
        set extended: bool => |fab, on| fab.set_extended(on);
        set size: choice(regular, medium, large) => |fab, size| fab.set_size(match size {
            "medium" => FabSize::Medium,
            "large" => FabSize::Large,
            _ => FabSize::Regular,
        });
        set color: choice(primary_container, secondary_container, tertiary_container, primary, secondary, tertiary) => |fab, color| fab.set_color(fab_color(color));
        event clicked => |fab, run| fab.on_click(move |_| run());
    }
    /// A FAB opening a menu of its `MdFabMenuItem` children.
    pub MdFabMenu(FabMenu) {
        layout flex;
        children only MdFabMenuItem;
        create |parent, icon: line, label: line, color: choice(primary_container, secondary_container, tertiary_container, primary, secondary, tertiary) = "primary_container"| {
            let menu = FabMenu::new(parent, super::icon(icon)?, label, fab_color(color))?;
            adopt(menu.clone(), &menu)
        };
        set open: bool => |menu, open| menu.set_open(open);
        get open: bool => |menu| menu.is_open();
    }
    /// An action of an `MdFabMenu`.
    pub MdFabMenuItem(FabMenuItem) {
        parent MdFabMenu;
        create |parent, icon: line, text: line| owned::<FabMenu>(parent)?.item(super::icon(icon)?, text);
        event clicked => |item, run| item.on_click(move |_| run());
    }
    /// A button group of `MdGroupButton` children.
    pub MdButtonGroup(ButtonGroup) {
        layout flex;
        children only MdGroupButton;
        create |parent, style: choice(standard, connected) = "standard", size: choice(extra_small, small, medium, large, extra_large) = "small", selection: choice(none, single, multiple) = "none"| group(parent, style, size, selection);
    }
    /// A button of an `MdButtonGroup`.
    pub MdGroupButton(Button) {
        parent MdButtonGroup;
        create |parent, text: line, style: choice(filled, tonal, outlined, elevated, text) = "tonal"| owned::<ButtonGroup>(parent)?.button(button_style(style), text);
        set selected: bool => |button, on| button.set_selected(Some(on));
        event clicked => |button, run| button.on_click(move |_| run());
        get selected: bool => |button| button.selected().map(|s| s == Some(true));
    }
    /// A split button: an action and a menu button.
    pub MdSplitButton(SplitButton) {
        create |parent, text: line, style: choice(filled, tonal, outlined, elevated) = "filled"| SplitButton::new(parent, button_style(style), text);
        set size: choice(extra_small, small, medium, large, extra_large) => |split, size| split.set_size(button_size(size));
        set menu_open: bool => |split, open| split.set_menu_open(open);
        event clicked => |split, run| split.action().on_click(move |_| run());
        event menu => |split, run| split.menu().on_click(move |_| run());
    }
    /// A segmented button of `MdSegment` children.
    pub MdSegmentedButton(SegmentedButton) {
        layout flex;
        children only MdSegment;
        create |parent, multiple: bool = false| {
            let segmented = SegmentedButton::new(parent, multiple)?;
            adopt(segmented.clone(), &segmented)
        };
        get selected: int => |segmented| segmented.selected().map(|s| s.first().map_or(-1, |&i| i as i64));
    }
    /// A segment of an `MdSegmentedButton`.
    pub MdSegment(Segment) {
        parent MdSegmentedButton;
        create |parent, text: line, icon: line = ""| owned::<SegmentedButton>(parent)?.segment(text, maybe_icon(icon)?);
        set selected: bool => |segment, on| segment.set_selected(Some(on));
        event clicked => |segment, run| segment.on_click(move |_| run());
        get selected: bool => |segment| segment.selected().map(|s| s == Some(true));
    }
    /// A checkbox with a label.
    pub MdCheckbox(Checkbox) {
        create |parent, text: line = "", checked: bool = false| Checkbox::new(parent, text, checked);
        set text: line => |checkbox, text| checkbox.set_text(text);
        set checked: bool => |checkbox, on| checkbox.set_checked(on);
        set mixed: bool => |checkbox, on| checkbox.set_mixed(on);
        set error: bool => |checkbox, on| checkbox.set_error(on);
        event changed => |checkbox, run| checkbox.on_change(move |_| run());
        get checked: bool => |checkbox| checkbox.is_checked();
    }
    /// A radio button, exclusive among its sibling radio buttons.
    pub MdRadio(Radio) {
        create |parent, text: line = "", checked: bool = false| Radio::new(parent, text, checked);
        set text: line => |radio, text| radio.set_text(text);
        set checked: bool => |radio, on| radio.set_checked(on);
        event changed => |radio, run| radio.on_change(move |_| run());
        get checked: bool => |radio| radio.is_checked();
    }
    /// A switch with a label.
    pub MdSwitch(Switch) {
        create |parent, text: line = "", checked: bool = false| Switch::new(parent, text, checked);
        set text: line => |switch, text| switch.set_text(text);
        set checked: bool => |switch, on| switch.set_checked(on);
        set icons: bool => |switch, on| switch.set_icons(on.then(icons::check), on.then(icons::close));
        event changed => |switch, run| switch.on_change(move |_| run());
        get checked: bool => |switch| switch.is_checked();
    }
    /// A slider; a positive `step` makes it discrete, with stop
    /// indicators when `ticks` is set.
    pub MdSlider(Slider) {
        create |parent, min: float = 0, max: float = 1, value: float = 0, step: float(0) = 0, ticks: bool = false| slider(parent, min, max, value, step, ticks);
        set value: float => |slider, value| slider.set_value(value);
        set size: choice(extra_small, small, medium, large, extra_large) => |slider, size| slider.set_size(slider_size(size));
        set centered: bool => |slider, on| slider.set_centered(on);
        event changed => |slider, run| slider.on_change(move |_| run());
        get value: float => |slider| slider.value();
    }
    /// A chip; filter chips select, input chips can be removed.
    pub MdChip(Chip) {
        create |parent, text: line, kind: choice(assist, filter, input, suggestion) = "assist"| Chip::new(parent, chip_kind(kind), text);
        set text: line => |chip, text| chip.set_text(text);
        set icon: line => |chip, name| chip.set_leading(maybe_icon(name)?);
        set elevated: bool => |chip, on| chip.set_elevated(on);
        set selected: bool => |chip, on| chip.set_selected(Some(on));
        event clicked => |chip, run| chip.on_click(move |_| run());
        event removed => |chip, run| chip.on_remove(move |_| run());
        get selected: bool => |chip| chip.selected().map(|s| s == Some(true));
    }
    /// A linear or circular progress indicator; without a value it is
    /// indeterminate.
    pub MdProgress(Progress) {
        create |parent, circular: bool = false, indeterminate: bool = false| {
            let value = (!indeterminate).then_some(0.0);
            if circular { Progress::circular(parent, value) } else { Progress::linear(parent, value) }
        };
        set value: fraction => |progress, value| progress.set_value(Some(value));
        set wavy: bool => |progress, on| progress.set_wavy(on);
    }
    /// A loading indicator, on a container when `contained`.
    pub MdLoadingIndicator(LoadingIndicator) {
        create |parent, contained: bool = false| LoadingIndicator::new(parent, contained);
    }
}

pub(super) fn add(elements: Elements) -> Elements {
    elements
        .with::<MdButton>()
        .with::<MdIconButton>()
        .with::<MdFab>()
        .with::<MdFabMenu>()
        .with::<MdFabMenuItem>()
        .with::<MdButtonGroup>()
        .with::<MdGroupButton>()
        .with::<MdSplitButton>()
        .with::<MdSegmentedButton>()
        .with::<MdSegment>()
        .with::<MdCheckbox>()
        .with::<MdRadio>()
        .with::<MdSwitch>()
        .with::<MdSlider>()
        .with::<MdChip>()
        .with::<MdProgress>()
        .with::<MdLoadingIndicator>()
}
