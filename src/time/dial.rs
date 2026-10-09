//! The clock dial of a time picker: 256 dp across, hours (an inner ring
//! for 12–23 in 24-hour time) or minutes around it, and a hand ending in a
//! 48 dp selector. Dragging or clicking sets the value; arrows step it.

use std::{cell::RefCell, f32::consts::TAU};

use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    ControlKind, Key, Point, Rect, Result, Size,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, PathBuilder, RoundedRect, Stroke},
};

use crate::{Scheme, color::alpha, skin, tokens::typescale};

const SIZE: f32 = 256.0;
const SELECTOR: f32 = 48.0;
const OUTER: f32 = SIZE / 2.0 - SELECTOR / 2.0 - 4.0;
const INNER: f32 = OUTER - 32.0;

/// What the dial sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Hour,
    Minute,
}

/// The control inside a time picker's dial.
pub struct DialControl {
    pub(crate) mode: Mode,
    /// 0 to 23.
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) h24: bool,
    /// "12, 1 … 11", "00, 5 … 55" and "00, 13 … 23".
    labels: [Vec<Paragraph>; 4],
    dragging: bool,
}

impl DialControl {
    pub(crate) fn new(fonts: &RefCell<TextSystem>, style: &TextStyle<'_>) -> Result<Self> {
        let mut fonts = fonts.borrow_mut();
        let mut ring = |f: &dyn Fn(u32) -> String| -> Result<Vec<Paragraph>> {
            (0..12).map(|i| Ok(fonts.paragraph(f(i), style)?)).collect()
        };
        let labels = [
            ring(&|i| if i == 0 { "12".into() } else { i.to_string() })?,
            ring(&|i| format!("{:02}", i * 5))?,
            ring(&|i| (i + 12).to_string())?,
            ring(&|i| if i == 0 { "00".into() } else { i.to_string() })?,
        ];
        Ok(Self {
            mode: Mode::Hour,
            hour: 0,
            minute: 0,
            h24: false,
            labels,
            dragging: false,
        })
    }

    /// The selector's angle (0 at twelve o'clock, clockwise) and radius.
    fn hand(&self) -> (f32, f32) {
        match self.mode {
            Mode::Hour => {
                let inner = self.h24 && self.hour >= 12;
                (
                    f32::from(self.hour % 12) / 12.0 * TAU,
                    if inner { INNER } else { OUTER },
                )
            }
            Mode::Minute => (f32::from(self.minute) / 60.0 * TAU, OUTER),
        }
    }

    /// Sets the value at a point; returns whether it changed.
    fn set_at(&mut self, at: Point) -> bool {
        let (dx, dy) = (at.x - SIZE / 2.0, at.y - SIZE / 2.0);
        let turn = (dx.atan2(-dy) / TAU).rem_euclid(1.0);
        let before = (self.hour, self.minute);
        match self.mode {
            Mode::Hour => {
                let step = ((turn * 12.0).round() as u8) % 12;
                let inner = self.h24 && dx.hypot(dy) < (OUTER + INNER) / 2.0;
                self.hour = match (self.h24, inner) {
                    (true, true) => step + 12,
                    (true, false) => step,
                    // 12-hour time keeps the period.
                    (false, _) => step + if self.hour >= 12 { 12 } else { 0 },
                };
            }
            Mode::Minute => self.minute = ((turn * 60.0).round() as u8) % 60,
        }
        before != (self.hour, self.minute)
    }

    fn step(&mut self, by: i8) {
        match self.mode {
            Mode::Hour => {
                let span = if self.h24 { 24 } else { 12 };
                let base = if self.h24 { 0 } else { self.hour / 12 * 12 };
                let h = (i16::from(self.hour - base) + i16::from(by)).rem_euclid(span) as u8;
                self.hour = base + h;
            }
            Mode::Minute => {
                self.minute = (i16::from(self.minute) + i16::from(by)).rem_euclid(60) as u8
            }
        }
    }
}

impl Control for DialControl {
    fn kind(&self) -> &'static ControlKind {
        &super::DIAL
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
    fn text_role(&self, style: &mut TextStyle<'_>) {
        typescale::BODY_LARGE.apply(style);
    }
    fn restyle(&mut self, fonts: &mut TextSystem, style: &TextStyle<'_>) -> Result {
        for text in self.labels.iter_mut().flatten() {
            fonts.restyle(text, style)?;
        }
        Ok(())
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            pressed: self.dragging,
            ..Default::default()
        }
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let mut outcome = Outcome::default();
        match input {
            Input::Pointer(p) => {
                let changed = match p.kind {
                    aegle_ui::PointerKind::Down { .. } => {
                        self.dragging = true;
                        outcome.focus = true;
                        outcome.capture = Some(aegle_ui::control::Capture::Acquire(p.id));
                        self.set_at(p.position)
                    }
                    aegle_ui::PointerKind::Move if self.dragging => self.set_at(p.position),
                    aegle_ui::PointerKind::Up if self.dragging => {
                        self.dragging = false;
                        outcome.capture = Some(aegle_ui::control::Capture::Release(p.id));
                        let changed = self.set_at(p.position);
                        // Choosing the hour moves on to the minutes.
                        if self.mode == Mode::Hour {
                            self.mode = Mode::Minute;
                            cx.deferred.push(Box::new(super::show_mode));
                        }
                        changed
                    }
                    _ => false,
                };
                if changed {
                    outcome.action = Some(Action::Change);
                }
                outcome.handled = true;
                outcome.repaint = true;
            }
            Input::Key(key) if key.pressed => {
                let by = match key.key {
                    Key::Right | Key::Up => 1,
                    Key::Left | Key::Down => -1,
                    _ => return Ok(outcome),
                };
                self.step(if self.mode == Mode::Minute {
                    by * 5
                } else {
                    by
                });
                outcome.action = Some(Action::Change);
                outcome.handled = true;
                outcome.repaint = true;
            }
            Input::Cancel => self.dragging = false,
            _ => {}
        }
        Ok(outcome)
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(Size::new(SIZE, SIZE))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let s = Scheme::of(cx.theme);
        let dial = Rect::new(0.0, 0.0, SIZE, SIZE);
        cx.builder.fill(
            RoundedRect::new(dial, SIZE / 2.0)?,
            s.surface_container_highest,
        )?;
        let c = SIZE / 2.0;
        let (angle, radius) = self.hand();
        let at = |angle: f32, r: f32| Point::new(c + r * angle.sin(), c - r * angle.cos());
        let end = at(angle, radius);
        let accent = if cx.visual.enabled {
            s.primary
        } else {
            alpha(s.on_surface, 0.38)
        };
        let mut hand = PathBuilder::new();
        hand.move_to(Point::new(c, c)).line_to(end);
        cx.builder.stroke_path(
            &hand.finish(aegle_ui::scene::FillRule::NonZero)?,
            accent,
            Stroke::new(2.0),
        )?;
        let dot = |p: Point, d: f32| {
            RoundedRect::new(Rect::new(p.x - d / 2.0, p.y - d / 2.0, d, d), d / 2.0)
        };
        cx.builder.fill(dot(Point::new(c, c), 8.0)?, accent)?;
        cx.builder.fill(dot(end, SELECTOR)?, accent)?;
        if cx.visual.focused {
            let r = Rect::new(
                end.x - SELECTOR / 2.0,
                end.y - SELECTOR / 2.0,
                SELECTOR,
                SELECTOR,
            );
            skin::focus_ring(cx.builder, r, SELECTOR / 2.0, s.secondary)?;
        }
        let rings: &[(usize, f32)] = match (self.mode, self.h24) {
            (Mode::Minute, _) => &[(1, OUTER)],
            (Mode::Hour, false) => &[(0, OUTER)],
            (Mode::Hour, true) => &[(3, OUTER), (2, INNER)],
        };
        for &(ring, r) in rings {
            for (i, text) in self.labels[ring].iter().enumerate() {
                let p = at(i as f32 / 12.0 * TAU, r);
                let size = text.size();
                // A label under the selector reads in on-primary.
                let under = (p.x - end.x).hypot(p.y - end.y) < SELECTOR / 2.0 - 2.0;
                let color = if under { s.on_primary } else { s.on_surface };
                cx.builder.push_transform(Affine::translation(
                    p.x - size.width / 2.0,
                    p.y - size.height / 2.0,
                )?)?;
                text.paint_with_color(cx.builder, color)?;
                cx.builder.pop()?;
            }
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::Role;
        cx.node.set_role(Role::Slider);
        let (value, max) = match self.mode {
            Mode::Hour => (f64::from(self.hour), 23.0),
            Mode::Minute => (f64::from(self.minute), 59.0),
        };
        cx.node.set_numeric_value(value);
        cx.node.set_min_numeric_value(0.0);
        cx.node.set_max_numeric_value(max);
    }
}
