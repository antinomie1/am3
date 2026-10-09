//! The row of a [`Tabs`](super::Tabs): it draws the divider and the
//! indicator, which slides to the selected tab once layout places it.

use aegle_ui::{
    Appearance, ControlKind, NodeId, Rect, Result, State, Theme, VisualState,
    control::{Control, PaintCx},
    scene::RoundedRect,
};

use super::{TabControl, TabStyle};
use crate::{Scheme, anim::Value, tokens::motion};

/// The tab rows of a UI, so their indicators follow layout.
#[derive(Default)]
pub(super) struct Rows(pub(super) Vec<NodeId>);

/// The control of a tab row: it draws the divider and the indicator.
pub struct TabRowControl {
    pub(super) style: TabStyle,
    /// The indicator's target start and width, relative to the row.
    pub(super) target: Option<(f32, f32)>,
    pub(super) at: [Value; 2],
}

impl Control for TabRowControl {
    fn kind(&self) -> &'static ControlKind {
        &TAB_ROW
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: true,
            border: false,
        }
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
        let s = Scheme::of(cx.theme);
        let divider = Rect::new(0.0, size.height - 1.0, size.width, 1.0);
        cx.builder
            .fill(RoundedRect::new(divider, 0.0)?, s.outline_variant)?;
        let Some((x, width)) = self.target else {
            return Ok(());
        };
        let spring = motion::DEFAULT_SPATIAL;
        let (x, a) = self.at[0].at(x, spring, now, reduced);
        let (width, b) = self.at[1].at(width, spring, now, reduced);
        let (height, radius) = match self.style {
            TabStyle::Primary => (3.0, 3.0),
            TabStyle::Secondary => (2.0, 0.0),
        };
        let bar = Rect::new(x, size.height - height, width.max(0.0), height);
        // Rounded on top only: a taller rounded box clipped to the bar.
        cx.builder.push_clip(RoundedRect::new(bar, 0.0)?)?;
        let tall = Rect::new(bar.origin.x, bar.origin.y, bar.size.width, height + radius);
        cx.builder
            .fill(RoundedRect::new(tall, radius)?, cx.appearance.indicator)?;
        cx.builder.pop()?;
        if a || b {
            cx.request_frame();
        }
        Ok(())
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        cx.node.set_role(aegle_ui::accesskit::Role::TabList);
    }
}

fn tab_row(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    Appearance {
        background: s.surface,
        indicator: s.primary,
        radius: 0.0,
        ..Appearance::base(theme, state)
    }
}

/// A row of tabs.
pub static TAB_ROW: ControlKind = ControlKind {
    name: "TabRow",
    skin: tab_row,
    accepts: aegle_ui::Accepts::INDICATOR,
    container: true,
};
/// Hook: points each tab row's indicator at its selected tab.
pub(crate) fn place(state: &mut State) -> bool {
    let Some(rows) = state.ext_ref::<Rows>().map(|r| r.0.clone()) else {
        return false;
    };
    for row in rows {
        let origin = state.tree.get(row).unwrap().context.bounds.origin;
        let children: Vec<_> = state.tree.children(row).into_iter().flatten().collect();
        let mut target = None;
        for child in children {
            let bounds = state.tree.get(child).unwrap().context.bounds;
            if let Some(tab) = state.control_as::<TabControl>(child)
                && tab.selected
            {
                let x = bounds.origin.x - origin.x;
                target = Some(if tab.secondary {
                    (x, bounds.size.width)
                } else {
                    let w = tab.content_width();
                    (x + (bounds.size.width - w) / 2.0, w)
                });
            }
        }
        let control = state.control_as::<TabRowControl>(row).expect("a tab row");
        if control.target != target {
            control.target = target;
            let _ = state.tree.mark_dirty(row, aegle_ui::Dirty::PAINT);
        }
    }
    false
}

/// Hook: forgets removed rows.
pub(crate) fn removed(state: &mut State, id: NodeId) {
    if state.ext_ref::<Rows>().is_some() {
        state.ext::<Rows>().0.retain(|&row| row != id);
    }
}
