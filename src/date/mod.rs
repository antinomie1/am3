//! Date pickers: a modal picker in a dialog headed by the chosen date, and
//! a docked picker on a raised surface, both with a month row (the month
//! and year, previous and next buttons) over the month grid. Single dates
//! or ranges.

mod calendar;

use aegle_ui::{
    Appearance, Container, ControlKind, Node, NodeId, Result, State, Theme, VisualState,
};

pub use calendar::CalendarControl;

use crate::{
    Dialog, Divider, IconButton, IconStyle, Role, Text, icons,
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
    tokens::typescale,
};

/// A day of the proleptic Gregorian calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// The year.
    pub year: i32,
    /// The month, 1 to 12.
    pub month: u8,
    /// The day of the month, from 1.
    pub day: u8,
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

impl Date {
    /// The date, if the month and day exist.
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if month >= 1 && month <= 12 && day >= 1 && day <= Self::days_in_month(year, month) {
            Some(Self { year, month, day })
        } else {
            None
        }
    }

    /// A date known to exist.
    pub(crate) const fn ymd(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Today in UTC, from the system clock.
    pub fn today() -> Self {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Self::from_days((seconds / 86_400) as i64)
    }

    /// Days since 1970-01-01.
    pub fn days(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let (m, d) = (i64::from(self.month), i64::from(self.day));
        let y = i64::from(self.year) - i64::from(m <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    /// The day of the week, 0 for Sunday.
    pub fn weekday(self) -> u8 {
        (self.days() + 4).rem_euclid(7) as u8
    }

    /// The number of days in a month.
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    /// The same day `months` later, or the month's last day if it is shorter.
    pub fn add_months(self, months: i32) -> Self {
        let index = self.year * 12 + i32::from(self.month) - 1 + months;
        let (year, month) = (index.div_euclid(12), (index.rem_euclid(12) + 1) as u8);
        Self::ymd(year, month, self.day.min(Self::days_in_month(year, month)))
    }

    /// As in a picker's headline: "Mon, Aug 17".
    pub fn headline(self) -> String {
        let month = &MONTHS[usize::from(self.month) - 1][..3];
        format!(
            "{}, {month} {}",
            WEEKDAYS[usize::from(self.weekday())],
            self.day
        )
    }
}

impl std::fmt::Display for Date {
    /// ISO 8601: 2026-08-17.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn calendar_skin(theme: &Theme, state: VisualState) -> Appearance {
    Appearance::base(theme, state)
}

/// A date picker's month grid.
pub static CALENDAR: ControlKind = ControlKind {
    name: "Calendar",
    skin: calendar_skin,
    accepts: aegle_ui::Accepts::TEXT.with(aegle_ui::Accepts::INTERACTIVE),
    container: false,
};

kinds! { container
    /// A docked date picker's surface.
    DOCKED_DATE_PICKER = "DockedDatePicker", docked => |s, _on| {
        Paint::new(s.surface_container_high, s.on_surface)
    };
}

/// Names the shown month on the calendar's title.
fn retitle(state: &mut State, id: NodeId) -> Result {
    let control = state.control_as::<CalendarControl>(id).expect("a calendar");
    let ((year, month), title) = (control.month, control.title);
    if let Some(title) = title {
        state.set_text(title, &format!("{} {year}", MONTHS[usize::from(month) - 1]))?;
    }
    Ok(())
}

/// A modal or docked date picker.
#[derive(Clone)]
pub struct DatePicker {
    calendar: Node,
    dialog: Option<Dialog>,
    /// A modal picker's headline, showing the selection.
    headline: Option<Text>,
}

impl DatePicker {
    /// Creates a hidden modal picker in the theme of `owner`; its headline
    /// shows the chosen date, and Cancel and OK close it.
    pub fn modal(owner: &Node) -> Result<Self> {
        let dialog = Dialog::new(owner, None, "Select date")?;
        dialog
            .surface()
            .set_padding(crate::edges(12.0, 16.0, 12.0, 12.0))?;
        dialog.surface().set_gap(12.0)?;
        // A label-large title over a headline-large selection, as in
        // Material's header, inset 24 dp.
        let title = dialog.headline();
        title.set_type(typescale::LABEL_LARGE)?;
        title.set_color(Role::on_surface_variant)?;
        title.set_margin(crate::edges(12.0, 0.0, 0.0, 0.0))?;
        let headline = Text::new(
            &dialog,
            typescale::HEADLINE_LARGE,
            Role::on_surface_variant,
            "",
        )?;
        headline.set_margin(crate::edges(12.0, 0.0, 0.0, 12.0))?;
        Divider::new(&dialog)?;
        let calendar = Self::grid(&dialog)?;
        let picker = Self {
            calendar,
            dialog: Some(dialog.clone()),
            headline: Some(headline),
        };
        picker.on_change(|picker| picker.show_selection())?;
        let closing = dialog.clone();
        dialog
            .action("Cancel")?
            .on_click(move |_| closing.close())?;
        Ok(picker)
    }

    /// Names the selection on a modal picker's headline.
    fn show_selection(&self) -> Result {
        let Some(headline) = &self.headline else {
            return Ok(());
        };
        let short = |day: Date| format!("{} {}", &MONTHS[usize::from(day.month) - 1][..3], day.day);
        let text = match (self.range()?, self.selected()?) {
            (Some((start, end)), _) => format!("{} – {}", short(start), short(end)),
            (None, Some(day)) if self.update(|c| c.range)? => format!("{} – End date", short(day)),
            (None, Some(day)) => day.headline(),
            (None, None) => "Selected date".to_owned(),
        };
        headline.set_text(&text)
    }

    /// Appends a docked picker to `parent`, on a raised 16 dp rounded
    /// surface.
    pub fn docked(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&DOCKED_DATE_PICKER, Part::Pane, 16.0, 3.0);
        let surface = surface::add(parent, control, 12.0)?;
        let calendar = Self::grid(&surface)?;
        Ok(Self {
            calendar,
            dialog: None,
            headline: None,
        })
    }

    /// Appends the month row and the grid.
    fn grid(parent: &Container) -> Result<Node> {
        let row = parent.row()?;
        row.set_align_items(Some(aegle_ui::Align::Center))?;
        row.set_padding(crate::edges(12.0, 0.0, 0.0, 0.0))?;
        let title = Text::new(&row, typescale::LABEL_LARGE, Role::on_surface_variant, "")?;
        row.row()?.set_grow(1.0)?;
        let previous = IconButton::new(
            &row,
            IconStyle::Standard,
            icons::chevron_left(),
            "Previous month",
        )?;
        let next = IconButton::new(
            &row,
            IconStyle::Standard,
            icons::chevron_right(),
            "Next month",
        )?;
        let calendar = parent.add(|state, theme| {
            let mut style = aegle_ui::text_style(theme);
            typescale::BODY_LARGE.apply(&mut style);
            let mut control = CalendarControl::new(&state.fonts, &style, Date::today())?;
            control.title = Some(title.id);
            Ok((
                Box::new(control) as Box<dyn aegle_ui::Control>,
                aegle_layout::Style {
                    flex_shrink: 0.0,
                    ..Default::default()
                },
            ))
        })?;
        calendar.change(retitle)?;
        let picker = Self {
            calendar: calendar.clone(),
            dialog: None,
            headline: None,
        };
        let back = picker.clone();
        previous.on_click(move |_| back.step_month(-1))?;
        let forward = picker.clone();
        next.on_click(move |_| forward.step_month(1))?;
        Ok(calendar)
    }

    fn update<R>(&self, update: impl FnOnce(&mut CalendarControl) -> R) -> Result<R> {
        self.calendar.change(|state, id| {
            let control = state
                .control_as::<CalendarControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?;
            let value = update(control);
            state
                .tree
                .mark_dirty(id, aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS)?;
            retitle(state, id)?;
            Ok(value)
        })
    }

    /// Shows the month `months` away.
    pub fn step_month(&self, months: i32) -> Result {
        self.update(|c| {
            let first = Date::ymd(c.month.0, c.month.1, 1).add_months(months);
            c.month = (first.year, first.month);
            c.cursor = first;
        })
    }

    /// Shows the month of `day`.
    pub fn show_month(&self, day: Date) -> Result {
        self.update(|c| {
            c.month = (day.year, day.month);
            c.cursor = day;
        })
    }

    /// Selects `day`, or clears the selection, and shows its month.
    pub fn set_selected(&self, day: Option<Date>) -> Result {
        self.update(|c| {
            c.selected = day;
            c.end = None;
            if let Some(day) = day {
                c.month = (day.year, day.month);
                c.cursor = day;
            }
        })
    }

    /// The selected date, or a range's start.
    pub fn selected(&self) -> Result<Option<Date>> {
        self.update(|c| c.selected)
    }

    /// Picks a range: a start, then an end on or after it.
    pub fn set_range(&self, range: bool) -> Result {
        self.update(|c| c.range = range)
    }

    /// The selected range, once both ends are chosen.
    pub fn range(&self) -> Result<Option<(Date, Date)>> {
        self.update(|c| c.selected.zip(c.end))
    }

    /// Selects a range.
    pub fn set_selected_range(&self, start: Date, end: Date) -> Result {
        self.update(|c| {
            c.range = true;
            c.selected = Some(start.min(end));
            c.end = Some(start.max(end));
            c.month = (start.year, start.month);
        })
    }

    /// Outlines another day as today.
    pub fn set_today(&self, today: Date) -> Result {
        self.update(|c| c.today = today)
    }

    /// Adds a handler run after each choice of a day.
    pub fn on_change(&self, mut callback: impl FnMut(DatePicker) -> Result + 'static) -> Result {
        let picker = self.clone();
        self.calendar
            .change(|state, id| state.on_action(id, move |_| callback(picker.clone())))
    }

    /// A modal picker's headline, naming the selection.
    pub fn headline(&self) -> Option<&Text> {
        self.headline.as_ref()
    }

    /// The month grid, which takes focus and arrow keys.
    pub fn calendar(&self) -> &Node {
        &self.calendar
    }

    /// The modal picker's dialog.
    pub fn dialog(&self) -> Option<&Dialog> {
        self.dialog.as_ref()
    }

    /// Adds an OK action to a modal picker that runs `callback` and closes
    /// it.
    pub fn on_confirm(&self, mut callback: impl FnMut(DatePicker) -> Result + 'static) -> Result {
        let Some(dialog) = &self.dialog else {
            return Ok(());
        };
        let (picker, closing) = (self.clone(), dialog.clone());
        dialog.action("OK")?.on_click(move |_| {
            callback(picker.clone())?;
            closing.close()
        })
    }

    /// Shows a modal picker.
    pub fn show(&self) -> Result {
        match &self.dialog {
            Some(dialog) => {
                self.show_selection()?;
                dialog.show()
            }
            None => Ok(()),
        }
    }
}
