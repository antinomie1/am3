//! The control behind every button-like component: a container with an
//! optional elevation and outline, a ripple, and centered icon, label and
//! trailing icon. Its kind's skin gives the colors; its [`Spec`] gives the
//! geometry, the shape morphs and the elevation, which it animates itself
//! with the expressive spatial springs.

mod methods;
mod shape;

use std::cell::RefCell;

use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    ControlKind, NodeId, Point, Result, Size, State,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, Stroke},
    text_style,
};

pub(crate) use methods::{add, pressable_methods, relook};
use shape::{Outline, Shape};

use crate::{
    Icon, Scheme,
    anim::{Ripple, Value},
    color::alpha,
    skin,
    tokens::{Corner, TypeStyle, motion},
};

/// The geometry of one size of a pressable component.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Spec {
    pub height: f32,
    /// A fixed width, as icon buttons and FABs have.
    pub width: Option<f32>,
    pub min_width: f32,
    /// Space before the first and after the last content.
    pub padding: f32,
    pub icon: f32,
    pub gap: f32,
    pub text: TypeStyle,
    /// Outer corners at rest, pressed and selected.
    pub corners: [Corner; 3],
    /// Corners where the control joins a neighbor, in the same states.
    pub inner: [Corner; 3],
    /// Hover and focus take the inner corners to their pressed shape too.
    pub hover_inner: bool,
    /// Outline width, scaling the skin's border width.
    pub outline: f32,
    /// Elevation level at rest and while hovered.
    pub elevation: [f32; 2],
}

/// Which component a pressable control is, with its variant: this picks
/// its kind and its [`Spec`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Look {
    Button(crate::button::ButtonLook),
    Icon(crate::icon_button::IconLook),
    Fab(crate::fab::FabLook),
    Segment(crate::segmented::SegmentLook),
    FabItem(crate::fab::FabColor),
}

impl Look {
    fn kind(self, toggle: bool) -> &'static ControlKind {
        match self {
            Self::Button(look) => look.kind(toggle),
            Self::Icon(look) => look.kind(toggle),
            Self::Fab(look) => look.kind(),
            Self::Segment(_) => &crate::segmented::SEGMENT,
            Self::FabItem(color) => crate::fab_menu::item_kind(color),
        }
    }

    fn spec(self) -> Spec {
        match self {
            Self::Button(look) => look.spec(),
            Self::Icon(look) => look.spec(),
            Self::Fab(look) => look.spec(),
            Self::Segment(look) => look.spec(),
            Self::FabItem(_) => crate::fab_menu::item_spec(),
        }
    }
}

/// Where a press started, before the next frame gives the ripple its time.
#[derive(Debug)]
enum Pending {
    Press(Point),
    Release,
}

/// The control inside every button-like handle.
pub struct PressableControl {
    pub(crate) look: Look,
    spec: Spec,
    button: aegle_controls::Button,
    /// `Some` for a toggle button: whether it is selected.
    pub(crate) selected: Option<bool>,
    /// Selecting it deselects its pressable siblings, and selecting it
    /// again changes nothing.
    pub(crate) exclusive: bool,
    /// Whether the start and end sides join a neighbor.
    pub(crate) joined: [bool; 2],
    pub(crate) icon: Option<Icon>,
    pub(crate) trailing: Option<Icon>,
    /// The icon while selected, as a segment's check mark or an open FAB
    /// menu's close icon.
    pub(crate) selected_icon: Option<Icon>,
    /// The trailing icon turns over while selected, as on a split button's
    /// menu part.
    pub(crate) turn: bool,
    /// The label names the control without showing, as on an icon button.
    pub(crate) hide_label: bool,
    /// In a standard button group: a press widens it into its neighbors.
    pub(crate) bounce: bool,
    /// How far the start and end sides reach out (or, negative, in) while
    /// a neighbor in a standard group is pressed.
    pub(crate) reach: [f32; 2],
    extent: [Value; 2],
    width: f32,
    label: Paragraph,
    radius: [Value; 2],
    lift: Value,
    ripple: Option<Ripple>,
    pending: Vec<Pending>,
    shape: Shape,
    edge: Shape,
}

impl PressableControl {
    pub(crate) fn new(
        fonts: &RefCell<TextSystem>,
        theme: &aegle_ui::Theme,
        look: Look,
        text: &str,
    ) -> Result<Self> {
        let spec = look.spec();
        let mut style = text_style(theme);
        spec.text.apply(&mut style);
        Ok(Self {
            look,
            spec,
            button: aegle_controls::Button::new(),
            selected: None,
            exclusive: false,
            joined: [false; 2],
            icon: None,
            trailing: None,
            selected_icon: None,
            turn: false,
            hide_label: false,
            bounce: false,
            reach: [0.0; 2],
            extent: Default::default(),
            width: 0.0,
            label: fonts.borrow_mut().paragraph(text, &style)?,
            radius: Default::default(),
            lift: Value::default(),
            ripple: None,
            pending: Vec::new(),
            shape: Shape::default(),
            edge: Shape::default(),
        })
    }

    /// Changes the look; the caller relayouts and reshapes the label.
    pub(crate) fn set_look(&mut self, look: Look) {
        self.look = look;
        self.spec = look.spec();
    }

    /// The label text.
    pub fn text(&self) -> &str {
        self.label.text()
    }

    /// Whether a toggle button is selected; `None` for a plain button.
    pub fn selected(&self) -> Option<bool> {
        self.selected
    }

    fn leading(&self) -> Option<Icon> {
        match (&self.selected_icon, self.selected) {
            (Some(icon), Some(true)) => Some(icon.clone()),
            _ => self.icon.clone(),
        }
    }

    /// Widths of the icon, label and trailing icon that show.
    fn parts(&self) -> [Option<f32>; 3] {
        let label = self.label.size().width;
        [
            self.leading().map(|_| self.spec.icon),
            (!self.hide_label && label > 0.0).then_some(label),
            self.trailing.as_ref().map(|_| self.spec.icon),
        ]
    }

    fn content_width(&self) -> f32 {
        let parts = self.parts();
        let count = parts.iter().flatten().count();
        parts.iter().flatten().sum::<f32>() + self.spec.gap * count.saturating_sub(1) as f32
    }

    /// The corner of the start (0) or end (1) side in `state`.
    fn corner(&self, side: usize, state: aegle_ui::VisualState) -> Corner {
        let (corners, hover) = if self.joined[side] {
            let hover = self.spec.hover_inner && (state.hovered || state.focused);
            (&self.spec.inner, hover)
        } else {
            (&self.spec.corners, false)
        };
        if state.pressed {
            corners[1]
        } else if self.selected == Some(true) {
            corners[2]
        } else if hover {
            corners[1]
        } else {
            corners[0]
        }
    }
}

/// Deselects the exclusive pressable siblings of `id`.
fn select_exclusive(state: &mut State, id: NodeId) -> Result {
    let Some(parent) = state.tree.parent(id)? else {
        return Ok(());
    };
    let siblings: Vec<_> = state.tree.children(parent)?.filter(|&n| n != id).collect();
    for sibling in siblings {
        if let Some(other) = state.control_as::<PressableControl>(sibling)
            && other.exclusive
            && other.selected == Some(true)
        {
            other.selected = Some(false);
            let dirty = aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS;
            state.tree.mark_dirty(sibling, dirty)?;
        }
    }
    Ok(())
}

/// Widens a pressed button of a standard group by 15 % into the
/// neighboring buttons, which give way; or returns all of them.
fn bounce(state: &mut State, id: NodeId, pressed: bool) -> Result {
    let Some(parent) = state.tree.parent(id)? else {
        return Ok(());
    };
    let siblings: Vec<_> = state.tree.children(parent)?.collect();
    let index = siblings
        .iter()
        .position(|&n| n == id)
        .expect("a child of its parent");
    let mut grouped = |n: Option<&NodeId>| {
        n.copied().filter(|&n| {
            state
                .control_as::<PressableControl>(n)
                .is_some_and(|c| c.bounce)
        })
    };
    let neighbors = [
        index.checked_sub(1).and_then(|i| grouped(siblings.get(i))),
        grouped(siblings.get(index + 1)),
    ];
    let sides = neighbors.iter().flatten().count();
    let control = state
        .control_as::<PressableControl>(id)
        .expect("a pressable");
    let amount = if pressed && sides > 0 {
        0.15 * control.width / sides as f32
    } else {
        0.0
    };
    control.reach = neighbors.map(|n| if n.is_some() { amount } else { 0.0 });
    let dirty = aegle_ui::Dirty::PAINT;
    state.tree.mark_dirty(id, dirty)?;
    for (side, neighbor) in neighbors.into_iter().enumerate() {
        if let Some(n) = neighbor {
            // The neighbor before gives way at its end, the one after at its start.
            let control = state
                .control_as::<PressableControl>(n)
                .expect("a pressable");
            control.reach[1 - side] = -amount;
            state.tree.mark_dirty(n, dirty)?;
        }
    }
    Ok(())
}

impl Control for PressableControl {
    fn kind(&self) -> &'static ControlKind {
        self.look.kind(self.selected.is_some())
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
    fn paragraph(&self) -> Option<&Paragraph> {
        Some(&self.label)
    }
    fn paragraph_mut(&mut self) -> Option<&mut Paragraph> {
        Some(&mut self.label)
    }
    fn text_role(&self, style: &mut TextStyle<'_>) {
        self.spec.text.apply(style);
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            pressed: self.button.is_pressed(),
            hovered: Some(self.button.is_hovered()),
            checked: self.selected == Some(true),
            ..Default::default()
        }
    }
    fn set_enabled(&mut self, _: &mut TextSystem, enabled: bool) -> Outcome {
        self.button.set_enabled(enabled)
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let was = self.button.is_pressed();
        let mut outcome = self.button.handle(input);
        if self.button.is_pressed() && !was {
            let at = match input {
                Input::Pointer(pointer) => pointer.position,
                _ => Point::new(cx.size.width / 2.0, cx.size.height / 2.0),
            };
            self.pending.push(Pending::Press(at));
        } else if was && !self.button.is_pressed() {
            self.pending.push(Pending::Release);
        }
        if outcome.action == Some(Action::Activate)
            && let Some(selected) = &mut self.selected
        {
            if self.exclusive && *selected {
                outcome.action = None;
            } else {
                *selected = !*selected;
                outcome.action = Some(Action::Change);
                outcome.repaint = true;
                outcome.semantics = true;
                if self.exclusive {
                    cx.deferred.push(Box::new(select_exclusive));
                }
            }
        }
        if self.bounce && was != self.button.is_pressed() {
            let pressed = self.button.is_pressed();
            cx.deferred
                .push(Box::new(move |state, id| bounce(state, id, pressed)));
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
        let width = self.spec.width.unwrap_or_else(|| {
            (self.content_width() + 2.0 * self.spec.padding).max(self.spec.min_width)
        });
        Ok(Size::new(width, self.spec.height))
    }
    fn baseline(&self, size: Size, _: f32) -> Option<f32> {
        Some((size.height - self.label.size().height) / 2.0 + self.label.first_baseline()?)
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let scheme = Scheme::of(cx.theme);
        let a = *cx.appearance;
        let side = size.width.min(size.height);
        let pressed = cx.visual.pressed;
        let mut moving = false;
        let mut radii = [0.0; 2];
        for (i, radius) in radii.iter_mut().enumerate() {
            let target = self.corner(i, cx.visual).radius(side);
            let (value, active) = self.radius[i].at(target, motion::FAST_SPATIAL, now, reduced);
            *radius = value.max(0.0);
            moving |= active;
        }
        let level = match () {
            _ if !cx.visual.enabled => 0.0,
            _ if cx.visual.hovered && !pressed => self.spec.elevation[1],
            _ => self.spec.elevation[0],
        };
        let (level, lifting) = self.lift.at(level, motion::FAST_EFFECTS, now, reduced);
        let [left, right] = if cx.rtl { [radii[1], radii[0]] } else { radii };
        // A standard group's press reaches into the neighbors.
        self.width = size.width;
        let mut reach = [0.0; 2];
        for (i, out) in reach.iter_mut().enumerate() {
            let (value, active) =
                self.extent[i].at(self.reach[i], motion::FAST_SPATIAL, now, reduced);
            *out = value;
            moving |= active;
        }
        let [out_left, out_right] = if cx.rtl { [reach[1], reach[0]] } else { reach };
        let rect = aegle_ui::Rect::new(
            -out_left,
            0.0,
            size.width + out_left + out_right,
            size.height,
        );

        for pending in self.pending.drain(..) {
            match pending {
                Pending::Press(at) => self.ripple = Some(Ripple::new(at, now)),
                Pending::Release => {
                    if let Some(ripple) = &mut self.ripple {
                        ripple.release(now);
                    }
                }
            }
        }

        let outline = self.shape.outline(rect, left, right, 0.0)?;
        let bounds = outline.bounds();
        skin::shadow(cx.builder, bounds, level, scheme.shadow)?;
        let mut rippling = false;
        match outline {
            Outline::Rect(rect) => {
                cx.builder.fill(rect, a.background)?;
                if let Some(ripple) = &self.ripple {
                    cx.builder.push_clip(rect)?;
                    rippling = ripple.paint(cx.builder, size, a.foreground, now)?;
                    cx.builder.pop()?;
                }
            }
            Outline::Path(path, _) => {
                cx.builder.fill_path(path, a.background)?;
                // A circle cannot be clipped to a path: the press shows as
                // an even layer over the whole shape instead.
                if let Some(ripple) = &self.ripple {
                    let opacity = ripple.opacity(now);
                    rippling = opacity > 0.0;
                    cx.builder.fill_path(path, alpha(a.foreground, opacity))?;
                }
            }
        }
        rippling &= !reduced;
        if !rippling {
            self.ripple = None;
        }
        let width = a.border_width * self.spec.outline;
        if width > 0.0 && a.border_color.to_rgba()[3] > 0 {
            match self.edge.outline(rect, left, right, width / 2.0)? {
                Outline::Rect(rect) => cx.builder.stroke(rect, a.border_color, width)?,
                Outline::Path(path, _) => {
                    cx.builder
                        .stroke_path(path, a.border_color, Stroke::new(width))?
                }
            };
        }

        // Content, centered and mirrored right to left.
        let parts = self.parts();
        let mut x = (size.width - self.content_width()) / 2.0;
        let shift = (out_right - out_left) / 2.0;
        let mut at = [None; 3];
        for (slot, width) in at.iter_mut().zip(parts) {
            if let Some(width) = width {
                *slot = Some(shift + if cx.rtl { size.width - x - width } else { x });
                x += width + self.spec.gap;
            }
        }
        let (icon, middle) = (self.spec.icon, size.height / 2.0);
        let top = middle - icon / 2.0;
        if let (Some(x), Some(glyph)) = (at[0], self.leading()) {
            glyph.paint(cx.builder, Point::new(x, top), icon, a.indicator)?;
        }
        if let Some(x) = at[1] {
            let y = middle - self.label.size().height / 2.0;
            cx.builder.push_transform(Affine::translation(x, y)?)?;
            self.label.paint_with_color(cx.builder, a.foreground)?;
            cx.builder.pop()?;
        }
        if let (Some(x), Some(glyph)) = (at[2], &self.trailing) {
            let turned = self.turn && self.selected == Some(true);
            if turned {
                // Turned over about the icon's center.
                let flip = Affine::new([1.0, 0.0, 0.0, -1.0, 0.0, 2.0 * middle])?;
                cx.builder.push_transform(flip)?;
            }
            glyph.paint(cx.builder, Point::new(x, top), icon, a.indicator)?;
            if turned {
                cx.builder.pop()?;
            }
        }
        if cx.visual.focused && cx.visual.enabled {
            skin::focus_ring(cx.builder, rect, bounds.radius(), a.focus_color)?;
        }
        if moving || lifting || rippling {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::{Action, Role, Toggled};
        cx.node.set_role(Role::Button);
        if let Some(selected) = self.selected {
            cx.node.set_toggled(if selected {
                Toggled::True
            } else {
                Toggled::False
            });
        }
        if !cx.labelled && !self.label.text().is_empty() {
            cx.node.set_label(self.label.text());
        }
        if cx.enabled {
            cx.node.add_action(Action::Focus);
            cx.node.add_action(Action::Click);
        }
    }
    #[cfg(feature = "accessibility")]
    fn action_input(
        &self,
        action: aegle_ui::accesskit::Action,
        _: Option<&aegle_ui::accesskit::ActionData>,
    ) -> Option<Input<'static>> {
        (action == aegle_ui::accesskit::Action::Click).then_some(Input::Activate)
    }
}
