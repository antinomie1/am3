//! The month grid of a date picker: weekday initials over six weeks of
//! 40 dp day circles in 48 dp cells. Today is outlined, the selection is a
//! primary circle and a range is joined by a secondary-container band.
//! Pointer clicks choose a day; arrows, Page Up and Page Down move a
//! keyboard cursor through days and months, and Enter or Space chooses it.

use std::cell::RefCell;

use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    ControlKind, Key, Point, Rect, Result, Size,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, RoundedRect},
};

use super::Date;
use crate::{
    Scheme,
    color::alpha,
    skin,
    tokens::{state, typescale},
};

const CELL: f32 = 48.0;
const DAY: f32 = 40.0;
const HEADER: f32 = 40.0;

/// The control inside a date picker's grid.
pub struct CalendarControl {
    /// The year and month shown.
    pub(crate) month: (i32, u8),
    pub(crate) selected: Option<Date>,
    /// The end of a selected range; ranges pick a start, then an end.
    pub(crate) end: Option<Date>,
    pub(crate) range: bool,
    pub(crate) today: Date,
    /// The day the keyboard is on.
    pub(crate) cursor: Date,
    /// The text node naming the shown month.
    pub(crate) title: Option<aegle_ui::NodeId>,
    hovered: Option<Date>,
    weekdays: Vec<Paragraph>,
    days: Vec<Paragraph>,
    button: aegle_controls::Button,
}

impl CalendarControl {
    pub(crate) fn new(
        fonts: &RefCell<TextSystem>,
        style: &TextStyle<'_>,
        today: Date,
    ) -> Result<Self> {
        let mut fonts = fonts.borrow_mut();
        let weekdays = ["S", "M", "T", "W", "T", "F", "S"]
            .iter()
            .map(|d| fonts.paragraph(*d, style))
            .collect::<std::result::Result<_, _>>()?;
        let days = (1..=31)
            .map(|d| fonts.paragraph(d.to_string(), style))
            .collect::<std::result::Result<_, _>>()?;
        Ok(Self {
            month: (today.year, today.month),
            selected: None,
            end: None,
            range: false,
            today,
            cursor: today,
            title: None,
            hovered: None,
            weekdays,
            days,
            button: aegle_controls::Button::new(),
        })
    }

    /// The first day of the grid's first row.
    fn first(&self) -> i64 {
        let first = Date::ymd(self.month.0, self.month.1, 1).days();
        first - Date::from_days(first).weekday() as i64
    }

    /// The day under a point, if it belongs to the shown month.
    fn day_at(&self, at: Point) -> Option<Date> {
        if at.y < HEADER || at.x < 0.0 || at.x >= 7.0 * CELL {
            return None;
        }
        let (col, row) = ((at.x / CELL) as i64, ((at.y - HEADER) / CELL) as i64);
        (row < 6)
            .then(|| Date::from_days(self.first() + row * 7 + col))
            .filter(|d| (d.year, d.month) == self.month)
    }

    /// The cell of a day of the shown month.
    fn cell(&self, day: Date, rtl: bool) -> Rect {
        let index = day.days() - self.first();
        let (col, row) = (index % 7, index / 7);
        let col = if rtl { 6 - col } else { col };
        Rect::new(col as f32 * CELL, HEADER + row as f32 * CELL, CELL, CELL)
    }

    /// Chooses `day`: the selection, or a range's start or end.
    fn choose(&mut self, day: Date) {
        self.cursor = day;
        match (self.range, self.selected, self.end) {
            (true, Some(start), None) if day >= start => self.end = Some(day),
            _ => {
                self.selected = Some(day);
                self.end = None;
            }
        }
    }

    fn move_cursor(&mut self, days: i64) {
        self.cursor = Date::from_days(self.cursor.days() + days);
    }
}

/// Paints `text` centered in `cell`.
fn centered(
    builder: &mut aegle_ui::scene::SceneBuilder,
    text: &Paragraph,
    cell: Rect,
    color: aegle_ui::Color,
) -> Result {
    let size = text.size();
    let at = Point::new(
        cell.origin.x + (cell.size.width - size.width) / 2.0,
        cell.origin.y + (cell.size.height - size.height) / 2.0,
    );
    builder.push_transform(Affine::translation(at.x, at.y)?)?;
    text.paint_with_color(builder, color)?;
    builder.pop()?;
    Ok(())
}

impl Control for CalendarControl {
    fn kind(&self) -> &'static ControlKind {
        &super::CALENDAR
    }
    fn interactive(&self) -> bool {
        true
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn text_role(&self, style: &mut TextStyle<'_>) {
        typescale::BODY_LARGE.apply(style);
    }
    fn restyle(&mut self, fonts: &mut TextSystem, style: &TextStyle<'_>) -> Result {
        for text in self.weekdays.iter_mut().chain(&mut self.days) {
            fonts.restyle(text, style)?;
        }
        Ok(())
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            pressed: self.button.is_pressed(),
            hovered: Some(self.hovered.is_some()),
            ..Default::default()
        }
    }
    fn set_enabled(&mut self, _: &mut TextSystem, enabled: bool) -> Outcome {
        self.button.set_enabled(enabled)
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let mut outcome = Outcome::default();
        match input {
            Input::Pointer(p) => {
                let at = if cx.rtl {
                    Point::new(7.0 * CELL - p.position.x, p.position.y)
                } else {
                    p.position
                };
                let day = p.inside.then(|| self.day_at(at)).flatten();
                if day != self.hovered {
                    self.hovered = day;
                    outcome.repaint = true;
                }
                if let (aegle_ui::PointerKind::Up, Some(day)) = (p.kind, day) {
                    self.choose(day);
                    outcome.action = Some(Action::Change);
                    outcome.repaint = true;
                    outcome.semantics = true;
                }
                if matches!(p.kind, aegle_ui::PointerKind::Down { .. }) {
                    outcome.focus = true;
                }
                outcome.handled = true;
            }
            Input::Key(key) if key.pressed => {
                let step = |forward: bool| if forward != cx.rtl { 1 } else { -1 };
                match key.key {
                    Key::Left => self.move_cursor(-step(true)),
                    Key::Right => self.move_cursor(step(true)),
                    Key::Up => self.move_cursor(-7),
                    Key::Down => self.move_cursor(7),
                    Key::PageUp => self.cursor = self.cursor.add_months(-1),
                    Key::PageDown => self.cursor = self.cursor.add_months(1),
                    Key::Enter | Key::Character(' ') => {
                        self.choose(self.cursor);
                        outcome.action = Some(Action::Change);
                        outcome.semantics = true;
                    }
                    _ => return Ok(outcome),
                }
                let month = (self.cursor.year, self.cursor.month);
                if month != self.month {
                    self.month = month;
                    cx.deferred.push(Box::new(super::retitle));
                }
                outcome.handled = true;
                outcome.repaint = true;
            }
            Input::Key(_) => {}
            other => outcome = self.button.handle(other),
        }
        Ok(outcome)
    }
    fn hover(
        &mut self,
        cx: &mut InputCx<'_>,
        _: aegle_ui::PointerId,
        input: Input<'_>,
    ) -> Result<Outcome> {
        self.handle(cx, input)
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(Size::new(7.0 * CELL, HEADER + 6.0 * CELL))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let s = Scheme::of(cx.theme);
        let enabled = cx.visual.enabled;
        let ink = |c| {
            if enabled {
                c
            } else {
                alpha(c, state::DISABLED_CONTENT)
            }
        };
        for (i, text) in self.weekdays.iter().enumerate() {
            let col = if cx.rtl { 6 - i } else { i };
            let cell = Rect::new(col as f32 * CELL, 0.0, CELL, HEADER);
            centered(cx.builder, text, cell, ink(s.on_surface))?;
        }
        let circle = |cell: Rect| {
            let inset = (CELL - DAY) / 2.0;
            Rect::new(cell.origin.x + inset, cell.origin.y + inset, DAY, DAY)
        };
        // The band joining a range's ends, drawn under their circles.
        if let (Some(start), Some(end)) = (self.selected, self.end) {
            let first = Date::ymd(self.month.0, self.month.1, 1).days();
            let last = first + Date::days_in_month(self.month.0, self.month.1) as i64 - 1;
            for d in start.days().max(first)..=end.days().min(last) {
                let cell = self.cell(Date::from_days(d), cx.rtl);
                // The ends' cells are banded only on their inner half.
                let (mut x, mut width) = (cell.origin.x, CELL);
                if d == start.days() {
                    width /= 2.0;
                    x += if cx.rtl { 0.0 } else { width };
                }
                if d == end.days() {
                    width /= 2.0;
                    x += if cx.rtl { width } else { 0.0 };
                }
                let band = Rect::new(x, cell.origin.y + 4.0, width, DAY);
                cx.builder
                    .fill(RoundedRect::new(band, 0.0)?, ink(s.secondary_container))?;
            }
        }
        let days = Date::days_in_month(self.month.0, self.month.1);
        for n in 1..=days {
            let day = Date::ymd(self.month.0, self.month.1, n);
            let cell = self.cell(day, cx.rtl);
            let disc = RoundedRect::new(circle(cell), DAY / 2.0)?;
            let chosen = Some(day) == self.selected || Some(day) == self.end;
            let banded =
                matches!((self.selected, self.end), (Some(a), Some(b)) if day > a && day < b);
            let mut color = if banded {
                s.on_secondary_container
            } else {
                s.on_surface
            };
            if chosen {
                cx.builder.fill(disc, ink(s.primary))?;
                color = s.on_primary;
            } else if day == self.today {
                cx.builder.stroke(disc, ink(s.primary), 1.0)?;
                color = s.primary;
            }
            if enabled && self.hovered == Some(day) {
                let layer = if chosen { s.on_primary } else { s.on_surface };
                cx.builder.fill(disc, alpha(layer, state::HOVER))?;
            }
            if cx.visual.focused && day == self.cursor {
                skin::focus_ring(cx.builder, circle(cell), DAY / 2.0, s.secondary)?;
            }
            centered(cx.builder, &self.days[n as usize - 1], cell, ink(color))?;
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::{Action as A, Role};
        cx.node.set_role(Role::Grid);
        if let Some(day) = self.selected {
            cx.node.set_value(day.to_string());
        }
        if cx.enabled {
            cx.node.add_action(A::Focus);
        }
    }
}
