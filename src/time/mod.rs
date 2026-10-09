//! Time pickers: hour and minute fields (96 × 80 dp, display large) with
//! an AM/PM selector over a clock dial, in a dialog or inline. Choosing an
//! hour on the dial moves on to the minutes; the fields switch what the
//! dial sets.

mod dial;

use aegle_ui::{
    Align, Appearance, Color, Container, ControlKind, Node, NodeId, Result, State, Theme,
    VisualState, handle,
};

pub use dial::DialControl;
use dial::Mode;

use crate::{
    Dialog, Role, Text,
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
    tokens::{Corner, typescale},
};

/// An hour or minute field, or half of the AM/PM selector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TimeLook {
    Field,
    Period,
}

impl TimeLook {
    pub fn kind(self) -> &'static ControlKind {
        match self {
            Self::Field => &TIME_FIELD,
            Self::Period => &TIME_PERIOD,
        }
    }

    pub fn spec(self) -> Spec {
        let (height, width, text, radius) = match self {
            Self::Field => (80.0, 96.0, typescale::DISPLAY_LARGE, 8.0),
            Self::Period => (40.0, 52.0, typescale::TITLE_MEDIUM, 7.0),
        };
        Spec {
            height,
            width: Some(width),
            min_width: width,
            padding: 0.0,
            icon_padding: 0.0,
            icon: 0.0,
            gap: 0.0,
            text,
            corners: [Corner::Dp(radius); 3],
            inner: [Corner::Dp(radius); 3],
            hover_inner: false,
            outline: 0.0,
            elevation: [0.0, 0.0],
        }
    }
}

kinds! {
    /// An hour or minute field of a time picker.
    TIME_FIELD = "TimeSelector", time_field => |s, on| if on {
        Paint::new(s.primary_container, s.on_primary_container)
    } else {
        Paint::new(s.surface_container_highest, s.on_surface)
    };
    /// Half of a time picker's AM/PM selector.
    TIME_PERIOD = "PeriodSelector", time_period => |s, on| if on {
        Paint::new(s.tertiary_container, s.on_tertiary_container)
    } else {
        Paint::new(Color::TRANSPARENT, s.on_surface_variant)
    };
}

kinds! { container
    /// The surface of an inline time picker, and the outline of its AM/PM
    /// selector.
    TIME_PICKER = "TimePicker", time_picker => |s, _on| {
        Paint::new(s.surface_container_high, s.on_surface)
    };
    /// The outlined box of the AM/PM selector.
    PERIOD_BOX = "PeriodSelectorBox", period_box => |s, _on| {
        Paint::new(Color::TRANSPARENT, s.on_surface).outlined(s.outline)
    };
}

fn dial_skin(theme: &Theme, state: VisualState) -> Appearance {
    Appearance::base(theme, state)
}

/// A time picker's clock dial.
pub static DIAL: ControlKind = ControlKind {
    name: "TimeDial",
    skin: dial_skin,
    accepts: aegle_ui::Accepts::TEXT.with(aegle_ui::Accepts::INTERACTIVE),
    container: false,
};

handle! {
    /// A field of a time picker's header.
    pub TimeField(PressableControl): text, interactive, pressed, indicator
}

pressable_methods!(TimeField);

/// The nodes a picker keeps in step, stored on the dial for its hooks.
#[derive(Default)]
struct Parts {
    pickers: Vec<(NodeId, [NodeId; 4])>,
}

/// Shows the dial's values and mode in the fields of its picker.
fn sync(state: &mut State, dial: NodeId) -> Result {
    let Some(&(_, [hour, minute, am, pm])) = state
        .ext_ref::<Parts>()
        .and_then(|p| p.pickers.iter().find(|(d, _)| *d == dial))
    else {
        return Ok(());
    };
    let control = state.control_as::<DialControl>(dial).expect("a dial");
    let (h, m, mode, h24) = (control.hour, control.minute, control.mode, control.h24);
    let shown = if h24 { h } else { (h + 11) % 12 + 1 };
    state.set_text(hour, &format!("{shown:02}"))?;
    state.set_text(minute, &format!("{m:02}"))?;
    for (id, on) in [
        (hour, mode == Mode::Hour),
        (minute, mode == Mode::Minute),
        (am, h < 12),
        (pm, h >= 12),
    ] {
        state
            .control_as::<PressableControl>(id)
            .expect("a field")
            .selected = Some(on);
        state.dirty_visual_state(id)?;
    }
    Ok(())
}

/// The dial moved on to the minutes.
fn show_mode(state: &mut State, dial: NodeId) -> Result {
    sync(state, dial)
}

/// A time picker.
#[derive(Clone)]
pub struct TimePicker {
    dial: Node,
    dialog: Option<Dialog>,
}

impl TimePicker {
    /// Creates a hidden time picker dialog in the theme of `owner`, with
    /// Cancel closing it.
    pub fn modal(owner: &Node) -> Result<Self> {
        let dialog = Dialog::new(owner, None, "")?;
        dialog.headline().set_visible(false)?;
        let mut picker = Self::build(&dialog)?;
        let closing = dialog.clone();
        dialog
            .action("Cancel")?
            .on_click(move |_| closing.close())?;
        picker.dialog = Some(dialog);
        Ok(picker)
    }

    /// Appends an inline time picker on a 28 dp rounded surface.
    pub fn inline(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&TIME_PICKER, Part::Pane, 28.0, 0.0);
        let surface = surface::add(parent, control, 24.0)?;
        Self::build(&surface)
    }

    fn build(parent: &Container) -> Result<Self> {
        parent.set_align_items(Some(Align::Center))?;
        let title = Text::new(
            parent,
            typescale::LABEL_MEDIUM,
            Role::on_surface_variant,
            "Select time",
        )?;
        title.set_align_self(Some(Align::Start))?;
        let header = parent.row()?;
        header.set_align_items(Some(Align::Center))?;
        header.set_gap(12.0)?;
        let fields = header.row()?;
        fields.set_align_items(Some(Align::Center))?;
        fields.set_gap(0.0)?;
        let field = |text: &str| -> Result<TimeField> {
            pressable::add(&fields, |fonts, theme| {
                let mut control =
                    PressableControl::new(fonts, theme, Look::Time(TimeLook::Field), text)?;
                control.selected = Some(false);
                control.exclusive = true;
                Ok(control)
            })
            .map(TimeField)
        };
        let hour = field("12")?;
        let colon = Text::new(&fields, typescale::DISPLAY_LARGE, Role::on_surface, ":")?;
        colon.set_margin(crate::edges(4.0, 0.0, 4.0, 0.0))?;
        let minute = field("00")?;
        let period = surface::add(
            &header,
            SurfaceControl::new(&PERIOD_BOX, Part::Pane, 8.0, 0.0),
            0.0,
        )?;
        period.set_padding(1.0)?;
        let half = |text: &str| -> Result<TimeField> {
            pressable::add(&period, |fonts, theme| {
                let mut control =
                    PressableControl::new(fonts, theme, Look::Time(TimeLook::Period), text)?;
                control.selected = Some(false);
                Ok(control)
            })
            .map(TimeField)
        };
        let am = half("AM")?;
        let pm = half("PM")?;
        let dial = parent.add(|state, theme| {
            let mut style = aegle_ui::text_style(theme);
            typescale::BODY_LARGE.apply(&mut style);
            let control = DialControl::new(&state.fonts, &style)?;
            Ok((
                Box::new(control) as Box<dyn aegle_ui::Control>,
                aegle_layout::Style {
                    flex_shrink: 0.0,
                    margin: aegle_layout::Edges {
                        top: aegle_layout::LengthPercentageAuto::length(24.0),
                        ..aegle_layout::Edges::auto()
                    },
                    ..Default::default()
                },
            ))
        })?;
        dial.change(|state, id| {
            state
                .ext::<Parts>()
                .pickers
                .push((id, [hour.id, minute.id, am.id, pm.id]));
            sync(state, id)
        })?;
        let picker = Self { dial, dialog: None };
        for (button, mode) in [(&hour, Mode::Hour), (&minute, Mode::Minute)] {
            let p = picker.clone();
            button.on_click(move |_| p.update(|c| c.mode = mode))?;
        }
        for (button, pm) in [(&am, false), (&pm, true)] {
            let p = picker.clone();
            button
                .on_click(move |_| p.update(|c| c.hour = c.hour % 12 + if pm { 12 } else { 0 }))?;
        }
        let p = picker.clone();
        picker.on_change(move |_| p.update(|_| ()))?;
        Ok(picker)
    }

    fn update<R>(&self, update: impl FnOnce(&mut DialControl) -> R) -> Result<R> {
        self.dial.change(|state, id| {
            let control = state
                .control_as::<DialControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?;
            let value = update(control);
            state
                .tree
                .mark_dirty(id, aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS)?;
            sync(state, id)?;
            Ok(value)
        })
    }

    /// Shows `hour` (0–23) and `minute`, clamped into range.
    pub fn set_time(&self, hour: u8, minute: u8) -> Result {
        self.update(|c| {
            c.hour = hour.min(23);
            c.minute = minute.min(59);
        })
    }

    /// The hour (0–23) and minute shown.
    pub fn time(&self) -> Result<(u8, u8)> {
        self.update(|c| (c.hour, c.minute))
    }

    /// Uses 24-hour time: the dial gets an inner ring and AM/PM hides.
    pub fn set_24_hour(&self, h24: bool) -> Result {
        self.update(|c| c.h24 = h24)?;
        self.dial.change(|state, id| {
            let am = state
                .ext_ref::<Parts>()
                .and_then(|p| p.pickers.iter().find(|(d, _)| *d == id))
                .map(|(_, parts)| parts[2])
                .expect("a picker's dial");
            let period = state.tree.parent(am)?.expect("the AM/PM box");
            state.set_visible(period, !h24)
        })
    }

    /// Adds a handler run after each change of the time.
    pub fn on_change(&self, mut callback: impl FnMut(TimePicker) -> Result + 'static) -> Result {
        let picker = self.clone();
        self.dial
            .change(|state, id| state.on_action(id, move |_| callback(picker.clone())))
    }

    /// Adds an OK action to a time picker dialog that runs `callback` and
    /// closes it.
    pub fn on_confirm(&self, mut callback: impl FnMut(TimePicker) -> Result + 'static) -> Result {
        let Some(dialog) = &self.dialog else {
            return Ok(());
        };
        let (picker, closing) = (self.clone(), dialog.clone());
        dialog.action("OK")?.on_click(move |_| {
            callback(picker.clone())?;
            closing.close()
        })
    }

    /// The clock dial, which takes focus and arrow keys.
    pub fn dial(&self) -> &Node {
        &self.dial
    }

    /// The dialog of a modal picker.
    pub fn dialog(&self) -> Option<&Dialog> {
        self.dialog.as_ref()
    }

    /// Shows a modal picker.
    pub fn show(&self) -> Result {
        match &self.dialog {
            Some(dialog) => dialog.show(),
            None => Ok(()),
        }
    }
}

/// Hook: forgets removed dials.
pub(crate) fn removed(state: &mut State, id: NodeId) {
    if state.ext_ref::<Parts>().is_some() {
        state.ext::<Parts>().pickers.retain(|(dial, _)| *dial != id);
    }
}
