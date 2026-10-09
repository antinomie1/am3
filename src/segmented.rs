//! Segmented buttons: two to five outlined segments that select options,
//! one at a time or several. A selected segment fills with the secondary
//! container color and shows a check mark.

use aegle_ui::{Color, Container, Result, handle};

use crate::{
    Icon, icons,
    pressable::{self, Look, PressableControl, Spec, pressable_methods},
    skin::{Paint, kinds},
    tokens::{Corner, shape, typescale},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SegmentLook;

impl SegmentLook {
    pub fn spec(self) -> Spec {
        Spec {
            height: 40.0,
            width: None,
            min_width: 48.0,
            padding: 12.0,
            icon_padding: 12.0,
            icon: 18.0,
            gap: 8.0,
            text: typescale::LABEL_LARGE,
            corners: [shape::FULL; 3],
            inner: [Corner::Dp(0.0); 3],
            hover_inner: false,
            outline: 1.0,
            elevation: [0.0, 0.0],
        }
    }
}

kinds! {
    /// A segment of a segmented button.
    SEGMENT = "Segment", segment => |s, on| if on {
        Paint::new(s.secondary_container, s.on_secondary_container).outlined(s.outline)
    } else {
        Paint::new(Color::TRANSPARENT, s.on_surface).outlined(s.outline)
    };
}

handle! {
    /// One segment of a [`SegmentedButton`].
    pub Segment(PressableControl): text, interactive, pressed, indicator
}

pressable_methods!(Segment);

/// A row of segments for selecting among a few options.
#[derive(Clone)]
pub struct SegmentedButton {
    row: Container,
    multiple: bool,
}

impl std::ops::Deref for SegmentedButton {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.row
    }
}

impl SegmentedButton {
    /// Appends an empty segmented button; with `multiple` its segments
    /// select independently, otherwise exactly one is selected.
    pub fn new(parent: &Container, multiple: bool) -> Result<Self> {
        let row = parent.row()?;
        row.set_gap(0.0)?;
        Ok(Self { row, multiple })
    }

    /// Appends a segment; in a single-select button the first one starts
    /// selected.
    pub fn segment(&self, text: &str, icon: Option<Icon>) -> Result<Segment> {
        let multiple = self.multiple;
        let first = self
            .row
            .change(|state, id| Ok(state.tree.children(id)?.next().is_none()))?;
        let segment = pressable::add(&self.row, |fonts, theme| {
            let mut control =
                PressableControl::new(fonts, theme, Look::Segment(SegmentLook), text)?;
            control.icon = icon;
            control.selected_icon = Some(icons::check());
            control.exclusive = !multiple;
            control.selected = Some(!multiple && first);
            Ok(control)
        })
        .map(Segment)?;
        // Join the segments: square inner corners, one shared outline.
        self.row.change(|state, id| {
            let children: Vec<_> = state.tree.children(id)?.collect();
            let last = children.len() - 1;
            for (index, child) in children.into_iter().enumerate() {
                if let Some(c) = state.control_as::<PressableControl>(child) {
                    c.joined = [index > 0, index < last];
                    c.reach = [if index > 0 { 1.0 } else { 0.0 }, 0.0];
                    state.tree.mark_dirty(child, aegle_ui::Dirty::PAINT)?;
                }
            }
            Ok(())
        })?;
        Ok(segment)
    }

    /// The indices of the selected segments.
    pub fn selected(&self) -> Result<Vec<usize>> {
        self.row.change(|state, id| {
            let children: Vec<_> = state.tree.children(id)?.collect();
            Ok(children
                .into_iter()
                .enumerate()
                .filter(|&(_, n)| {
                    state
                        .control_as::<PressableControl>(n)
                        .is_some_and(|c| c.selected == Some(true))
                })
                .map(|(i, _)| i)
                .collect())
        })
    }
}
