//! Tabs: primary tabs (title-small labels, optional icons above them, a
//! 3 dp indicator under the content in primary) and secondary tabs (a 2 dp
//! indicator across the whole tab). The indicator slides to the selected
//! tab on the expressive spatial spring; a 1 dp divider runs under the row.

use std::cell::RefCell;

use aegle_layout::Style;
use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    Appearance, Color, Container, ControlKind, NodeId, Point, Rect, Result, Size, State, Theme,
    VisualState,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
    handle,
    scene::{Affine, RoundedRect},
    text_style,
};

mod row;

use row::Rows;
pub use row::{TAB_ROW, TabRowControl};
pub(crate) use row::{place, removed};

use crate::{
    Icon, Scheme,
    anim::Ripple,
    color::alpha,
    skin::{self, layer_opacity},
    tokens::{state, typescale},
};

/// Primary or secondary tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabStyle {
    /// Top-level content sections, below an app bar.
    #[default]
    Primary,
    /// Sections within a content area.
    Secondary,
}

/// The control inside a [`Tab`].
pub struct TabControl {
    icon: Option<Icon>,
    label: Paragraph,
    selected: bool,
    secondary: bool,
    button: aegle_controls::Button,
    ripple: Option<Ripple>,
    pressed_at: Option<Point>,
    released: bool,
}

impl TabControl {
    /// The width of its content, which the primary indicator spans.
    fn content_width(&self) -> f32 {
        let icon = if self.icon.is_some() { 24.0 } else { 0.0 };
        self.label.size().width.max(icon).max(24.0)
    }
}

impl Control for TabControl {
    fn kind(&self) -> &'static ControlKind {
        if self.secondary { &SECONDARY_TAB } else { &TAB }
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
        typescale::TITLE_SMALL.apply(style);
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
            cx.deferred.push(Box::new(select));
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
        let height = if self.icon.is_some() && !self.secondary {
            64.0
        } else {
            48.0
        };
        Ok(Size::new(self.content_width() + 32.0, height))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let a = *cx.appearance;
        let rect = Rect::new(0.0, 0.0, size.width, size.height);
        let shape = RoundedRect::new(rect, 0.0)?;
        if a.background.to_rgba()[3] > 0 {
            cx.builder.fill(shape, a.background)?;
        }
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
        let label = self.label.size();
        let stacked = self.icon.is_some() && !self.secondary;
        let content = if stacked {
            24.0 + 2.0 + label.height
        } else {
            label.height
        };
        let mut y = (size.height - content) / 2.0;
        if let (true, Some(icon)) = (stacked, &self.icon) {
            icon.paint(
                cx.builder,
                Point::new((size.width - 24.0) / 2.0, y),
                24.0,
                a.indicator,
            )?;
            y += 26.0;
        }
        cx.builder
            .push_transform(Affine::translation((size.width - label.width) / 2.0, y)?)?;
        self.label.paint_with_color(cx.builder, a.foreground)?;
        cx.builder.pop()?;
        if cx.visual.focused && cx.visual.enabled {
            let inset = Rect::new(4.0, 4.0, size.width - 8.0, size.height - 8.0);
            skin::focus_ring(cx.builder, inset, 8.0, a.focus_color)?;
        }
        if rippling {
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

fn tab(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    let on = state.checked;
    let content = if !state.enabled {
        alpha(s.on_surface, state::DISABLED_CONTENT)
    } else if on {
        s.primary
    } else {
        s.on_surface_variant
    };
    let opacity = layer_opacity(state);
    Appearance {
        background: if opacity > 0.0 {
            alpha(if on { s.primary } else { s.on_surface }, opacity)
        } else {
            Color::TRANSPARENT
        },
        foreground: content,
        indicator: content,
        focus_color: s.secondary,
        focus_width: 0.0,
        ..Appearance::base(theme, state)
    }
}

fn secondary_tab(theme: &Theme, state: VisualState) -> Appearance {
    let on_surface = Scheme::of(theme).on_surface;
    let base = tab(theme, state);
    if state.checked && state.enabled {
        Appearance {
            foreground: on_surface,
            indicator: on_surface,
            ..base
        }
    } else {
        base
    }
}

/// A primary tab.
pub static TAB: ControlKind = ControlKind {
    name: "Tab",
    skin: tab,
    accepts: skin::PRESSABLE,
    container: false,
};
/// A secondary tab.
pub static SECONDARY_TAB: ControlKind = ControlKind {
    name: "SecondaryTab",
    skin: secondary_tab,
    accepts: skin::PRESSABLE,
    container: false,
};

/// Deselects the other tabs of a newly selected tab and moves the indicator.
fn select(state: &mut State, id: NodeId) -> Result {
    let Some(row) = state.tree.parent(id)? else {
        return Ok(());
    };
    let tabs: Vec<_> = state.tree.children(row)?.filter(|&n| n != id).collect();
    for other in tabs {
        if let Some(c) = state.control_as::<TabControl>(other)
            && c.selected
        {
            c.selected = false;
            let dirty = aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS;
            state.tree.mark_dirty(other, dirty)?;
        }
    }
    // The next geometry pass places the indicator.
    state.geometry_dirty = true;
    Ok(())
}

/// A row of tabs, exactly one selected.
#[derive(Clone)]
pub struct Tabs {
    row: Container,
    style: TabStyle,
}

impl std::ops::Deref for Tabs {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.row
    }
}

handle! {
    /// A tab of a [`Tabs`] row.
    pub Tab(TabControl): text, interactive, pressed, indicator
}

impl Tabs {
    /// Appends an empty row of fixed tabs, which share its width equally.
    pub fn new(parent: &Container, style: TabStyle) -> Result<Self> {
        let node = parent.add(|state, _| {
            state.install(&crate::HOOKS);
            let control = TabRowControl {
                style,
                target: None,
                at: Default::default(),
            };
            let layout = Style {
                flex_direction: aegle_layout::FlexDirection::Row,
                flex_shrink: 0.0,
                align_self: Some(aegle_ui::Align::Stretch.items()),
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn Control>, layout))
        })?;
        node.change(|state, id| {
            state.ext::<Rows>().0.push(id);
            Ok(())
        })?;
        Ok(Self {
            row: Container(node),
            style,
        })
    }

    /// Appends a tab, with `icon` above the label of a primary tab; the
    /// first one is selected.
    pub fn tab(&self, icon: Option<Icon>, text: &str) -> Result<Tab> {
        let secondary = self.style == TabStyle::Secondary;
        let first = self.selected()?.is_none();
        let node = self.row.add(|state, theme| {
            let control = new_tab(&state.fonts, theme, icon, text, secondary, first)?;
            let layout = Style {
                flex_grow: 1.0,
                flex_basis: aegle_layout::Dimension::length(0.0),
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn Control>, layout))
        })?;
        crate::effects(&node)?;
        Ok(Tab(node))
    }

    /// The index of the selected tab.
    pub fn selected(&self) -> Result<Option<usize>> {
        self.row.change(|state, id| {
            let children: Vec<_> = state.tree.children(id)?.collect();
            Ok(children.into_iter().position(|c| {
                state
                    .control_as::<TabControl>(c)
                    .is_some_and(|tab| tab.selected)
            }))
        })
    }
}

fn new_tab(
    fonts: &RefCell<TextSystem>,
    theme: &Theme,
    icon: Option<Icon>,
    text: &str,
    secondary: bool,
    selected: bool,
) -> Result<TabControl> {
    let mut style = text_style(theme);
    typescale::TITLE_SMALL.apply(&mut style);
    Ok(TabControl {
        icon,
        label: fonts.borrow_mut().paragraph(text, &style)?,
        selected,
        secondary,
        button: aegle_controls::Button::new(),
        ripple: None,
        pressed_at: None,
        released: false,
    })
}

impl Tab {
    /// Selects it, sliding the indicator over.
    pub fn select(&self) -> Result {
        self.change(|state, id| {
            state
                .control_as::<TabControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?
                .selected = true;
            state.dirty_visual_state(id)?;
            select(state, id)
        })
    }

    /// Whether it is selected.
    pub fn is_selected(&self) -> Result<bool> {
        self.read(|c| c.selected)
    }

    /// Adds a handler run after each activation.
    pub fn on_click(&self, mut callback: impl FnMut(Tab) -> Result + 'static) -> Result {
        self.change(|state, id| state.on_action(id, move |node| callback(Tab(node))))
    }
}
