//! Creating pressable controls and the methods their handles share.

use std::cell::RefCell;

use aegle_layout::Style;
use aegle_text::TextSystem;
use aegle_ui::{Container, Node, Result};

use super::{PressableControl, Spec};

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
pub(crate) fn size(spec: &Spec, style: &mut Style) {
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
        // The kind may have changed with the look.
        state.dirty_visual_state(id)
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
                // A selected icon may change the width.
                $crate::pressable::relook(self, |c| c.selected = selected)
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
            /// toggle button, after the input batch; an input chip's removal
            /// is not a click.
            pub fn on_click(
                &self,
                mut callback: impl FnMut($handle) -> aegle_ui::Result + 'static,
            ) -> aegle_ui::Result {
                self.on_click_any(move |handle| {
                    if handle.read(|c| c.removing)? {
                        return Ok(());
                    }
                    callback(handle)
                })
            }
            fn on_click_any(
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
