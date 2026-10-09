//! Text fields: filled (a surface-container-highest box with an active
//! indicator) and outlined (an outline that opens for the label), single
//! line or multiline, on Aegle's editor (selection, IME, clipboard, undo
//! and accessibility come from it). The label rests in the field and
//! floats above the text while focused or filled; supporting or error text
//! and a character counter sit under it, and optional leading and trailing
//! icons beside the text.

mod handle;
mod paint;

use std::cell::RefCell;

use aegle_text::{Paragraph, TextStyle, TextSystem};
use aegle_ui::{
    Appearance, Color, ControlKind, Point, Result, Size, Theme, VisualState,
    control::{Action, Control, ControlVisual, Input, InputCx, MeasureCx, Outcome, PaintCx},
};

pub use handle::{FieldStyle, TextField};

use crate::{
    Icon, Scheme,
    anim::Value,
    color::{alpha, blend},
    tokens::{TypeStyle, state, typescale},
};

/// The control inside a [`TextField`].
pub struct FieldControl {
    pub(crate) style: FieldStyle,
    pub(crate) field: Box<aegle_controls::TextField>,
    pub(crate) label: Option<Paragraph>,
    pub(crate) placeholder: Option<Paragraph>,
    pub(crate) supporting: Option<Paragraph>,
    pub(crate) counter: Option<(usize, Paragraph)>,
    pub(crate) leading: Option<Icon>,
    pub(crate) trailing: Option<Icon>,
    /// The trailing icon clears the text.
    pub(crate) clearable: bool,
    pub(crate) error: bool,
    float: Value,
}

/// The body-small style beside a body-large one.
pub(crate) fn small<'a>(style: &TextStyle<'a>) -> TextStyle<'a> {
    let mut small = style.clone();
    small.size = style.size * 14.0 / 16.0;
    typescale::BODY_SMALL.apply(&mut small);
    small
}

const ROLE: TypeStyle = typescale::BODY_LARGE;

impl FieldControl {
    pub(crate) fn new(
        fonts: &RefCell<TextSystem>,
        style: &TextStyle<'_>,
        kind: FieldStyle,
        text: &str,
        multiline: bool,
    ) -> Result<Self> {
        let editor = fonts.borrow_mut().editor(
            text,
            style,
            aegle_text::EditorOptions {
                multiline,
                ..Default::default()
            },
        )?;
        Ok(Self {
            style: kind,
            field: Box::new(aegle_controls::TextField::new(editor)),
            label: None,
            placeholder: None,
            supporting: None,
            counter: None,
            leading: None,
            trailing: None,
            clearable: false,
            error: false,
            float: Value::default(),
        })
    }

    fn filled(&self) -> bool {
        self.style == FieldStyle::Filled
    }

    /// The start of the text, after the leading icon.
    fn text_x(&self) -> f32 {
        if self.leading.is_some() { 52.0 } else { 16.0 }
    }

    /// The space after the text, before the trailing icon.
    fn text_end(&self) -> f32 {
        if self.trailing.is_some() { 52.0 } else { 16.0 }
    }

    /// The top of the text: under a floating label in a filled field.
    fn text_y(&self) -> f32 {
        if self.filled() && self.label.is_some() {
            24.0
        } else {
            16.0
        }
    }

    /// The height under the container for supporting text or a counter.
    fn below(&self) -> f32 {
        if self.supporting.is_some() || self.counter.is_some() {
            20.0
        } else {
            0.0
        }
    }

    fn container_height(&self) -> f32 {
        // 16 dp above and below, or 24 above and 8 below a filled label.
        (self.field.editor().size().height.max(24.0) + 32.0).max(56.0)
    }

    fn empty(&self) -> bool {
        self.field.editor().display_text().is_empty()
    }

    fn reflow(
        &mut self,
        fonts: &RefCell<TextSystem>,
        width: Option<f32>,
        cx: &MeasureCx<'_>,
    ) -> Result {
        let width = width.map(|w| (w - self.text_x() - self.text_end()).max(0.0));
        fonts
            .borrow_mut()
            .edit(self.field.editor_mut())
            .reflow(width, cx.alignment())?;
        Ok(())
    }
}

impl Control for FieldControl {
    fn kind(&self) -> &'static ControlKind {
        match (self.style, self.error) {
            (FieldStyle::Filled, false) => &FILLED_TEXT_FIELD,
            (FieldStyle::Filled, true) => &FILLED_TEXT_FIELD_ERROR,
            (FieldStyle::Outlined, false) => &OUTLINED_TEXT_FIELD,
            (FieldStyle::Outlined, true) => &OUTLINED_TEXT_FIELD_ERROR,
            (FieldStyle::Search, _) => &SEARCH_BAR,
        }
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
    fn default_padding(&self, _: &Theme) -> f32 {
        0.0
    }
    fn editor(&self) -> Option<&aegle_controls::TextField> {
        Some(&self.field)
    }
    fn editor_mut(&mut self) -> Option<&mut aegle_controls::TextField> {
        Some(&mut self.field)
    }
    fn text_role(&self, style: &mut TextStyle<'_>) {
        ROLE.apply(style);
    }
    fn restyle(&mut self, fonts: &mut TextSystem, style: &TextStyle<'_>) -> Result {
        fonts.edit(self.field.editor_mut()).restyle(style)?;
        for text in [&mut self.label, &mut self.placeholder]
            .into_iter()
            .flatten()
        {
            fonts.restyle(text, style)?;
        }
        let small = small(style);
        if let Some(text) = &mut self.supporting {
            fonts.restyle(text, &small)?;
        }
        if let Some((_, text)) = &mut self.counter {
            fonts.restyle(text, &small)?;
        }
        Ok(())
    }
    fn visual(&self) -> ControlVisual {
        ControlVisual {
            read_only: self.field.editor().is_read_only(),
            ..Default::default()
        }
    }
    fn set_enabled(&mut self, fonts: &mut TextSystem, enabled: bool) -> Outcome {
        self.field.set_enabled(fonts, enabled)
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        if let Input::Pointer(p) = input
            && matches!(p.kind, aegle_ui::PointerKind::Down { .. })
            && self.clearable
            && !self.empty()
        {
            // Pointer positions arrive relative to the text origin.
            let x = p.position.x + self.text_x();
            let zone = if cx.rtl {
                x < self.text_end()
            } else {
                x > cx.size.width - self.text_end()
            };
            if zone {
                cx.deferred.push(Box::new(|state, id| {
                    state.set_text(id, "")?;
                    handle::update_counter(state, id)?;
                    let outcome = Outcome {
                        action: Some(Action::Change),
                        ..Outcome::default()
                    };
                    state.effects(id, outcome)
                }));
                return Ok(Outcome {
                    handled: true,
                    ..Outcome::default()
                });
            }
        }
        let before = self.field.editor().changes().value;
        let mut outcome = self.field.handle(cx.fonts, input)?;
        if !before && self.field.editor().changes().value {
            outcome.action.get_or_insert(Action::Change);
            outcome.repaint = true;
            if self.counter.is_some() {
                cx.deferred.push(Box::new(handle::update_counter));
            }
        }
        Ok(outcome)
    }
    fn content_offset(&self, _: Size, _: f32, scroll: Point) -> Point {
        Point::new(scroll.x - self.text_x(), scroll.y - self.text_y())
    }
    fn text_viewport(&self, size: Size, _: f32) -> Size {
        Size::new(
            (size.width - self.text_x() - self.text_end()).max(0.0),
            (self.container_height() - self.text_y() - 8.0).max(0.0),
        )
    }
    fn baseline(&self, _: Size, _: f32) -> Option<f32> {
        Some(self.text_y() + self.field.editor().first_baseline()?)
    }
    fn measure(&mut self, cx: &MeasureCx<'_>) -> Result<Size> {
        self.reflow(cx.fonts, Some(cx.width.unwrap_or(280.0)), cx)?;
        Ok(Size::new(280.0, self.container_height() + self.below()))
    }
    fn finalize(&mut self, cx: &MeasureCx<'_>) -> Result {
        self.reflow(cx.fonts, cx.width, cx)
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        paint::paint(self, cx)
    }
}

fn search_bar(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    let hovered = state.enabled && state.hovered && !state.focused;
    Appearance {
        background: if hovered {
            blend(s.surface_container_high, s.on_surface, state::HOVER)
        } else {
            s.surface_container_high
        },
        border_width: 0.0,
        ..colors(theme, state, true, false)
    }
}

/// A search bar.
pub static SEARCH_BAR: ControlKind = ControlKind {
    name: "SearchBar",
    skin: search_bar,
    accepts: aegle_ui::Accepts::TEXT
        .with(aegle_ui::Accepts::INTERACTIVE)
        .with(aegle_ui::Accepts::EDITOR),
    container: false,
};

/// The colors of a field: `background` is the filled container,
/// `border_color` the indicator or outline, `indicator` the label.
fn colors(theme: &Theme, state: VisualState, filled: bool, error: bool) -> Appearance {
    let s = Scheme::of(theme);
    let off = |opacity| alpha(s.on_surface, opacity);
    let accent = if error { s.error } else { s.primary };
    let (line, label) = if !state.enabled {
        (
            off(if filled { 0.38 } else { 0.12 }),
            off(state::DISABLED_CONTENT),
        )
    } else if state.focused {
        (accent, accent)
    } else if error {
        (
            if state.hovered {
                s.on_error_container
            } else {
                s.error
            },
            s.error,
        )
    } else if state.hovered {
        (s.on_surface, s.on_surface_variant)
    } else {
        (
            if filled {
                s.on_surface_variant
            } else {
                s.outline
            },
            s.on_surface_variant,
        )
    };
    let background = match (filled, state.enabled) {
        (false, _) => Color::TRANSPARENT,
        (true, false) => off(0.04),
        (true, true) if state.hovered && !state.focused => {
            blend(s.surface_container_highest, s.on_surface, state::HOVER)
        }
        (true, true) => s.surface_container_highest,
    };
    Appearance {
        background,
        foreground: if state.enabled {
            s.on_surface
        } else {
            off(state::DISABLED_CONTENT)
        },
        border_color: line,
        border_width: if state.focused { 2.0 } else { 1.0 },
        indicator: label,
        caret: accent,
        selection: alpha(s.primary, 0.4),
        focus_width: 0.0,
        radius: 4.0,
        ..Appearance::base(theme, state)
    }
}

macro_rules! fields {
    ($($(#[$doc:meta])* $kind:ident = $name:literal, $skin:ident, $filled:literal, $error:literal;)*) => {$(
        fn $skin(theme: &Theme, state: VisualState) -> Appearance {
            colors(theme, state, $filled, $error)
        }
        $(#[$doc])*
        pub static $kind: ControlKind = ControlKind {
            name: $name,
            skin: $skin,
            accepts: aegle_ui::Accepts::TEXT
                .with(aegle_ui::Accepts::INTERACTIVE)
                .with(aegle_ui::Accepts::EDITOR),
            container: false,
        };
    )*};
}

fields! {
    /// A filled text field.
    FILLED_TEXT_FIELD = "FilledTextField", filled, true, false;
    /// A filled text field showing an error.
    FILLED_TEXT_FIELD_ERROR = "FilledTextFieldError", filled_error, true, true;
    /// An outlined text field.
    OUTLINED_TEXT_FIELD = "OutlinedTextField", outlined, false, false;
    /// An outlined text field showing an error.
    OUTLINED_TEXT_FIELD_ERROR = "OutlinedTextFieldError", outlined_error, false, true;
}
