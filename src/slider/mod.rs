//! Sliders, the Material 3 Expressive way: a thick track split by a 4 dp
//! bar handle with 6 dp gaps, stop indicators, and a value bubble while
//! dragging. A range slider has a second handle. Each handle reuses Aegle's
//! slider behavior (pointer capture, arrow, page, Home and End keys).

use aegle_controls::{Range, Slider as Behavior};
use aegle_text::{Paragraph, TextSystem};
use aegle_ui::{
    Color, ControlKind, Point, Result, Size, Theme,
    control::{Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, Rect, RoundedRect},
    text_style,
};

use crate::{
    Scheme,
    anim::Value,
    skin,
    tokens::{motion, typescale},
};

/// A slider size, by track thickness.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderSize {
    /// 16 dp track, the default.
    #[default]
    ExtraSmall,
    /// 24 dp track.
    Small,
    /// 40 dp track.
    Medium,
    /// 56 dp track.
    Large,
    /// 96 dp track.
    ExtraLarge,
}

impl SliderSize {
    /// Track thickness, handle height and the track's outer corner.
    fn metrics(self) -> (f32, f32, f32) {
        match self {
            Self::ExtraSmall => (16.0, 44.0, 8.0),
            Self::Small => (24.0, 44.0, 8.0),
            Self::Medium => (40.0, 52.0, 12.0),
            Self::Large => (56.0, 68.0, 16.0),
            Self::ExtraLarge => (96.0, 108.0, 28.0),
        }
    }
}

const GAP: f32 = 6.0;
const HANDLE: f32 = 4.0;
const INNER: f32 = 2.0;
const DOT: f32 = 4.0;

mod handle;
pub use handle::{SLIDER, Slider};

/// The control inside a [`Slider`].
pub struct SliderControl {
    /// The end handle, and the start handle of a range slider.
    thumbs: [Behavior; 2],
    range: bool,
    /// Which handle the keyboard moves.
    active: usize,
    pub(crate) size: SliderSize,
    /// Shows a dot at every step.
    pub(crate) ticks: bool,
    /// Fills from the middle of the range, for values around a center.
    pub(crate) centered: bool,
    bubble: Paragraph,
    /// The theme font size the bubble's label scales from.
    font_size: f32,
    widths: [Value; 2],
    glide: [Value; 2],
}

impl SliderControl {
    fn new(fonts: &std::cell::RefCell<TextSystem>, theme: &Theme, range: Range) -> Result<Self> {
        let mut style = text_style(theme);
        typescale::LABEL_LARGE.apply(&mut style);
        Ok(Self {
            thumbs: [Behavior::new(range), Behavior::new(range)],
            range: false,
            active: 0,
            size: SliderSize::default(),
            ticks: false,
            centered: false,
            bubble: fonts.borrow_mut().paragraph("", &style)?,
            font_size: theme.font_size,
            widths: Default::default(),
            glide: Default::default(),
        })
    }

    /// The value; a range slider's end value.
    pub fn value(&self) -> f64 {
        self.thumbs[0].range().value()
    }

    /// A range slider's start value.
    pub fn start(&self) -> Option<f64> {
        self.range.then(|| self.thumbs[1].range().value())
    }

    pub(crate) fn behaviors(&mut self) -> &mut [Behavior; 2] {
        &mut self.thumbs
    }

    pub(crate) fn set_range(&mut self, range: bool) {
        self.range = range;
        if range {
            let min = self.thumbs[0].range().min();
            let _ = self.thumbs[1].range_mut().set_value(min);
        }
        self.active = 0;
    }

    fn pressed(&self) -> Option<usize> {
        (0..2).find(|&i| self.thumbs[i].is_pressed() && (i == 0 || self.range))
    }

    /// The handle centers' travel: from the start to the end of the track.
    fn track(size: Size) -> (f32, f32) {
        let start = HANDLE / 2.0;
        (start, (size.width - HANDLE).max(0.0))
    }

    /// Keeps a range slider's start at or below its end.
    fn order(&mut self) {
        if self.range {
            let (end, start) = (
                self.thumbs[0].range().value(),
                self.thumbs[1].range().value(),
            );
            if start > end {
                let moved = self.active;
                let other = 1 - moved;
                let value = self.thumbs[moved].range().value();
                let _ = self.thumbs[other].range_mut().set_value(value);
            }
        }
    }
}

impl Control for SliderControl {
    fn kind(&self) -> &'static ControlKind {
        &SLIDER
    }
    fn interactive(&self) -> bool {
        true
    }
    fn drags(&self) -> bool {
        true
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            pressed: self.pressed().is_some(),
            hovered: Some(self.thumbs.iter().any(Behavior::is_hovered)),
            ..Default::default()
        }
    }
    fn set_enabled(&mut self, _: &mut TextSystem, enabled: bool) -> Outcome {
        let outcome = self.thumbs[1].set_enabled(enabled);
        merge(self.thumbs[0].set_enabled(enabled), outcome)
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let (start, extent) = Self::track(cx.size);
        let input = match cx.logical(input) {
            Input::Pointer(mut pointer) => {
                pointer.position.x -= start;
                if cx.rtl {
                    pointer.position.x = extent - pointer.position.x;
                }
                if self.range
                    && self.pressed().is_none()
                    && matches!(pointer.kind, aegle_ui::PointerKind::Down { .. })
                {
                    // A press takes the nearer handle.
                    let at = |i: usize| self.thumbs[i].range().fraction() as f32 * extent;
                    let x = pointer.position.x;
                    self.active = usize::from((at(1) - x).abs() < (at(0) - x).abs());
                }
                Input::Pointer(pointer)
            }
            input => input,
        };
        let outcome = match input {
            Input::Focus(_) | Input::Cancel => {
                let outcome = self.thumbs[1].handle(input, extent)?;
                merge(self.thumbs[0].handle(input, extent)?, outcome)
            }
            input => self.thumbs[self.active].handle(input, extent)?,
        };
        self.order();
        if let Some(i) = self.pressed() {
            let range = self.thumbs[i].range();
            let text = format_value(range.value(), range.step());
            if self.bubble.text() != text {
                let mut style = aegle_text::TextStyle {
                    size: self.font_size,
                    ..Default::default()
                };
                typescale::LABEL_LARGE.apply(&mut style);
                cx.fonts.update(&mut self.bubble, &text, &style)?;
            }
        }
        Ok(outcome)
    }
    fn hover(
        &mut self,
        _: &mut InputCx<'_>,
        pointer: aegle_ui::PointerId,
        _: Input<'_>,
    ) -> Result<Outcome> {
        Ok(self.thumbs[self.active].update_hover(pointer, true))
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        let (_, handle, _) = self.size.metrics();
        Ok(Size::new(200.0, handle))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let a = *cx.appearance;
        let (thick, tall, outer) = self.size.metrics();
        let (start, extent) = Self::track(size);
        let middle = size.height / 2.0;
        let mut moving = false;
        let mut centers = [0.0; 2];
        let count = if self.range { 2 } else { 1 };
        let thumbs = self.thumbs.iter().zip(&mut self.glide).zip(&mut centers);
        for ((thumb, glide), center) in thumbs.take(count) {
            let fraction = thumb.range().fraction() as f32;
            // Programmatic changes glide; drags follow the pointer.
            let snap = reduced || thumb.is_pressed();
            let (f, gliding) = glide.at(fraction, motion::FAST_SPATIAL, now, snap);
            let f = if cx.rtl { 1.0 - f } else { f };
            *center = start + extent * f.clamp(0.0, 1.0);
            moving |= gliding;
        }
        // Handles from left to right, with the active span between them.
        let (lo, hi) = match (self.range, self.centered) {
            (true, _) => (centers[0].min(centers[1]), centers[0].max(centers[1])),
            (false, true) => {
                let mid = start + extent / 2.0;
                (centers[0].min(mid) - 0.01, centers[0].max(mid) + 0.01)
            }
            (false, false) if cx.rtl => (centers[0], size.width),
            (false, false) => (0.0, centers[0]),
        };
        let handles: &[f32] = &centers[..count];
        let s = Scheme::of(cx.theme);
        let top = middle - thick / 2.0;
        // Segments of track between the handles' gaps.
        let mut edges = vec![0.0];
        for &c in handles {
            edges.push(c - HANDLE / 2.0 - GAP);
            edges.push(c + HANDLE / 2.0 + GAP);
        }
        // A centered slider's span starts at the middle, without a gap.
        let mid = start + extent / 2.0;
        if self.centered && !self.range {
            edges.extend([mid, mid]);
        }
        edges.push(size.width);
        edges.sort_by(f32::total_cmp);
        let corner = |x: f32| match x {
            x if x <= 0.0 || x >= size.width => outer,
            x if self.centered && (x - mid).abs() < 0.01 => 0.0,
            _ => INNER,
        };
        for pair in edges.chunks(2) {
            let (x0, x1) = (pair[0], pair[1]);
            if x1 - x0 < 0.5 {
                continue;
            }
            let active = (x0 + x1) / 2.0 > lo && (x0 + x1) / 2.0 < hi;
            let color = if active { a.indicator } else { a.border_color };
            let rect = Rect::new(x0, top, x1 - x0, thick);
            segment(cx.builder, rect, corner(x0), corner(x1), color)?;
        }
        // Stop indicators: every step with ticks, else the track's ends.
        let dot = |builder: &mut aegle_ui::scene::SceneBuilder, x: f32, color: Color| {
            let rect = Rect::new(x - DOT / 2.0, middle - DOT / 2.0, DOT, DOT);
            builder
                .fill(RoundedRect::new(rect, DOT / 2.0)?, color)
                .map(drop)
        };
        let clear = |x: f32| {
            handles
                .iter()
                .all(|&c| (c - x).abs() > HANDLE / 2.0 + GAP + DOT)
        };
        let on_active = if cx.visual.enabled {
            s.secondary_container
        } else {
            s.surface
        };
        let range = self.thumbs[0].range();
        let steps = (range.step() > 0.0)
            .then(|| ((range.max() - range.min()) / range.step()).round() as usize)
            .filter(|&n| self.ticks && n <= 200);
        let inset = thick / 2.0;
        let stops: Vec<f32> = match steps {
            Some(n) => (0..=n)
                .map(|i| inset + (size.width - 2.0 * inset) * i as f32 / n as f32)
                .collect(),
            None => vec![size.width - inset],
        };
        for x in stops {
            if clear(x) {
                let active = x > lo && x < hi;
                dot(cx.builder, x, if active { on_active } else { a.indicator })?;
            }
        }
        // The handles narrow while dragged.
        let pressed = self.pressed();
        for (i, &c) in handles.iter().enumerate() {
            let target = if pressed == Some(i) {
                HANDLE / 2.0
            } else {
                HANDLE
            };
            let (w, active) = self.widths[i].at(target, motion::FAST_SPATIAL, now, reduced);
            moving |= active;
            let rect = Rect::new(c - w / 2.0, middle - tall / 2.0, w, tall);
            cx.builder
                .fill(RoundedRect::new(rect, w / 2.0)?, a.indicator)?;
            if cx.visual.focused && cx.visual.enabled && i == self.active {
                let ring = Rect::new(c - HANDLE / 2.0, middle - tall / 2.0, HANDLE, tall);
                skin::focus_ring(cx.builder, ring, HANDLE / 2.0, a.focus_color)?;
            }
        }
        // The value above the dragged handle.
        if let Some(i) = pressed {
            let label = self.bubble.size();
            let w = (label.width + 32.0).max(48.0);
            let rect = Rect::new(
                centers[i] - w / 2.0,
                middle - tall / 2.0 - 4.0 - 44.0,
                w,
                44.0,
            );
            cx.builder
                .fill(RoundedRect::new(rect, 22.0)?, s.inverse_surface)?;
            let at = Point::new(
                centers[i] - label.width / 2.0,
                rect.origin.y + (44.0 - label.height) / 2.0,
            );
            cx.builder
                .push_transform(Affine::translation(at.x, at.y)?)?;
            self.bubble
                .paint_with_color(cx.builder, s.inverse_on_surface)?;
            cx.builder.pop()?;
        }
        if moving {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::{Action, Role};
        let range = self.thumbs[0].range();
        cx.node.set_role(Role::Slider);
        cx.node.set_numeric_value(range.value());
        cx.node.set_min_numeric_value(range.min());
        cx.node.set_max_numeric_value(range.max());
        if range.step() > 0.0 {
            cx.node.set_numeric_value_step(range.step());
        }
        if cx.enabled {
            for action in [
                Action::Focus,
                Action::Increment,
                Action::Decrement,
                Action::SetValue,
            ] {
                cx.node.add_action(action);
            }
        }
    }
    #[cfg(feature = "accessibility")]
    fn action_input(
        &self,
        action: aegle_ui::accesskit::Action,
        data: Option<&aegle_ui::accesskit::ActionData>,
    ) -> Option<Input<'static>> {
        use aegle_ui::accesskit::{Action, ActionData};
        match (action, data) {
            (Action::Increment, _) => Some(Input::Increment),
            (Action::Decrement, _) => Some(Input::Decrement),
            (Action::SetValue, Some(ActionData::NumericValue(v))) => Some(Input::SetValue(*v)),
            _ => None,
        }
    }
}

fn merge(mut a: Outcome, b: Outcome) -> Outcome {
    a.repaint |= b.repaint;
    a.semantics |= b.semantics;
    a.handled |= b.handled;
    a.focus |= b.focus;
    a.action = a.action.or(b.action);
    a.capture = a.capture.or(b.capture);
    a
}

/// A value as the bubble shows it: whole numbers for whole steps.
fn format_value(value: f64, step: f64) -> String {
    if step >= 1.0 || step == 0.0 && value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// A track segment with independent left and right corners.
fn segment(
    builder: &mut aegle_ui::scene::SceneBuilder,
    rect: Rect,
    left: f32,
    right: f32,
    color: Color,
) -> Result {
    if (left - right).abs() < 0.01 {
        builder.fill(RoundedRect::new(rect, left)?, color)?;
    } else {
        builder.fill_path(&crate::pressable::shape::path(rect, left, right)?, color)?;
    }
    Ok(())
}
