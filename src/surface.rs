//! Surfaces: the containers cards, dialogs, sheets, menus, snackbars and
//! rich tooltips are made of. A surface paints its shadow, container and
//! outline with its own corner radii; a clickable one is also a button with
//! a state layer, a ripple and an elevation that rises on hover.

use aegle_layout::Style;
use aegle_text::TextSystem;
use aegle_ui::{
    Container, ControlKind, Point, Rect, Result,
    control::{Control, ControlVisual, Input, InputCx, Outcome, PaintCx},
    scene::{RoundedRect, Stroke},
};

use crate::{
    Scheme,
    anim::{Ripple, Value},
    pressable::shape::rounded,
    skin::{self, Paint, kinds},
    tokens::motion,
};

/// What a surface is, for assistive technology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    Card,
    Dialog,
    Sheet,
    Snackbar,
    Tooltip,
    Pane,
    List,
    ListItem,
}

/// The control inside every surface.
pub struct SurfaceControl {
    kind: &'static ControlKind,
    #[cfg_attr(not(feature = "accessibility"), allow(dead_code))]
    pub(crate) part: Part,
    /// Top-left, top-right, bottom-right and bottom-left radii, mirrored
    /// right to left.
    pub(crate) corners: [f32; 4],
    /// Elevation levels at rest, hovered and pressed.
    pub(crate) elevation: [f32; 3],
    button: Option<aegle_controls::Button>,
    lift: Value,
    ripple: Option<Ripple>,
    pressed_at: Option<Point>,
    released: bool,
}

impl SurfaceControl {
    pub(crate) fn new(kind: &'static ControlKind, part: Part, radius: f32, level: f32) -> Self {
        Self {
            kind,
            part,
            corners: [radius; 4],
            elevation: [level; 3],
            button: None,
            lift: Value::default(),
            ripple: None,
            pressed_at: None,
            released: false,
        }
    }

    /// Makes it clickable.
    pub(crate) fn clickable(mut self) -> Self {
        self.button = Some(aegle_controls::Button::new());
        self
    }

    fn level(&self, state: aegle_ui::VisualState) -> f32 {
        match () {
            _ if !state.enabled => 0.0,
            _ if state.pressed => self.elevation[2],
            _ if state.hovered => self.elevation[1],
            _ => self.elevation[0],
        }
    }
}

impl Control for SurfaceControl {
    fn kind(&self) -> &'static ControlKind {
        self.kind
    }
    fn interactive(&self) -> bool {
        self.button.is_some()
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn visual(&self) -> ControlVisual {
        match &self.button {
            Some(button) => ControlVisual {
                pressed: button.is_pressed(),
                hovered: Some(button.is_hovered()),
                ..Default::default()
            },
            None => ControlVisual::default(),
        }
    }
    fn set_enabled(&mut self, _: &mut TextSystem, enabled: bool) -> Outcome {
        self.button
            .as_mut()
            .map_or_else(Outcome::default, |b| b.set_enabled(enabled))
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let Some(button) = &mut self.button else {
            return Ok(Outcome::default());
        };
        let was = button.is_pressed();
        let outcome = button.handle(input);
        if button.is_pressed() && !was {
            self.pressed_at = Some(match input {
                Input::Pointer(pointer) => pointer.position,
                _ => Point::new(cx.size.width / 2.0, cx.size.height / 2.0),
            });
        } else if was && !button.is_pressed() {
            self.released = true;
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
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let a = *cx.appearance;
        let scheme = Scheme::of(cx.theme);
        let (level, lifting) =
            self.lift
                .at(self.level(cx.visual), motion::FAST_EFFECTS, now, reduced);
        let rect = Rect::new(0.0, 0.0, size.width, size.height);
        let [tl, tr, br, bl] = self.corners;
        let radii = if cx.rtl {
            [tr, tl, bl, br]
        } else {
            [tl, tr, br, bl]
        };
        let even = radii.iter().all(|&r| (r - radii[0]).abs() < 0.01);
        let largest = radii.iter().copied().fold(0.0, f32::max);
        let shape = RoundedRect::new(rect, largest.min(size.width.min(size.height) / 2.0))?;
        skin::shadow(cx.builder, shape, level, scheme.shadow)?;
        let path = (!even).then(|| rounded(rect, radii)).transpose()?;
        match &path {
            Some(path) => cx.builder.fill_path(path, a.background)?,
            None => cx.builder.fill(shape, a.background)?,
        };
        if let Some(at) = self.pressed_at.take() {
            self.ripple = Some(Ripple::new(at, now));
        }
        if std::mem::take(&mut self.released)
            && let Some(ripple) = &mut self.ripple
        {
            ripple.release(now);
        }
        let rippling = match &self.ripple {
            Some(ripple) if !reduced => {
                cx.builder.push_clip(shape)?;
                let visible = ripple.paint(cx.builder, size, a.foreground, now)?;
                cx.builder.pop()?;
                visible
            }
            _ => false,
        };
        if !rippling {
            self.ripple = None;
        }
        if a.border_width > 0.0 && a.border_color.to_rgba()[3] > 0 {
            let w = a.border_width;
            let inner = Rect::new(w / 2.0, w / 2.0, size.width - w, size.height - w);
            match &path {
                Some(_) => {
                    let edge = rounded(inner, radii.map(|r| (r - w / 2.0).max(0.0)))?;
                    cx.builder
                        .stroke_path(&edge, a.border_color, Stroke::new(w))?;
                }
                None => {
                    let edge = RoundedRect::new(inner, (shape.radius() - w / 2.0).max(0.0))?;
                    cx.builder.stroke(edge, a.border_color, w)?;
                }
            }
        }
        if self.button.is_some() && cx.visual.focused && cx.visual.enabled {
            skin::focus_ring(cx.builder, rect, shape.radius(), a.focus_color)?;
        }
        if lifting || rippling {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        use aegle_ui::accesskit::{Action as A, Role};
        cx.node.set_role(match (self.part, self.button.is_some()) {
            (_, true) => Role::Button,
            (Part::Dialog | Part::Sheet, _) => Role::Dialog,
            (Part::Snackbar, _) => Role::Status,
            (Part::Tooltip, _) => Role::Tooltip,
            (Part::List, _) => Role::List,
            (Part::ListItem, _) => Role::ListItem,
            _ => Role::Group,
        });
        if self.button.is_some() && cx.enabled {
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
        (self.button.is_some() && action == aegle_ui::accesskit::Action::Click)
            .then_some(Input::Activate)
    }
}

kinds! { container
    /// An elevated card.
    ELEVATED_CARD = "ElevatedCard", elevated => |s, _on| {
        Paint::new(s.surface_container_low, s.on_surface)
    };
    /// A filled card.
    FILLED_CARD = "FilledCard", filled => |s, _on| {
        Paint::new(s.surface_container_highest, s.on_surface)
    };
    /// An outlined card.
    OUTLINED_CARD = "OutlinedCard", outlined => |s, _on| {
        Paint::new(s.surface, s.on_surface).outlined(s.outline_variant)
    };
    /// A dialog's container.
    DIALOG = "Dialog", dialog => |s, _on| Paint::new(s.surface_container_high, s.on_surface);
    /// A modal bottom or side sheet, or a modal navigation drawer.
    SHEET = "Sheet", sheet => |s, _on| Paint::new(s.surface_container_low, s.on_surface);
    /// A standard side sheet beside the content.
    STANDARD_SHEET = "StandardSheet", standard_sheet => |s, _on| {
        Paint::new(s.surface, s.on_surface_variant)
    };
    /// A snackbar.
    SNACKBAR = "Snackbar", snackbar => |s, _on| {
        Paint::new(s.inverse_surface, s.inverse_on_surface)
    };
    /// A rich tooltip.
    RICH_TOOLTIP = "RichTooltip", rich_tooltip => |s, _on| {
        Paint::new(s.surface_container, s.on_surface_variant)
    };
}

/// Appends a surface column; `padding` insets its children.
pub(crate) fn add(parent: &Container, control: SurfaceControl, padding: f32) -> Result<Container> {
    let node = parent.add(move |_, _| {
        let p = aegle_layout::LengthPercentage::length(padding);
        let style = Style {
            flex_direction: aegle_layout::FlexDirection::Column,
            flex_shrink: 0.0,
            padding: aegle_layout::Edges {
                left: p,
                right: p,
                top: p,
                bottom: p,
            },
            ..Default::default()
        };
        Ok((Box::new(control) as Box<dyn Control>, style))
    })?;
    crate::effects(&node)?;
    Ok(Container(node))
}

/// Gives a surface another kind, as when its role changes with scrolling.
pub(crate) fn set_kind(surface: &Container, kind: &'static ControlKind) -> Result {
    surface.change(|state, id| {
        state
            .control_as::<SurfaceControl>(id)
            .ok_or(aegle_ui::UiError::WrongKind)?
            .kind = kind;
        state.dirty_visual_state(id)
    })
}

/// The three card styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardStyle {
    /// On a low container with a shadow, separated from a busy background.
    #[default]
    Elevated,
    /// On the highest container, the subtlest separation.
    Filled,
    /// On the surface inside an outline.
    Outlined,
}

/// A card: a 12 dp rounded container of related content and actions.
#[derive(Clone)]
pub struct Card(Container);

impl std::ops::Deref for Card {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.0
    }
}

impl Card {
    /// Appends an empty card with 16 dp padding and 8 dp gaps; add its
    /// content through the container methods.
    pub fn new(parent: &Container, style: CardStyle) -> Result<Self> {
        Self::create(parent, style, false)
    }

    /// Appends a card that is itself a button: hovering raises it and
    /// activating it runs [`Card::on_click`].
    pub fn clickable(parent: &Container, style: CardStyle) -> Result<Self> {
        Self::create(parent, style, true)
    }

    fn create(parent: &Container, style: CardStyle, clickable: bool) -> Result<Self> {
        let (kind, levels): (&'static ControlKind, _) = match style {
            CardStyle::Elevated => (&ELEVATED_CARD, [1.0, 2.0, 1.0]),
            CardStyle::Filled => (&FILLED_CARD, [0.0, 1.0, 0.0]),
            CardStyle::Outlined => (&OUTLINED_CARD, [0.0, 1.0, 0.0]),
        };
        let mut control = SurfaceControl::new(kind, Part::Card, 12.0, levels[0]);
        if clickable {
            control = control.clickable();
            control.elevation = levels;
        }
        let card = add(parent, control, 16.0)?;
        card.set_gap(8.0)?;
        Ok(Self(card))
    }

    /// Adds a handler run after each activation of a clickable card.
    pub fn on_click(&self, mut callback: impl FnMut(Card) -> Result + 'static) -> Result {
        self.change(|state, id| state.on_action(id, move |node| callback(Card(Container(node)))))
    }

    /// Activates a clickable card through the normal enabled checks.
    pub fn activate(&self) -> Result {
        self.change(|state, id| state.dispatch(id, Input::Activate))
    }
}
