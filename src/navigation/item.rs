//! The destination control shared by the navigation bar, rail and drawer:
//! an icon and label with an active indicator behind the icon (stacked)
//! or behind both (inline). Selecting one deselects the other items of its
//! navigation component; the indicator grows in from its center.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    ControlKind, NodeId, Point, Rect, Result, Size,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, RoundedRect},
    text_style,
};

use crate::{
    Icon,
    anim::{Ripple, Value},
    skin,
    tokens::{TypeStyle, motion, typescale},
};

/// How an item lays out its icon and label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Layout {
    /// The label under a 56 × 32 dp indicator, as in a bar or collapsed rail.
    Stacked,
    /// Icon and label in one 56 dp tall pill, as in a drawer or expanded rail.
    Inline,
}

/// The control inside every navigation item.
pub struct NavItemControl {
    pub(crate) layout: Layout,
    pub(crate) icon: Icon,
    pub(crate) selected_icon: Option<Icon>,
    pub(crate) selected: bool,
    /// The navigation component whose items exclude each other.
    pub(crate) group: Option<NodeId>,
    /// Where the icon was last painted, for a badge.
    pub(crate) icon_at: Rc<Cell<Rect>>,
    button: aegle_controls::Button,
    label: Paragraph,
    grow: Value,
    ripple: Option<Ripple>,
    pressed_at: Option<Point>,
    released: bool,
}

const INDICATOR: Size = Size::new(56.0, 32.0);
const ICON: f32 = 24.0;

impl NavItemControl {
    pub(crate) fn new(
        fonts: &RefCell<TextSystem>,
        theme: &aegle_ui::Theme,
        layout: Layout,
        icon: Icon,
        text: &str,
    ) -> Result<Self> {
        let mut style = text_style(theme);
        Self::role(layout).apply(&mut style);
        Ok(Self {
            layout,
            icon,
            selected_icon: None,
            selected: false,
            group: None,
            icon_at: Rc::default(),
            button: aegle_controls::Button::new(),
            label: fonts.borrow_mut().paragraph(text, &style)?,
            grow: Value::default(),
            ripple: None,
            pressed_at: None,
            released: false,
        })
    }

    fn role(layout: Layout) -> TypeStyle {
        match layout {
            Layout::Stacked => typescale::LABEL_MEDIUM,
            Layout::Inline => typescale::LABEL_LARGE,
        }
    }

    /// The label text.
    pub fn text(&self) -> &str {
        self.label.text()
    }

    /// Whether it is the selected destination.
    pub fn selected(&self) -> bool {
        self.selected
    }

    /// The indicator's full box in a control of `size`.
    fn indicator(&self, size: Size) -> Rect {
        match self.layout {
            Layout::Stacked => {
                let top = (size.height - INDICATOR.height - 4.0 - self.label.size().height) / 2.0;
                Rect::new(
                    (size.width - INDICATOR.width) / 2.0,
                    top.max(0.0),
                    INDICATOR.width,
                    INDICATOR.height,
                )
            }
            Layout::Inline => Rect::new(0.0, 0.0, size.width, size.height),
        }
    }

    /// Where the icon sits in a control of `size`, mirrored right to left.
    fn icon_box(&self, size: Size, rtl: bool) -> Rect {
        let indicator = self.indicator(size);
        match self.layout {
            Layout::Stacked => Rect::new(
                indicator.origin.x + (INDICATOR.width - ICON) / 2.0,
                indicator.origin.y + (INDICATOR.height - ICON) / 2.0,
                ICON,
                ICON,
            ),
            Layout::Inline => {
                let x = if rtl { size.width - 16.0 - ICON } else { 16.0 };
                Rect::new(x, (size.height - ICON) / 2.0, ICON, ICON)
            }
        }
    }
}

impl Control for NavItemControl {
    fn kind(&self) -> &'static ControlKind {
        match self.layout {
            Layout::Stacked => &super::NAV_ITEM,
            Layout::Inline => &super::NAV_DRAWER_ITEM,
        }
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
        Self::role(self.layout).apply(style);
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            pressed: self.button.is_pressed(),
            hovered: Some(self.button.is_hovered()),
            checked: self.selected,
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
            self.pressed_at = Some(match input {
                Input::Pointer(pointer) => pointer.position,
                _ => Point::new(cx.size.width / 2.0, cx.size.height / 2.0),
            });
        } else if was && !self.button.is_pressed() {
            self.released = true;
        }
        if outcome.action == Some(Action::Activate) && !self.selected {
            self.selected = true;
            outcome.repaint = true;
            outcome.semantics = true;
            cx.deferred.push(Box::new(super::deselect_others));
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
        let label = self.label.size();
        Ok(match self.layout {
            Layout::Stacked => Size::new(
                INDICATOR.width.max(label.width),
                INDICATOR.height + 4.0 + label.height,
            ),
            Layout::Inline => Size::new(16.0 + ICON + 12.0 + label.width + 24.0, 56.0),
        })
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let a = *cx.appearance;
        let full = self.indicator(size);
        let target = if self.selected { 1.0 } else { 0.0 };
        let (grown, growing) = self.grow.at(target, motion::FAST_SPATIAL, now, reduced);
        let radius = full.size.height / 2.0;
        let pill = RoundedRect::new(full, radius)?;
        // The state layer fills the whole indicator; the selected indicator
        // grows from its center.
        if a.border_color.to_rgba()[3] > 0 {
            cx.builder.fill(pill, a.border_color)?;
        }
        let width = (full.size.width * grown.clamp(0.0, 1.2)).max(0.0);
        if width > 0.5 {
            let r = Rect::new(
                full.origin.x + (full.size.width - width) / 2.0,
                full.origin.y,
                width,
                full.size.height,
            );
            cx.builder
                .fill(RoundedRect::new(r, radius.min(width / 2.0))?, a.background)?;
        }
        if let Some(at) = self.pressed_at.take() {
            let local = Point::new(at.x - full.origin.x, at.y - full.origin.y);
            self.ripple = Some(Ripple::new(local, now));
        }
        if std::mem::take(&mut self.released)
            && let Some(ripple) = &mut self.ripple
        {
            ripple.release(now);
        }
        let rippling = match &self.ripple {
            Some(ripple) if !reduced => {
                cx.builder.push_clip(pill)?;
                cx.builder
                    .push_transform(Affine::translation(full.origin.x, full.origin.y)?)?;
                let visible = ripple.paint(cx.builder, full.size, a.foreground, now)?;
                cx.builder.pop()?;
                cx.builder.pop()?;
                visible
            }
            _ => false,
        };
        if !rippling {
            self.ripple = None;
        }
        let icon = self.icon_box(size, cx.rtl);
        self.icon_at.set(icon);
        let glyph = match (&self.selected_icon, self.selected) {
            (Some(icon), true) => icon,
            _ => &self.icon,
        };
        glyph.paint(cx.builder, icon.origin, ICON, a.indicator)?;
        let label = self.label.size();
        let at = match self.layout {
            Layout::Stacked => Point::new(
                (size.width - label.width) / 2.0,
                full.origin.y + full.size.height + 4.0,
            ),
            Layout::Inline => {
                let x = if cx.rtl {
                    icon.origin.x - 12.0 - label.width
                } else {
                    icon.origin.x + ICON + 12.0
                };
                Point::new(x, (size.height - label.height) / 2.0)
            }
        };
        cx.builder
            .push_transform(Affine::translation(at.x, at.y)?)?;
        self.label.paint_with_color(cx.builder, a.foreground)?;
        cx.builder.pop()?;
        if cx.visual.focused && cx.visual.enabled {
            skin::focus_ring(cx.builder, full, radius, a.focus_color)?;
        }
        if growing || rippling {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::{Action as A, Role};
        cx.node.set_role(Role::Tab);
        cx.node.set_selected(self.selected);
        if !cx.labelled {
            cx.node.set_label(self.label.text());
        }
        if cx.enabled {
            cx.node.add_action(A::Focus);
            cx.node.add_action(A::Click);
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
