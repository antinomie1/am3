//! Creating selection controls and the methods their handles share.

use aegle_layout::Style;
use aegle_ui::{Container, Result};

use super::{Mark, SelectionControl};

/// Appends a selection control.
pub(crate) fn add(
    parent: &Container,
    mark: Mark,
    text: &str,
    checked: bool,
) -> Result<aegle_ui::Node> {
    let node = parent.add(|state, theme| {
        if mark == Mark::Radio {
            state.install(&crate::HOOKS);
        }
        let control = SelectionControl::new(&state.fonts, theme, mark, text, checked)?;
        Ok((
            Box::new(control),
            Style {
                flex_shrink: 0.0,
                ..Default::default()
            },
        ))
    })?;
    crate::effects(&node)?;
    Ok(node)
}

/// The methods the selection handles share.
macro_rules! selection_methods {
    ($handle:ident) => {
        impl $handle {
            /// Whether it is checked, selected or on.
            pub fn is_checked(&self) -> aegle_ui::Result<bool> {
                self.read(|c| c.is_checked())
            }
            /// Sets the state without reporting a change.
            pub fn set_checked(&self, checked: bool) -> aegle_ui::Result {
                self.change(|state, id| {
                    let control = state
                        .control_as::<$crate::selection::SelectionControl>(id)
                        .ok_or(aegle_ui::UiError::WrongKind)?;
                    let outcome = control.set_checked(checked);
                    state.effects(id, outcome)
                })
            }
            /// Replaces the label.
            pub fn set_text(&self, text: &str) -> aegle_ui::Result {
                self.change(|state, id| state.set_text(id, text))
            }
            /// The label.
            pub fn text(&self) -> aegle_ui::Result<String> {
                self.read(|c| c.text().to_owned())
            }
            /// Adds a handler run after each change by the user.
            pub fn on_change(
                &self,
                mut callback: impl FnMut($handle) -> aegle_ui::Result + 'static,
            ) -> aegle_ui::Result {
                self.change(|state, id| state.on_action(id, move |node| callback($handle(node))))
            }
        }
    };
}
pub(crate) use selection_methods;
