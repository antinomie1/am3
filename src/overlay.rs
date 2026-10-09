//! Layers above the window content: modal dialogs, sheets and drawers over a
//! scrim, and snackbars. A layer is a full-window (or bottom strip) host
//! under the root that places one surface; showing moves it above every
//! other child of the root. While a modal layer is on top, input below it is
//! blocked, Tab stays inside it, and Escape or a click on the scrim asks it
//! to close. Dismissal goes through the host's action, so the owning handle
//! closes it with its own animation and callbacks.

use std::time::{Duration, Instant};

use aegle_layout::{Edges, LengthPercentageAuto, Position, Style};
use aegle_ui::{
    Appearance, Container, ControlKind, Node, NodeId, Point, Rect, Result, State, Theme,
    VisualState,
    control::{Action, Control, Input, InputCx, Outcome, PaintCx},
    scene::RoundedRect,
};

use crate::{Scheme, color::alpha};

/// The open layers, bottom to top.
#[derive(Default)]
struct Layers {
    open: Vec<Open>,
}

struct Open {
    host: NodeId,
    surface: NodeId,
    restore: Option<NodeId>,
    modal: bool,
    /// When a snackbar closes on its own.
    until: Option<Instant>,
}

/// The control of a layer's host: the scrim of a modal layer, or nothing.
pub(crate) struct Scrim {
    modal: bool,
    open: bool,
    /// A press started outside the surface, so its release may dismiss.
    outside: bool,
}

impl Control for Scrim {
    fn kind(&self) -> &'static ControlKind {
        &SCRIM
    }
    fn interactive(&self) -> bool {
        self.modal && self.open
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let Input::Pointer(pointer) = input else {
            return Ok(Outcome::default());
        };
        let down = match pointer.kind {
            aegle_ui::PointerKind::Down { .. } => true,
            aegle_ui::PointerKind::Up => false,
            _ => return Ok(Outcome::default()),
        };
        let at = pointer.position;
        cx.deferred.push(Box::new(move |state, host| {
            let origin = state.tree.get(host).unwrap().context.bounds.origin;
            let at = Point::new(origin.x + at.x, origin.y + at.y);
            let surface = layers(state)
                .open
                .iter()
                .find(|o| o.host == host)
                .map(|o| o.surface);
            let outside = !surface.is_some_and(|s| bounds(state, s).contains(at));
            let scrim = state.control_as::<Scrim>(host).expect("a layer host");
            if down {
                scrim.outside = outside;
            } else if std::mem::take(&mut scrim.outside) && outside {
                dismiss(state, host)?;
            }
            Ok(())
        }));
        Ok(Outcome {
            handled: true,
            ..Outcome::default()
        })
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        if self.modal {
            let rect = Rect::new(0.0, 0.0, cx.size.width, cx.size.height);
            cx.builder
                .fill(RoundedRect::new(rect, 0.0)?, cx.appearance.background)?;
        }
        Ok(())
    }
}

fn scrim_skin(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        background: alpha(Scheme::of(theme).scrim, 0.32),
        ..Appearance::base(theme, state)
    }
}

/// The scrim behind modal surfaces.
pub static SCRIM: ControlKind = ControlKind {
    name: "Scrim",
    skin: scrim_skin,
    accepts: aegle_ui::Accepts::NONE,
    container: true,
};

fn layers(state: &mut State) -> &mut Layers {
    state.ext()
}

fn bounds(state: &State, id: NodeId) -> Rect {
    state.tree.get(id).unwrap().context.bounds
}

/// Where a layer places its surface in the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Place {
    Center,
    Bottom,
    Start,
    End,
}

/// A hidden layer and the surface it places; the owning handle shows and
/// closes it.
#[derive(Clone)]
pub(crate) struct Layer {
    pub host: Container,
    pub modal: bool,
}

impl Layer {
    /// A hidden host under the root, in the theme of `owner`, placing its
    /// child at `place`. A modal host covers the window with a scrim; the
    /// other is a strip along the bottom that lets input through.
    pub fn new(owner: &Node, place: Place, modal: bool, padding: f32) -> Result<Self> {
        let host = owner.change(|state, owner| {
            state.install(&crate::HOOKS);
            let auto = LengthPercentageAuto::auto();
            let zero = LengthPercentageAuto::length(0.0);
            let p = aegle_layout::LengthPercentage::length(padding);
            let (justify, align) = match place {
                Place::Center => (aegle_ui::Justify::Center, aegle_ui::Align::Center),
                Place::Bottom => (aegle_ui::Justify::End, aegle_ui::Align::Center),
                Place::Start => (aegle_ui::Justify::Start, aegle_ui::Align::Start),
                Place::End => (aegle_ui::Justify::Start, aegle_ui::Align::End),
            };
            let style = Style {
                position: Position::Absolute,
                inset: Edges {
                    left: zero,
                    right: zero,
                    top: if modal { zero } else { auto },
                    bottom: zero,
                },
                flex_direction: aegle_layout::FlexDirection::Column,
                justify_content: Some(justify.content()),
                align_items: Some(align.items()),
                padding: Edges {
                    left: p,
                    right: p,
                    top: p,
                    bottom: p,
                },
                ..Default::default()
            };
            let scrim = Scrim {
                modal,
                open: false,
                outside: false,
            };
            let root = state.root;
            let id = state.insert(root, usize::MAX, Box::new(scrim), style)?;
            state.set_visible(id, false)?;
            let local = state.tree.get(owner).unwrap().context.theme.clone();
            if local.is_some() {
                state.propagate_theme(id, local)?;
            }
            Ok(id)
        })?;
        let layer = Self {
            host: Container(Node {
                state: owner.state.clone(),
                id: host,
            }),
            modal,
        };
        #[cfg(feature = "motion")]
        {
            let hiding = layer.clone();
            layer.host.on_transition_end(move |_| hiding.hide())?;
        }
        Ok(layer)
    }

    /// Opens the layer with `surface` coming in `how`; `timeout` closes it
    /// on its own.
    pub fn show(&self, surface: &Container, timeout: Option<Duration>, how: Enter) -> Result {
        if self.open(surface, timeout)? {
            #[cfg(feature = "motion")]
            enter(&self.host, surface, how)?;
            #[cfg(not(feature = "motion"))]
            let _ = how;
        }
        Ok(())
    }

    /// Closes the layer, fading it out.
    pub fn dismiss(&self) -> Result {
        if self.close()? {
            #[cfg(feature = "motion")]
            return exit(&self.host);
            #[cfg(not(feature = "motion"))]
            return self.hide();
        }
        Ok(())
    }

    /// Runs `close` when Escape, the scrim or a timeout dismisses the layer.
    pub fn on_dismiss(&self, mut close: impl FnMut() -> Result + 'static) -> Result {
        self.host
            .change(|state, id| state.on_action(id, move |_| close()))
    }

    /// Whether it is shown and not closing.
    pub fn is_open(&self) -> Result<bool> {
        self.host.change(|state, id| {
            Ok(state
                .ext_ref::<Layers>()
                .is_some_and(|l| l.open.iter().any(|o| o.host == id)))
        })
    }

    /// Shows the layer above everything else with `surface` as its content,
    /// focusing the surface's first control if modal; `timeout` closes it on
    /// its own. Returns false if it was already open.
    fn open(&self, surface: &Node, timeout: Option<Duration>) -> Result<bool> {
        if self.is_open()? {
            return Ok(false);
        }
        let surface = surface.id;
        let modal = self.modal;
        self.host.change(|state, host| {
            let root = state.root;
            state.tree.reparent(host, Some(root))?;
            state.invalidate_structure();
            state.set_visible(host, true)?;
            state.control_as::<Scrim>(host).expect("a layer host").open = true;
            let restore = state.focus.current(&state.tree);
            let until = timeout.map(|t| Instant::now() + t);
            if let Some(due) = until {
                state.wake = Some(state.wake.map_or(due, |wake| wake.min(due)));
            }
            layers(state).open.push(Open {
                host,
                surface,
                restore,
                modal,
                until,
            });
            if modal {
                let first = controls(state, surface).first().copied();
                state.set_focus(first)?;
            }
            Ok(())
        })?;
        Ok(true)
    }

    /// Stops the layer blocking input and returns focus to where it was; the
    /// caller hides it, after an exit animation if any. Returns false if it
    /// was not open.
    fn close(&self) -> Result<bool> {
        self.host.change(|state, host| {
            let layers = layers(state);
            let Some(index) = layers.open.iter().position(|o| o.host == host) else {
                return Ok(false);
            };
            let open = layers.open.remove(index);
            state.control_as::<Scrim>(host).expect("a layer host").open = false;
            let inside = state
                .focus
                .current(&state.tree)
                .is_some_and(|f| state.contains(host, f));
            if inside {
                let restore = open.restore.filter(|&r| state.usable(r));
                state.set_focus(restore)?;
            }
            Ok(true)
        })
    }

    /// Hides the host if it has not been opened again.
    fn hide(&self) -> Result {
        if !self.is_open()? {
            self.host.set_visible(false)?;
        }
        Ok(())
    }
}

/// Asks the owner of a layer to close it.
fn dismiss(state: &mut State, host: NodeId) -> Result {
    let outcome = Outcome {
        action: Some(Action::Change),
        ..Outcome::default()
    };
    state.effects(host, outcome)
}

/// The usable interactive controls inside `surface`, in focus order.
fn controls(state: &mut State, surface: NodeId) -> Vec<NodeId> {
    state.rebuild_order();
    state
        .order
        .iter()
        .copied()
        .filter(|&n| {
            state.contains(surface, n)
                && state.tree.get(n).unwrap().context.control.interactive()
                && state.usable(n)
        })
        .collect()
}

/// The top modal layer, if no other root child shows above it.
fn top_modal(state: &State) -> Option<(NodeId, NodeId)> {
    let open = state.ext_ref::<Layers>()?.open.last()?;
    let root = state.root;
    let mut later = state
        .tree
        .children(root)
        .ok()?
        .skip_while(|&c| c != open.host)
        .skip(1);
    let covered = later.any(|c| state.tree.get(c).unwrap().context.effective_visible);
    (open.modal && !covered).then_some((open.host, open.surface))
}

/// Hook: a modal layer covers the window, except where a popup, tooltip or
/// snackbar shown after it lies.
pub(crate) fn overlay_at(state: &State, at: Point) -> Option<NodeId> {
    let open = state.ext_ref::<Layers>()?.open.last()?;
    if !open.modal {
        return None;
    }
    let root = state.root;
    let above = state
        .tree
        .children(root)
        .ok()?
        .skip_while(|&c| c != open.host)
        .skip(1)
        .any(|c| {
            let element = &state.tree.get(c).unwrap().context;
            element.effective_visible && element.bounds.contains(at)
        });
    (!above).then_some(open.host)
}

/// Hook: Escape dismisses the top modal layer and Tab cycles inside it.
pub(crate) fn key(state: &mut State, key: &aegle_ui::KeyInput<'_>) -> Result<bool> {
    if !key.pressed {
        return Ok(false);
    }
    let Some((host, surface)) = top_modal(state) else {
        return Ok(false);
    };
    match key.key {
        aegle_ui::Key::Escape => {
            dismiss(state, host)?;
            Ok(true)
        }
        aegle_ui::Key::Tab => {
            let controls = controls(state, surface);
            if controls.is_empty() {
                return Ok(true);
            }
            let current = state.focus.current(&state.tree);
            let index = controls.iter().position(|&n| Some(n) == current);
            let n = controls.len();
            let next = match (index, key.modifiers.shift) {
                (Some(i), false) => (i + 1) % n,
                (Some(i), true) => (i + n - 1) % n,
                (None, false) => 0,
                (None, true) => n - 1,
            };
            state.set_focus(Some(controls[next]))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Hook: dismisses layers whose time ran out and waits for the next.
pub(crate) fn wake(state: &mut State, now: Instant) -> Result {
    let Some(layers) = state.ext_ref::<Layers>() else {
        return Ok(());
    };
    let (due, waiting): (Vec<_>, Vec<_>) = layers
        .open
        .iter()
        .filter_map(|o| o.until.map(|until| (o.host, until)))
        .partition(|&(_, until)| until <= now);
    for (host, _) in due {
        for open in &mut self::layers(state).open {
            if open.host == host {
                open.until = None;
            }
        }
        dismiss(state, host)?;
    }
    if let Some(next) = waiting.iter().map(|&(_, until)| until).min() {
        state.wake = Some(state.wake.map_or(next, |wake| wake.min(next)));
    }
    Ok(())
}

/// Hook: forgets removed layers.
pub(crate) fn removed(state: &mut State, id: NodeId) {
    if state.ext_ref::<Layers>().is_some() {
        layers(state).open.retain(|o| o.host != id);
    }
}

/// How a surface comes in.
#[derive(Clone, Copy)]
pub(crate) enum Enter {
    /// Growing from 80 %.
    Scale,
    /// Rising or sliding in from an edge by this offset.
    Slide(aegle_ui::Point),
}

/// Fades a layer in with its surface entering on the expressive springs.
#[cfg(feature = "motion")]
fn enter(host: &Container, surface: &Container, how: Enter) -> Result {
    use aegle_motion::Animation;
    use aegle_ui::{Animate, Point};

    use crate::tokens::motion;
    let fade = Animation::tween(0.0, 1.0, motion::DEFAULT_EFFECTS.transition())?;
    host.animate(Animate::Opacity(fade))?;
    let spatial = motion::DEFAULT_SPATIAL.transition();
    match how {
        Enter::Scale => surface.animate(Animate::Scale(Animation::tween(0.8, 1.0, spatial)?)),
        Enter::Slide(from) => {
            let slide = Animation::tween(from, Point::new(0.0, 0.0), spatial)?;
            surface.animate(Animate::Offset(slide))
        }
    }
}

/// Fades a layer out; its end transition hides it.
#[cfg(feature = "motion")]
fn exit(host: &Container) -> Result {
    use aegle_motion::Animation;
    use aegle_ui::Animate;
    let fade = Animation::tween(1.0, 0.0, crate::tokens::motion::FAST_EFFECTS.transition())?;
    host.animate(Animate::Opacity(fade))
}
