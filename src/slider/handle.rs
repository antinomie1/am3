//! The slider's kind, skin and handle.

use aegle_controls::Range;
use aegle_layout::Style;
use aegle_ui::{Appearance, Container, Control, ControlKind, Result, Theme, VisualState, handle};

use super::{SliderControl, SliderSize};
use crate::{Scheme, color::alpha, tokens::state};

fn slider_skin(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    let base = Appearance::base(theme, state);
    Appearance {
        indicator: if state.enabled {
            s.primary
        } else {
            alpha(s.on_surface, state::DISABLED_CONTENT)
        },
        border_color: if state.enabled {
            s.secondary_container
        } else {
            alpha(s.on_surface, state::DISABLED_CONTAINER)
        },
        focus_color: s.secondary,
        focus_width: 0.0,
        ..base
    }
}

/// A slider.
pub static SLIDER: ControlKind = ControlKind {
    name: "Slider",
    skin: slider_skin,
    accepts: aegle_ui::Accepts::INTERACTIVE
        .with(aegle_ui::Accepts::PRESSED)
        .with(aegle_ui::Accepts::INDICATOR),
    container: false,
};

handle! {
    /// A slider selecting a value, or a range of values, from a range.
    pub Slider(SliderControl): interactive, pressed, indicator
}

impl Slider {
    /// Appends a continuous slider over `min..=max`; it needs `min < max`.
    pub fn new(parent: &Container, min: f64, max: f64, value: f64) -> Result<Self> {
        let range = Range::new(min, max, value.clamp(min, max), 0.0)?;
        let node = parent.add(|state, theme| {
            let control = SliderControl::new(&state.fonts, theme, range)?;
            let style = Style {
                flex_shrink: 0.0,
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn Control>, style))
        })?;
        crate::effects(&node)?;
        Ok(Self(node))
    }

    /// The value; a range slider's end value.
    pub fn value(&self) -> Result<f64> {
        self.read(|c| c.value())
    }
    /// A range slider's start value.
    pub fn start(&self) -> Result<Option<f64>> {
        self.read(|c| c.start())
    }
    /// Sets the value (a range slider's end) without reporting a change.
    pub fn set_value(&self, value: f64) -> Result {
        self.update(|c| c.behaviors()[0].range_mut().set_value(value).map(drop))??;
        Ok(())
    }
    /// Sets a range slider's start without reporting a change.
    pub fn set_start(&self, value: f64) -> Result {
        self.update(|c| c.behaviors()[1].range_mut().set_value(value).map(drop))??;
        Ok(())
    }
    /// Makes it a range slider with a second handle at the minimum, or a
    /// single-value slider again.
    pub fn set_range(&self, range: bool) -> Result {
        self.update(|c| c.set_range(range))
    }
    /// Snaps to multiples of `step` from the minimum; 0 is continuous.
    /// With `ticks`, a stop indicator marks every step.
    pub fn set_step(&self, step: f64, ticks: bool) -> Result {
        self.update(|c| {
            c.ticks = ticks;
            for b in c.behaviors() {
                b.range_mut().set_step(step)?;
            }
            Ok::<_, aegle_controls::RangeError>(())
        })??;
        Ok(())
    }
    /// Fills from the middle, for values around a center.
    pub fn set_centered(&self, centered: bool) -> Result {
        self.update(|c| c.centered = centered)
    }
    /// Changes the track size and relayouts.
    pub fn set_size(&self, size: SliderSize) -> Result {
        self.change(|state, id| {
            let control = state
                .control_as::<SliderControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?;
            control.size = size;
            state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?;
            Ok(())
        })
    }
    /// Adds a handler run after each change by the user.
    pub fn on_change(&self, mut callback: impl FnMut(Slider) -> Result + 'static) -> Result {
        self.change(|state, id| state.on_action(id, move |node| callback(Slider(node))))
    }
}
