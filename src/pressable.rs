//! The control behind every button-like component: a container with an
//! optional elevation and outline, a ripple, and centered icon, label and
//! trailing icon. Its kind's skin gives the colors; its [`Spec`] gives the
//! geometry, the shape morphs and the elevation, which it animates itself
//! with the expressive spatial springs.

use std::cell::RefCell;

use aegle_layout::Style;
use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    Container, ControlKind, Node, Point, Result, Size,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, RoundedRect},
    text_style,
};

use crate::{
    Icon, Scheme,
    anim::{Ripple, Value},
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
    pub corner: Corner,
    pub pressed: Corner,
    /// The shape of a selected toggle.
    pub selected: Corner,
    /// Outline width, scaling the skin's border width.
    pub outline: f32,
    /// Elevation level at rest and while hovered.
    pub elevation: [f32; 2],
}

/// Where a press started, before the next frame gives the ripple its time.
#[derive(Debug)]
enum Pending {
    Press(Point),
    Release,
}

/// Which component a pressable control is, with its variant: this picks
/// its kind and its [`Spec`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Look {
    Button(crate::button::ButtonLook),
    Icon(crate::icon_button::IconLook),
    Fab(crate::fab::FabLook),
}

impl Look {
    fn kind(self, toggle: bool) -> &'static ControlKind {
        match self {
            Self::Button(look) => look.kind(toggle),
            Self::Icon(look) => look.kind(toggle),
            Self::Fab(look) => look.kind(),
        }
    }

    fn spec(self) -> Spec {
        match self {
            Self::Button(look) => look.spec(),
            Self::Icon(look) => look.spec(),
            Self::Fab(look) => look.spec(),
        }
    }
}

/// The control inside every button-like handle.
pub struct PressableControl {
    pub(crate) look: Look,
    spec: Spec,
    button: aegle_controls::Button,
    /// `Some` for a toggle button: whether it is selected.
    pub(crate) selected: Option<bool>,
    pub(crate) icon: Option<Icon>,
    pub(crate) trailing: Option<Icon>,
    /// The label names the control without showing, as on an icon button.
    pub(crate) hide_label: bool,
    label: Paragraph,
    radius: Value,
    lift: Value,
    ripple: Option<Ripple>,
    pending: Vec<Pending>,
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
            icon: None,
            trailing: None,
            hide_label: false,
            label: fonts.borrow_mut().paragraph(text, &style)?,
            radius: Value::default(),
            lift: Value::default(),
            ripple: None,
            pending: Vec::new(),
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

    fn label_width(&self) -> f32 {
        if self.hide_label {
            0.0
        } else {
            self.label.size().width
        }
    }

    fn content_width(&self) -> f32 {
        let label = self.label_width();
        let parts = [
            self.icon.as_ref().map(|_| self.spec.icon),
            (label > 0.0).then_some(label),
            self.trailing.as_ref().map(|_| self.spec.icon),
        ];
        let count = parts.iter().flatten().count();
        parts.iter().flatten().sum::<f32>() + self.spec.gap * count.saturating_sub(1) as f32
    }

    fn corner(&self, pressed: bool) -> Corner {
        if pressed {
            self.spec.pressed
        } else if self.selected == Some(true) {
            self.spec.selected
        } else {
            self.spec.corner
        }
    }
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
            *selected = !*selected;
            outcome.action = Some(Action::Change);
            outcome.repaint = true;
            outcome.semantics = true;
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
        let target = self.corner(cx.visual.pressed).radius(side);
        let (radius, morphing) = self.radius.at(target, motion::FAST_SPATIAL, now, reduced);
        let level = match () {
            _ if !cx.visual.enabled => 0.0,
            _ if cx.visual.hovered && !cx.visual.pressed => self.spec.elevation[1],
            _ => self.spec.elevation[0],
        };
        let (level, lifting) = self.lift.at(level, motion::FAST_EFFECTS, now, reduced);
        let shape = RoundedRect::new(
            aegle_ui::Rect::new(0.0, 0.0, size.width, size.height),
            radius,
        )?;
        skin::shadow(cx.builder, shape, level, scheme.shadow)?;
        cx.builder.fill(shape, a.background)?;
        skin::outline(
            cx.builder,
            shape,
            a.border_color,
            a.border_width * self.spec.outline,
        )?;

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
        let mut rippling = false;
        if let Some(ripple) = &self.ripple {
            cx.builder.push_clip(shape)?;
            rippling = !reduced && ripple.paint(cx.builder, size, a.foreground, now)?;
            cx.builder.pop()?;
            if !rippling {
                self.ripple = None;
            }
        }

        // Content, centered and mirrored right to left.
        let mut x = (size.width - self.content_width()) / 2.0;
        let icon = self.spec.icon;
        let mut parts: [Option<f32>; 3] = [None; 3];
        let label = self.label.size();
        let order = [
            self.icon.as_ref().map(|_| icon),
            (self.label_width() > 0.0).then_some(label.width),
            self.trailing.as_ref().map(|_| icon),
        ];
        for (slot, width) in parts.iter_mut().zip(order) {
            if let Some(width) = width {
                *slot = Some(if cx.rtl { size.width - x - width } else { x });
                x += width + self.spec.gap;
            }
        }
        let middle = size.height / 2.0;
        if let (Some(icon_x), Some(glyph)) = (parts[0], &self.icon) {
            glyph.paint(
                cx.builder,
                Point::new(icon_x, middle - icon / 2.0),
                icon,
                a.indicator,
            )?;
        }
        if let Some(label_x) = parts[1] {
            let y = middle - label.height / 2.0;
            cx.builder
                .push_transform(Affine::translation(label_x, y)?)?;
            self.label.paint_with_color(cx.builder, a.foreground)?;
            cx.builder.pop()?;
        }
        if let (Some(icon_x), Some(glyph)) = (parts[2], &self.trailing) {
            glyph.paint(
                cx.builder,
                Point::new(icon_x, middle - icon / 2.0),
                icon,
                a.indicator,
            )?;
        }
        if cx.visual.focused && cx.visual.enabled {
            skin::focus_ring(cx.builder, size, radius, a.focus_color)?;
        }
        if morphing || lifting || rippling {
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

/// Appends a pressable control to `parent`, with the effects spring on its
/// colors.
pub(crate) fn add(
    parent: &Container,
    create: impl FnOnce(&RefCell<TextSystem>, &aegle_ui::Theme) -> Result<PressableControl>,
) -> Result<Node> {
    let node = parent.add(|state, theme| {
        let control = create(&state.fonts, theme)?;
        let mut style = Style {
            flex_shrink: 0.0,
            ..Default::default()
        };
        size(&control.spec, &mut style);
        Ok((Box::new(control), style))
    })?;
    crate::effects(&node)?;
    Ok(node)
}

/// A fixed width keeps the control from stretching in a column; the height
/// is always the size's.
fn size(spec: &Spec, style: &mut Style) {
    use aegle_layout::Dimension;
    style.size.width = spec.width.map_or(Dimension::auto(), Dimension::length);
    style.size.height = Dimension::length(spec.height);
}

/// Changes a pressable control's look or content that its layout and label
/// depend on, then reshapes the label in its (new) type role.
pub(crate) fn relook(node: &Node, change: impl FnOnce(&mut PressableControl)) -> Result {
    node.change(|state, id| {
        let control = state
            .control_as::<PressableControl>(id)
            .ok_or(aegle_ui::UiError::WrongKind)?;
        change(control);
        let style = state.text_style(id);
        let fonts = state.fonts.clone();
        let control = state
            .control_as::<PressableControl>(id)
            .expect("checked above");
        fonts.borrow_mut().restyle(&mut control.label, &style)?;
        let spec = control.spec;
        let mut layout = state.tree.get(id).expect("live node").style().clone();
        size(&spec, &mut layout);
        aegle_layout::set_style(&mut state.tree, id, layout)?;
        state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?;
        Ok(())
    })
}

/// The methods every pressable handle shares.
macro_rules! pressable_methods {
    ($handle:ident) => {
        impl $handle {
            /// Replaces the label.
            pub fn set_text(&self, text: &str) -> aegle_ui::Result {
                self.change(|state, id| state.set_text(id, text))
            }
            /// The label.
            pub fn text(&self) -> aegle_ui::Result<String> {
                self.read(|c| c.text().to_owned())
            }
            /// Sets or removes the leading icon.
            pub fn set_icon(&self, icon: Option<$crate::Icon>) -> aegle_ui::Result {
                $crate::pressable::relook(self, |c| c.icon = icon)
            }
            /// Sets or removes the trailing icon.
            pub fn set_trailing_icon(&self, icon: Option<$crate::Icon>) -> aegle_ui::Result {
                $crate::pressable::relook(self, |c| c.trailing = icon)
            }
            /// Makes it a toggle button (`Some(selected)`) or a plain one.
            /// A toggle flips on activation and reports a change; selected,
            /// it takes the selected colors and shape.
            pub fn set_selected(&self, selected: Option<bool>) -> aegle_ui::Result {
                self.update(|c| c.selected = selected)
            }
            /// Whether a toggle button is selected; `None` for a plain one.
            pub fn selected(&self) -> aegle_ui::Result<Option<bool>> {
                self.read(|c| c.selected())
            }
            /// Activates it through the normal enabled and visible checks.
            pub fn activate(&self) -> aegle_ui::Result {
                self.change(|state, id| state.dispatch(id, aegle_ui::control::Input::Activate))
            }
            /// Adds a handler run on each activation, or each toggle of a
            /// toggle button, after the input batch.
            pub fn on_click(
                &self,
                mut callback: impl FnMut($handle) -> aegle_ui::Result + 'static,
            ) -> aegle_ui::Result {
                self.change(|state, id| state.on_action(id, move |node| callback($handle(node))))
            }
            /// Removes the click handlers.
            pub fn clear_on_click(&self) -> aegle_ui::Result {
                self.change(|state, id| {
                    state.clear_actions(id);
                    Ok(())
                })
            }
        }
    };
}
pub(crate) use pressable_methods;
