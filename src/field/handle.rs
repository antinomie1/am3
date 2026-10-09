//! The text field handle.

use aegle_layout::Style;
use aegle_ui::{Container, Result, State, handle, text_style};

use super::{FieldControl, ROLE, small};
use crate::Icon;

/// The two text field styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldStyle {
    /// A filled container with an active indicator, for more emphasis.
    #[default]
    Filled,
    /// An outline, for less emphasis or many fields together.
    Outlined,
    /// A 56 dp pill on the high surface container, for search.
    Search,
}

handle! {
    /// A Material text field.
    pub TextField(FieldControl): text, interactive, editor
}

/// Which line of extra text to replace.
enum Part {
    Label,
    Placeholder,
    Supporting,
}

impl TextField {
    /// Appends an empty single-line field labelled `label` (empty for none).
    pub fn new(parent: &Container, style: FieldStyle, label: &str) -> Result<Self> {
        Self::create(parent, style, label, false)
    }

    /// Appends an empty multiline field that grows with its text.
    pub fn multiline(parent: &Container, style: FieldStyle, label: &str) -> Result<Self> {
        Self::create(parent, style, label, true)
    }

    fn create(parent: &Container, style: FieldStyle, label: &str, multiline: bool) -> Result<Self> {
        let node = parent.add(|state, theme| {
            let mut text = text_style(theme);
            ROLE.apply(&mut text);
            let control = FieldControl::new(&state.fonts, &text, style, "", multiline)?;
            let layout = Style {
                flex_shrink: 0.0,
                ..Default::default()
            };
            Ok((Box::new(control) as Box<dyn aegle_ui::Control>, layout))
        })?;
        crate::effects(&node)?;
        let field = Self(node);
        if !label.is_empty() {
            field.set_part(Part::Label, Some(label))?;
            field.set_accessible_label(label)?;
        }
        Ok(field)
    }

    fn set_part(&self, part: Part, text: Option<&str>) -> Result {
        self.change(|state, id| {
            let style = state.text_style(id);
            let paragraph = match (&part, text) {
                (_, None) => None,
                (Part::Supporting, Some(text)) => {
                    Some(state.fonts.borrow_mut().paragraph(text, &small(&style))?)
                }
                (_, Some(text)) => Some(state.fonts.borrow_mut().paragraph(text, &style)?),
            };
            let control = control(state, id)?;
            *match part {
                Part::Label => &mut control.label,
                Part::Placeholder => &mut control.placeholder,
                Part::Supporting => &mut control.supporting,
            } = paragraph;
            Ok(state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?)
        })
    }

    /// Replaces the text, ending any IME composition.
    pub fn set_text(&self, text: &str) -> Result {
        self.change(|state, id| {
            state.set_text(id, text)?;
            update_counter(state, id)
        })
    }

    /// The committed text.
    pub fn text(&self) -> Result<String> {
        self.change(|state, id| state.text(id))
    }

    /// Shows `text` in the empty field while its label floats.
    pub fn set_placeholder(&self, text: Option<&str>) -> Result {
        self.set_part(Part::Placeholder, text)
    }

    /// Shows supporting text under the field, in the error color while it
    /// shows an error.
    pub fn set_supporting(&self, text: Option<&str>) -> Result {
        self.set_part(Part::Supporting, text)
    }

    /// Shows the field in the error colors.
    pub fn set_error(&self, error: bool) -> Result {
        self.change(|state, id| {
            control(state, id)?.error = error;
            state.dirty_visual_state(id)
        })
    }

    /// Whether it shows an error.
    pub fn is_error(&self) -> Result<bool> {
        self.read(|c| c.error)
    }

    /// Sets or removes the leading icon.
    pub fn set_leading_icon(&self, icon: Option<Icon>) -> Result {
        self.relayout(|c| c.leading = icon)
    }

    /// Sets or removes the trailing icon.
    pub fn set_trailing_icon(&self, icon: Option<Icon>) -> Result {
        self.relayout(|c| c.trailing = icon)
    }

    /// Shows a clear button at the end while the field has text; pressing
    /// it empties the field and reports a change.
    pub fn set_clearable(&self, clearable: bool) -> Result {
        self.relayout(|c| {
            c.clearable = clearable;
            if clearable && c.trailing.is_none() {
                c.trailing = Some(crate::icons::close());
            }
        })
    }

    /// Counts the characters against `max` under the field's end; the
    /// application decides what to do past it.
    pub fn set_counter(&self, max: Option<usize>) -> Result {
        self.change(|state, id| {
            let fonts = state.fonts.clone();
            let style = small(&state.text_style(id));
            let control = control(state, id)?;
            control.counter = match max {
                Some(max) => Some((max, fonts.borrow_mut().paragraph("", &style)?)),
                None => None,
            };
            update_counter(state, id)?;
            Ok(state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?)
        })
    }

    /// Masks the text, as for a password.
    pub fn set_password(&self, password: bool) -> Result {
        self.edit(|editor| {
            editor.set_password(password);
            Ok(())
        })
    }

    /// Allows selection but rejects edits.
    pub fn set_read_only(&self, read_only: bool) -> Result {
        self.edit(|editor| {
            editor.set_read_only(read_only);
            Ok(())
        })
    }

    /// Adds a handler run after each edit, and after Enter in a single-line
    /// field.
    pub fn on_change(&self, mut callback: impl FnMut(TextField) -> Result + 'static) -> Result {
        self.change(|state, id| state.on_action(id, move |node| callback(TextField(node))))
    }

    fn relayout(&self, change: impl FnOnce(&mut FieldControl)) -> Result {
        self.change(|state, id| {
            change(control(state, id)?);
            Ok(state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?)
        })
    }

    fn edit(
        &self,
        apply: impl FnOnce(
            &mut aegle_text::EditorDriver<'_>,
        ) -> std::result::Result<(), aegle_text::TextError>,
    ) -> Result {
        self.change(|state, id| {
            let fonts = state.fonts.clone();
            let field = &mut control(state, id)?.field;
            apply(&mut fonts.borrow_mut().edit(field.editor_mut()))?;
            Ok(state.tree.mark_dirty(id, aegle_ui::Dirty::ALL)?)
        })
    }
}

fn control(state: &mut State, id: aegle_ui::NodeId) -> Result<&mut FieldControl> {
    Ok(state
        .control_as::<FieldControl>(id)
        .ok_or(aegle_ui::UiError::WrongKind)?)
}

/// Shows the current length on the counter.
pub(super) fn update_counter(state: &mut State, id: aegle_ui::NodeId) -> Result {
    let style = small(&state.text_style(id));
    let fonts = state.fonts.clone();
    let control = control(state, id)?;
    let count: usize = (control.field.editor().text().parts().iter())
        .map(|part| part.chars().count())
        .sum();
    if let Some((max, text)) = &mut control.counter {
        fonts
            .borrow_mut()
            .update(text, &format!("{count}/{max}"), &style)?;
        state.tree.mark_dirty(id, aegle_ui::Dirty::PAINT)?;
    }
    Ok(())
}
