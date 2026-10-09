//! Selection controls: checkbox, radio button and switch. One control holds
//! the toggle behavior, an optional label beside the mark, and the mark's
//! animations; each mark paints itself in its own module.
//!
//! Skin slots: `foreground` is the label, `indicator` the selected container
//! (checkbox box, radio ring and dot, switch track), `border_color` the
//! unselected outline, `background` the state layer around the mark, and
//! `caret` the mark on the container (check, switch handle).

mod checkbox;
mod kinds;
mod methods;
mod radio;
mod switch;

use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    Appearance, ControlKind, Key, KeyInput, NodeId, Point, Result, Size, State, Theme,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    scene::{Affine, Rect, RoundedRect},
    text_style,
};

pub use checkbox::Checkbox;
pub use kinds::{CHECKBOX, CHECKBOX_ERROR, RADIO_BUTTON, SWITCH};
pub(crate) use methods::{add, selection_methods};
pub use radio::Radio;
pub use switch::Switch;

use crate::{
    Icon, Scheme,
    anim::Value,
    skin,
    tokens::{motion, typescale},
};

/// Which mark a selection control shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    Checkbox,
    Radio,
    Switch,
}

/// The side of the 48 dp touch target around a checkbox or radio mark.
const TARGET: f32 = 48.0;
/// The state layer around a mark.
const LAYER: f32 = 40.0;

/// The control inside [`Checkbox`], [`Radio`] and [`Switch`].
pub struct SelectionControl {
    pub(crate) mark: Mark,
    toggle: aegle_controls::Toggle,
    /// A checkbox in the indeterminate state, shown as a dash.
    pub(crate) mixed: bool,
    /// A checkbox reporting an error, in the error color.
    pub(crate) error: bool,
    /// Icons on a switch's handle, selected and unselected.
    pub(crate) icons: [Option<Icon>; 2],
    label: Paragraph,
    /// How selected the mark looks, 0 to 1, and its pressed swell.
    on: Value,
    press: Value,
}

impl SelectionControl {
    fn new(
        fonts: &std::cell::RefCell<TextSystem>,
        theme: &Theme,
        mark: Mark,
        text: &str,
        checked: bool,
    ) -> Result<Self> {
        let mut style = text_style(theme);
        typescale::BODY_LARGE.apply(&mut style);
        Ok(Self {
            mark,
            toggle: aegle_controls::Toggle::new(checked),
            mixed: false,
            error: false,
            icons: [None, None],
            label: fonts.borrow_mut().paragraph(text, &style)?,
            on: Value::default(),
            press: Value::default(),
        })
    }

    /// Whether it is checked, selected or on.
    pub fn is_checked(&self) -> bool {
        self.toggle.is_checked()
    }

    /// The label.
    pub fn text(&self) -> &str {
        self.label.text()
    }

    pub(crate) fn set_checked(&mut self, checked: bool) -> Outcome {
        self.mixed = false;
        self.toggle.set_checked(checked)
    }

    /// The size of the mark's box: a touch target, or the switch's track.
    fn mark_size(&self) -> Size {
        match self.mark {
            Mark::Switch => Size::new(52.0, 32.0),
            _ => Size::new(TARGET, TARGET),
        }
    }
}

/// Deselects the other radio buttons among `id`'s siblings.
fn select_radio(state: &mut State, id: NodeId) -> Result {
    let Some(parent) = state.tree.parent(id)? else {
        return Ok(());
    };
    let siblings: Vec<_> = state.tree.children(parent)?.filter(|&n| n != id).collect();
    for sibling in siblings {
        if let Some(radio) = state.control_as::<SelectionControl>(sibling)
            && radio.mark == Mark::Radio
            && radio.is_checked()
        {
            let outcome = radio.set_checked(false);
            state.effects(sibling, outcome)?;
        }
    }
    Ok(())
}

/// Arrow keys move the choice among sibling radio buttons, wrapping; Left
/// moves forward right to left.
pub(crate) fn radio_key(state: &mut State, key: &KeyInput<'_>) -> Result<bool> {
    let Some(id) = state.focus.current(&state.tree) else {
        return Ok(false);
    };
    let forward = match key.key {
        Key::Down => true,
        Key::Up => false,
        Key::Right => !state.rtl(id),
        Key::Left => state.rtl(id),
        _ => return Ok(false),
    };
    let radio = |state: &mut State, node| {
        state
            .control_as::<SelectionControl>(node)
            .is_some_and(|c| c.mark == Mark::Radio)
            && state.usable(node)
    };
    if !radio(state, id) {
        return Ok(false);
    }
    let parent = state.tree.parent(id)?.expect("a radio button has a parent");
    let children: Vec<_> = state.tree.children(parent)?.collect();
    let group: Vec<_> = children.into_iter().filter(|&n| radio(state, n)).collect();
    let index = group.iter().position(|&n| n == id).expect("in its group");
    let next = if forward {
        (index + 1) % group.len()
    } else {
        (index + group.len() - 1) % group.len()
    };
    state.set_focus(Some(group[next]))?;
    state.dispatch(group[next], Input::Activate)?;
    Ok(true)
}

impl Control for SelectionControl {
    fn kind(&self) -> &'static ControlKind {
        match self.mark {
            Mark::Checkbox if self.error => &CHECKBOX_ERROR,
            Mark::Checkbox => &CHECKBOX,
            Mark::Radio => &RADIO_BUTTON,
            Mark::Switch => &SWITCH,
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
        typescale::BODY_LARGE.apply(style);
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            pressed: self.toggle.is_pressed(),
            hovered: Some(self.toggle.is_hovered()),
            checked: self.toggle.is_checked() && !self.mixed,
            ..Default::default()
        }
    }
    fn set_enabled(&mut self, _: &mut TextSystem, enabled: bool) -> Outcome {
        self.toggle.set_enabled(enabled)
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let was = self.toggle.is_checked();
        let mut outcome = self.toggle.handle(input);
        if outcome.action == Some(Action::Change) {
            if self.mark == Mark::Radio {
                // Choosing the chosen radio button changes nothing.
                self.toggle.set_checked(true);
                outcome.action = (!was).then_some(Action::Change);
                if !was {
                    cx.deferred.push(Box::new(select_radio));
                }
            } else if std::mem::take(&mut self.mixed) {
                self.toggle.set_checked(true);
            }
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
        let mark = self.mark_size();
        let label = self.label.size();
        if self.label.text().is_empty() {
            return Ok(Size::new(mark.width, mark.height.max(TARGET)));
        }
        // A switch's label sits at its start, a box's or radio's after it.
        let gap = if self.mark == Mark::Switch { 16.0 } else { 4.0 };
        Ok(Size::new(
            mark.width + gap + label.width,
            TARGET.max(label.height),
        ))
    }
    fn baseline(&self, size: Size, _: f32) -> Option<f32> {
        Some((size.height - self.label.size().height) / 2.0 + self.label.first_baseline()?)
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let a = *cx.appearance;
        let mark = self.mark_size();
        let label = self.label.size();
        let labelled = !self.label.text().is_empty();
        // Switches put the label first; checkboxes and radios after the mark.
        let mark_first = self.mark != Mark::Switch || !labelled;
        let start = if mark_first {
            0.0
        } else {
            size.width - mark.width
        };
        let mark_x = if cx.rtl {
            size.width - start - mark.width
        } else {
            start
        };
        let origin = Point::new(mark_x, (size.height - mark.height) / 2.0);
        if labelled {
            let x = if mark_first { mark.width + 4.0 } else { 0.0 };
            let x = if cx.rtl {
                size.width - x - label.width
            } else {
                x
            };
            let y = (size.height - label.height) / 2.0;
            cx.builder.push_transform(Affine::translation(x, y)?)?;
            self.label.paint_with_color(cx.builder, a.foreground)?;
            cx.builder.pop()?;
        }

        let selected = self.toggle.is_checked() || self.mixed;
        let spring = match self.mark {
            Mark::Checkbox => motion::FAST_EFFECTS,
            _ => motion::FAST_SPATIAL,
        };
        let (on, moving) = self
            .on
            .at(if selected { 1.0 } else { 0.0 }, spring, now, reduced);
        let pressed = if cx.visual.pressed { 1.0 } else { 0.0 };
        let (press, swelling) = self.press.at(pressed, motion::FAST_SPATIAL, now, reduced);
        let scheme = Scheme::of(cx.theme);
        let mut paint = MarkPaint {
            builder: cx.builder,
            origin,
            size: mark,
            appearance: a,
            scheme: &scheme,
            enabled: cx.visual.enabled,
            on,
            press,
            rtl: cx.rtl,
        };
        let center = match self.mark {
            Mark::Checkbox => checkbox::paint(&mut paint, self.mixed)?,
            Mark::Radio => radio::paint(&mut paint)?,
            Mark::Switch => switch::paint(&mut paint, &self.icons, selected)?,
        };
        if cx.visual.focused && cx.visual.enabled {
            let ring = Rect::new(center.x - LAYER / 2.0, center.y - LAYER / 2.0, LAYER, LAYER);
            if self.mark == Mark::Switch {
                let track = Rect::new(origin.x, origin.y, mark.width, mark.height);
                skin::focus_ring(cx.builder, track, mark.height / 2.0, a.focus_color)?;
            } else {
                skin::focus_ring(cx.builder, ring, LAYER / 2.0, a.focus_color)?;
            }
        }
        if moving || swelling {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::{Action, Role, Toggled};
        cx.node.set_role(match self.mark {
            Mark::Checkbox => Role::CheckBox,
            Mark::Radio => Role::RadioButton,
            Mark::Switch => Role::Switch,
        });
        cx.node.set_toggled(match (self.mixed, self.is_checked()) {
            (true, _) => Toggled::Mixed,
            (_, true) => Toggled::True,
            _ => Toggled::False,
        });
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

/// What a mark painter needs.
pub(crate) struct MarkPaint<'a> {
    pub builder: &'a mut aegle_ui::scene::SceneBuilder,
    /// Top left of the mark's box.
    pub origin: Point,
    pub size: Size,
    pub appearance: Appearance,
    pub scheme: &'a Scheme,
    pub enabled: bool,
    /// How selected it looks, 0 to 1 (overshooting with springs).
    pub on: f32,
    /// How pressed it looks, 0 to 1.
    pub press: f32,
    pub rtl: bool,
}

impl MarkPaint<'_> {
    /// Fills the state layer circle around `center`.
    pub fn layer(&mut self, center: Point) -> Result {
        let rect = Rect::new(center.x - LAYER / 2.0, center.y - LAYER / 2.0, LAYER, LAYER);
        self.builder.fill(
            RoundedRect::new(rect, LAYER / 2.0)?,
            self.appearance.background,
        )?;
        Ok(())
    }
}
