//! Progress indicators: linear and circular, determinate or indeterminate,
//! flat or wavy (Material 3 Expressive). The active indicator is primary,
//! separated by a 4 dp gap from the secondary-container track, which ends in
//! a stop indicator. Determinate values glide; indeterminate ones and waves
//! animate continuously, requesting frames only while shown.

mod stroke;

use std::{f32::consts::TAU, time::Instant};

use aegle_layout::Style;
use aegle_motion::Easing;
use aegle_ui::{
    Appearance, Container, ControlKind, Point, Result, Size, Theme, VisualState,
    control::{Control, MeasureCx, PaintCx},
    handle,
    scene::{Rect, RoundedRect},
};
use stroke::{Wave, arc, line};

use crate::{Scheme, anim::Value, tokens::motion};

const STROKE: f32 = 4.0;
const GAP: f32 = 4.0;
const AMPLITUDE: f32 = 3.0;

/// The control inside a [`Progress`].
pub struct ProgressControl {
    circular: bool,
    /// The fraction done, or `None` while indeterminate.
    pub(crate) value: Option<f32>,
    pub(crate) wavy: bool,
    shown: Value,
    epoch: Option<Instant>,
}

impl ProgressControl {
    /// The fraction done, or `None` while indeterminate.
    pub fn value(&self) -> Option<f32> {
        self.value
    }
}

/// The indeterminate linear animation: two bars whose heads and tails
/// run across the track on their own curves within one 1.8 s cycle.
fn linear_bars(t: f32) -> [(f32, f32); 2] {
    let curve = |from: f32, to: f32, bezier: (f32, f32, f32, f32)| {
        let ease =
            Easing::cubic_bezier(bezier.0, bezier.1, bezier.2, bezier.3).expect("valid curve");
        ease.sample(((t * 1800.0 - from) / (to - from)).clamp(0.0, 1.0))
    };
    [
        (
            curve(333.0, 1183.0, (0.4, 0.0, 1.0, 1.0)),
            curve(0.0, 750.0, (0.2, 0.0, 0.8, 1.0)),
        ),
        (
            curve(1267.0, 1800.0, (0.1, 0.0, 0.45, 1.0)),
            curve(1000.0, 1567.0, (0.0, 0.0, 0.65, 1.0)),
        ),
    ]
}

/// The indeterminate circular arc at `t` seconds: its start and end angle.
/// The arc grows then shrinks each 1.33 s while the whole turns.
fn circular_arc(t: f32) -> (f32, f32) {
    const PERIOD: f32 = 1.333;
    let ease = Easing::cubic_bezier(0.4, 0.0, 0.2, 1.0).expect("valid curve");
    let cycles = (t / PERIOD).floor();
    let u = t / PERIOD - cycles;
    let head = ease.sample((u * 2.0).min(1.0)) * 0.75;
    let tail = ease.sample((u * 2.0 - 1.0).max(0.0)) * 0.75;
    let base = cycles * 0.75 + t / 4.0;
    ((base + tail) * TAU, (base + head + 0.03) * TAU)
}

impl Control for ProgressControl {
    fn kind(&self) -> &'static ControlKind {
        &PROGRESS
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(match (self.circular, self.wavy) {
            (true, false) => Size::new(40.0, 40.0),
            (true, true) => Size::new(48.0, 48.0),
            (false, false) => Size::new(240.0, STROKE),
            (false, true) => Size::new(240.0, STROKE + 2.0 * AMPLITUDE),
        })
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let a = *cx.appearance;
        let elapsed = now.saturating_duration_since(*self.epoch.get_or_insert(now));
        let t = elapsed.as_secs_f32();
        let mut moving = false;
        let shown = self.value.map(|v| {
            let (v, glide) = self
                .shown
                .at(v, motion::standard::DEFAULT_SPATIAL, now, reduced);
            moving |= glide;
            v.clamp(0.0, 1.0)
        });
        // Waves flatten near the ends of a determinate indicator.
        let calm = shown.map_or(1.0, |f| (f * 10.0).min((1.0 - f) * 20.0).clamp(0.0, 1.0));
        let wave = |length: f32| {
            self.wavy.then(|| Wave {
                amplitude: AMPLITUDE * calm,
                length,
                phase: if reduced { 0.0 } else { -t * length },
            })
        };
        moving |= self.value.is_none() || (self.wavy && !reduced && calm > 0.0);
        let (active, track) = (a.indicator, a.border_color);
        if self.circular {
            let center = Point::new(size.width / 2.0, size.height / 2.0);
            let wavy = if self.wavy { AMPLITUDE / 2.0 } else { 0.0 };
            let radius = (size.width.min(size.height) - STROKE) / 2.0 - wavy;
            let waves = (TAU * radius / 15.0).round().max(1.0);
            let wave = wave(waves).map(|w| Wave {
                amplitude: w.amplitude / 2.0,
                phase: if reduced { 0.0 } else { -t * TAU },
                ..w
            });
            let gap = (GAP + STROKE) / radius;
            match shown {
                Some(f) => {
                    let end = f * TAU;
                    arc(cx.builder, center, radius, (0.0, end), STROKE, wave, active)?;
                    let from = if f > 0.0 { end + gap } else { 0.0 };
                    let to = if f > 0.0 { TAU - gap } else { TAU };
                    arc(cx.builder, center, radius, (from, to), STROKE, None, track)?;
                }
                None => {
                    let (a0, a1) = circular_arc(t);
                    arc(cx.builder, center, radius, (a0, a1), STROKE, wave, active)?;
                }
            }
        } else {
            let y = size.height / 2.0;
            let (x0, x1) = (STROKE / 2.0, size.width - STROKE / 2.0);
            let span = x1 - x0;
            let flip = |x: f32| if cx.rtl { size.width - x } else { x };
            let spans: Vec<(f32, f32)> = match shown {
                Some(f) if f > 0.0 => vec![(x0, x0 + span * f)],
                Some(_) => Vec::new(),
                None => linear_bars((t / 1.8).fract())
                    .into_iter()
                    .filter(|(tail, head)| head > tail)
                    .map(|(tail, head)| (x0 + span * tail, x0 + span * head))
                    .collect(),
            };
            let wave = wave(if self.value.is_some() { 40.0 } else { 20.0 });
            // The track fills what the active spans leave, a gap away.
            let reserve = GAP + STROKE;
            let mut from = x0;
            for &(s0, s1) in &spans {
                if s0 - reserve > from {
                    let (l, r) = (flip(from), flip(s0 - reserve));
                    line(cx.builder, (l.min(r), l.max(r)), y, STROKE, None, track)?;
                }
                let (l, r) = (flip(s0), flip(s1));
                line(cx.builder, (l.min(r), l.max(r)), y, STROKE, wave, active)?;
                from = s1 + reserve;
            }
            if from < x1 {
                let (l, r) = (flip(from), flip(x1));
                line(cx.builder, (l.min(r), l.max(r)), y, STROKE, None, track)?;
                if self.value.is_some() {
                    // The stop indicator at the end of the track.
                    let x = flip(x1);
                    let dot = Rect::new(x - STROKE / 2.0, y - STROKE / 2.0, STROKE, STROKE);
                    cx.builder
                        .fill(RoundedRect::new(dot, STROKE / 2.0)?, active)?;
                }
            }
        }
        if moving {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::Role;
        cx.node.set_role(Role::ProgressIndicator);
        if let Some(value) = self.value {
            cx.node.set_numeric_value(f64::from(value) * 100.0);
            cx.node.set_min_numeric_value(0.0);
            cx.node.set_max_numeric_value(100.0);
        }
    }
}

fn progress_skin(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    Appearance {
        indicator: s.primary,
        border_color: s.secondary_container,
        ..Appearance::base(theme, state)
    }
}

/// A progress indicator.
pub static PROGRESS: ControlKind = ControlKind {
    name: "ProgressIndicator",
    skin: progress_skin,
    accepts: aegle_ui::Accepts::INDICATOR,
    container: false,
};

handle! {
    /// A linear or circular progress indicator.
    pub Progress(ProgressControl): indicator
}

impl Progress {
    /// Appends a linear indicator showing `value` (0 to 1), or an
    /// indeterminate one for `None`.
    pub fn linear(parent: &Container, value: Option<f32>) -> Result<Self> {
        Self::create(parent, false, value)
    }

    /// Appends a circular indicator.
    pub fn circular(parent: &Container, value: Option<f32>) -> Result<Self> {
        Self::create(parent, true, value)
    }

    fn create(parent: &Container, circular: bool, value: Option<f32>) -> Result<Self> {
        let value = value.map(|v| v.clamp(0.0, 1.0));
        let node = parent.add(|_, _| {
            let control = ProgressControl {
                circular,
                value,
                wavy: false,
                shown: Value::default(),
                epoch: None,
            };
            let style = Style {
                flex_shrink: 0.0,
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn Control>, style))
        })?;
        crate::effects(&node)?;
        Ok(Self(node))
    }

    /// Shows `value` (clamped to 0–1), gliding to it, or runs the
    /// indeterminate animation for `None`.
    pub fn set_value(&self, value: Option<f32>) -> Result {
        self.update(|c| c.value = value.map(|v| v.clamp(0.0, 1.0)))
    }

    /// The fraction shown, or `None` while indeterminate.
    pub fn value(&self) -> Result<Option<f32>> {
        self.read(|c| c.value)
    }

    /// Makes the active indicator a travelling wave, the expressive style
    /// for longer waits; the indicator grows to fit the wave.
    pub fn set_wavy(&self, wavy: bool) -> Result {
        self.change(|state, id| {
            let control = state
                .control_as::<ProgressControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?;
            control.wavy = wavy;
            state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?;
            Ok(())
        })
    }
}
