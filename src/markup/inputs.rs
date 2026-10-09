//! Elements of the input components: text fields, search, the date and
//! time pickers and carousels.

use aegle_loader::{Elements, element};
use aegle_ui::{Container, Result, UiError};

use super::{adopt, maybe_icon, owned};
use crate::{Card, Carousel, Date, DatePicker, FieldStyle, Search, TextField, TimePicker};

fn text_field(parent: &Container, label: &str, style: &str, multiline: bool) -> Result<TextField> {
    let style = if style == "outlined" {
        FieldStyle::Outlined
    } else {
        FieldStyle::Filled
    };
    if multiline {
        TextField::multiline(parent, style, label)
    } else {
        TextField::new(parent, style, label)
    }
}

/// A date from markup: ISO 8601, or empty for none.
fn date(text: &str) -> Result<Option<Date>> {
    if text.is_empty() {
        return Ok(None);
    }
    Date::parse(text)
        .map(Some)
        .ok_or_else(|| UiError::InvalidValue.into())
}

fn date_picker(parent: &Container, modal: bool, range: bool) -> Result<DatePicker> {
    let picker = if modal {
        DatePicker::modal(parent)?
    } else {
        DatePicker::docked(parent)?
    };
    picker.set_range(range)?;
    Ok(picker)
}

fn time_picker(parent: &Container, modal: bool, h24: bool) -> Result<TimePicker> {
    let picker = if modal {
        TimePicker::modal(parent)?
    } else {
        TimePicker::inline(parent)?
    };
    picker.set_24_hour(h24)?;
    Ok(picker)
}

element! {
    /// A filled or outlined text field, multiline when asked; `counter`
    /// above 0 shows a character count against it.
    pub MdTextField(TextField) {
        create |parent, label: line = "", style: choice(filled, outlined) = "filled", multiline: bool = false| text_field(parent, label, style, multiline);
        set text: string => |field, text| field.set_text(text);
        set placeholder: line => |field, text| field.set_placeholder((!text.is_empty()).then_some(text));
        set supporting: line => |field, text| field.set_supporting((!text.is_empty()).then_some(text));
        set error: bool => |field, on| field.set_error(on);
        set leading_icon: line => |field, name| field.set_leading_icon(maybe_icon(name)?);
        set trailing_icon: line => |field, name| field.set_trailing_icon(maybe_icon(name)?);
        set clearable: bool => |field, on| field.set_clearable(on);
        set counter: int(0, 100000) => |field, max| field.set_counter((max > 0).then_some(max as usize));
        set password: bool => |field, on| field.set_password(on);
        set read_only: bool => |field, on| field.set_read_only(on);
        event changed => |field, run| field.on_change(move |_| run());
        get text: string => |field| field.text();
    }
    /// A search bar opening a view of its children, such as an `MdList`,
    /// while it holds text.
    pub MdSearch(Search) {
        layout flex;
        create |parent, placeholder: line = ""| Search::new(parent, placeholder);
        set trailing_icon: line => |search, name| search.set_trailing_icon(maybe_icon(name)?);
        event changed => |search, run| search.on_change(move |_| run());
        get text: string => |search| search.text();
        children => |search, _| search.results().clone();
    }
    /// A docked or modal date picker of one date or a range, as ISO 8601
    /// text; a modal one shows while `open` and reports `confirmed`.
    pub MdDatePicker(DatePicker) {
        create |parent, modal: bool = false, range: bool = false| date_picker(parent, modal, range);
        set selected: line => |picker, text| picker.set_selected(date(text)?);
        set open: bool => |picker, open| match (open, picker.dialog()) {
            (true, _) => picker.show(),
            (false, Some(dialog)) => dialog.close(),
            (false, None) => Ok(()),
        };
        event changed => |picker, run| picker.on_change(move |_| run());
        event confirmed => |picker, run| picker.on_confirm(move |_| run());
        get selected: string => |picker| picker.selected().map(|d| d.map_or_else(String::new, |d| d.to_string()));
        get end: string => |picker| picker.range().map(|r| r.map_or_else(String::new, |(_, end)| end.to_string()));
    }
    /// An inline or modal time picker; a modal one shows while `open`.
    pub MdTimePicker(TimePicker) {
        create |parent, modal: bool = false, h24: bool = false| time_picker(parent, modal, h24);
        set hour: int(0, 23) => |picker, hour| picker.set_time(hour as u8, picker.time()?.1);
        set minute: int(0, 59) => |picker, minute| picker.set_time(picker.time()?.0, minute as u8);
        set h24: bool => |picker, on| picker.set_24_hour(on);
        set open: bool => |picker, open| match (open, picker.dialog()) {
            (true, _) => picker.show(),
            (false, Some(dialog)) => dialog.close(),
            (false, None) => Ok(()),
        };
        event changed => |picker, run| picker.on_change(move |_| run());
        event confirmed => |picker, run| picker.on_confirm(move |_| run());
        get hour: int => |picker| picker.time().map(|(h, _)| i64::from(h));
        get minute: int => |picker| picker.time().map(|(_, m)| i64::from(m));
    }
    /// A carousel of `MdCarouselItem` children `item_width` dp wide.
    pub MdCarousel(Carousel) {
        layout flex;
        children only MdCarouselItem;
        create |parent, item_width: float(0) = 180, height: float(0) = 200| {
            let carousel = Carousel::new(parent, item_width as f32, height as f32)?;
            adopt(carousel.clone(), &carousel)
        };
    }
    /// An item of an `MdCarousel`, holding its children.
    pub MdCarouselItem(Card) {
        layout flex;
        parent MdCarousel;
        create |parent, clickable: bool = false| owned::<Carousel>(parent)?.item(clickable);
        event clicked => |item, run| item.on_click(move |_| run());
    }
}

pub(super) fn add(elements: Elements) -> Elements {
    elements
        .with::<MdTextField>()
        .with::<MdSearch>()
        .with::<MdDatePicker>()
        .with::<MdTimePicker>()
        .with::<MdCarousel>()
        .with::<MdCarouselItem>()
}
