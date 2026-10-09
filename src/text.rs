//! Text in a role of the type scale and a color role of the scheme: the
//! headlines, supporting text and labels inside cards, dialogs, lists and
//! sheets. It wraps to the width its container gives it.

use aegle_layout::Style;
use aegle_text::{Paragraph, TextStyle};
use aegle_ui::{
    Appearance, Container, ControlKind, Result, Size, Theme, VisualState,
    control::{Control, MeasureCx, PaintCx},
    handle,
    scene::Affine,
    text_style,
};

use crate::{
    Role, Scheme,
    color::alpha,
    tokens::{TypeStyle, state},
};

/// The control inside a [`Text`].
pub struct TextControl {
    text: Paragraph,
    pub(crate) style: TypeStyle,
    pub(crate) color: Role,
}

impl Control for TextControl {
    fn kind(&self) -> &'static ControlKind {
        &TEXT
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
    fn paragraph(&self) -> Option<&Paragraph> {
        Some(&self.text)
    }
    fn paragraph_mut(&mut self) -> Option<&mut Paragraph> {
        Some(&mut self.text)
    }
    fn text_role(&self, style: &mut TextStyle<'_>) {
        self.style.apply(style);
    }
    fn baseline(&self, _: Size, padding: f32) -> Option<f32> {
        Some(padding + self.text.first_baseline()?)
    }
    fn measure(&mut self, cx: &MeasureCx<'_>) -> Result<Size> {
        let size = self.text.reflow(cx.content_width(), cx.alignment())?;
        Ok(Size::new(
            size.width + 2.0 * cx.padding,
            size.height + 2.0 * cx.padding,
        ))
    }
    fn finalize(&mut self, cx: &MeasureCx<'_>) -> Result {
        self.text.reflow(cx.content_width(), cx.alignment())?;
        Ok(())
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let mut color = Scheme::of(cx.theme).role(self.color);
        if !cx.visual.enabled {
            color = alpha(color, state::DISABLED_CONTENT);
        }
        cx.builder
            .push_transform(Affine::translation(cx.padding, cx.padding)?)?;
        self.text.paint_with_color(cx.builder, color)?;
        cx.builder.pop()?;
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        cx.node.set_role(aegle_ui::accesskit::Role::Label);
        cx.node.set_value(self.text.text());
    }
}

fn text_skin(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        foreground: Scheme::of(theme).on_surface,
        ..Appearance::base(theme, state)
    }
}

/// Text of the type scale.
pub static TEXT: ControlKind = ControlKind {
    name: "Text",
    skin: text_skin,
    accepts: aegle_ui::Accepts::TEXT,
    container: false,
};

handle! {
    /// Display text in a type role and a color role.
    pub Text(TextControl): text
}

impl Text {
    /// Appends `text` in the type role `style`, colored by `color`.
    pub fn new(parent: &Container, style: TypeStyle, color: Role, text: &str) -> Result<Self> {
        parent
            .add(|state, theme| {
                let mut text_style = text_style(theme);
                style.apply(&mut text_style);
                let control = TextControl {
                    text: state.fonts.borrow_mut().paragraph(text, &text_style)?,
                    style,
                    color,
                };
                let layout = Style {
                    flex_shrink: 0.0,
                    ..Default::default()
                };
                Ok((Box::new(control) as Box<dyn Control>, layout))
            })
            .map(Self)
    }

    /// Replaces the text.
    pub fn set_text(&self, text: &str) -> Result {
        self.change(|state, id| state.set_text(id, text))
    }

    /// The text.
    pub fn text(&self) -> Result<String> {
        self.change(|state, id| state.text(id))
    }

    /// Colors it with another role.
    pub fn set_color(&self, color: Role) -> Result {
        self.update(|c| c.color = color)
    }

    /// Changes its type role, reshaping and relaying it out.
    pub fn set_type(&self, style: TypeStyle) -> Result {
        self.change(|state, id| {
            state
                .control_as::<TextControl>(id)
                .ok_or(aegle_ui::UiError::WrongKind)?
                .style = style;
            let text_style = state.text_style(id);
            let fonts = state.fonts.clone();
            let control = state.control_as::<TextControl>(id).expect("checked above");
            fonts.borrow_mut().restyle(&mut control.text, &text_style)?;
            state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?;
            Ok(())
        })
    }
}
