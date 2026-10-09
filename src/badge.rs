//! Badges: a small error-colored dot, or a 16 dp pill with a count, on the
//! top-end corner of a control's centered 24 dp icon, as on icon buttons
//! and navigation items. A badge decorates the control, so it follows its
//! layout, theme and visibility without a node of its own.

use std::{cell::RefCell, rc::Rc};

use aegle_text::{Paragraph, TextStyle};
use aegle_ui::{
    Decorator, Dirty, Node, Point, Result,
    control::PaintCx,
    scene::{Affine, Rect, RoundedRect},
    text_style,
};

use crate::{Scheme, tokens::typescale};

#[derive(Default)]
struct Shown {
    /// `None` for a small badge, else its label.
    label: Option<Paragraph>,
    visible: bool,
}

struct Paint(Rc<RefCell<Shown>>);

impl Decorator for Paint {
    fn over(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let shown = self.0.borrow();
        if !shown.visible {
            return Ok(());
        }
        let s = Scheme::of(cx.theme);
        // The icon's box, centered in the control.
        let icon = Rect::new(
            (cx.size.width - 24.0) / 2.0,
            (cx.size.height - 24.0) / 2.0,
            24.0,
            24.0,
        );
        let end = |width: f32, inset: f32| {
            if cx.rtl {
                icon.origin.x + inset - width
            } else {
                icon.origin.x + icon.size.width - inset
            }
        };
        match &shown.label {
            None => {
                let dot = Rect::new(end(6.0, 6.0), icon.origin.y, 6.0, 6.0);
                cx.builder.fill(RoundedRect::new(dot, 3.0)?, s.error)?;
            }
            Some(label) => {
                let text = label.size();
                let width = (text.width + 8.0).max(16.0);
                let pill = Rect::new(end(width, 12.0), icon.origin.y - 2.0, width, 16.0);
                cx.builder.fill(RoundedRect::new(pill, 8.0)?, s.error)?;
                let at = Point::new(
                    pill.origin.x + (width - text.width) / 2.0,
                    pill.origin.y + (16.0 - text.height) / 2.0,
                );
                cx.builder
                    .push_transform(Affine::translation(at.x, at.y)?)?;
                label.paint_with_color(cx.builder, s.on_error)?;
                cx.builder.pop()?;
            }
        }
        Ok(())
    }
}

/// A badge on a control.
#[derive(Clone)]
pub struct Badge {
    node: Node,
    shown: Rc<RefCell<Shown>>,
}

impl Badge {
    /// Adds a hidden badge to `node`.
    pub fn new(node: &Node) -> Result<Self> {
        let shown = Rc::new(RefCell::new(Shown::default()));
        node.decorate(Paint(shown.clone()))?;
        Ok(Self {
            node: node.clone(),
            shown,
        })
    }

    /// Shows a small dot.
    pub fn show_dot(&self) -> Result {
        *self.shown.borrow_mut() = Shown {
            label: None,
            visible: true,
        };
        self.repaint()
    }

    /// Shows `count`, as "999+" above 999.
    pub fn show_count(&self, count: u32) -> Result {
        let text = if count > 999 {
            "999+".to_owned()
        } else {
            count.to_string()
        };
        self.show_text(&text)
    }

    /// Shows a short label.
    pub fn show_text(&self, text: &str) -> Result {
        let label = self.node.change(|state, id| {
            let mut style: TextStyle<'_> = text_style(state.theme_of(id));
            typescale::LABEL_SMALL.apply(&mut style);
            Ok(state.fonts.borrow_mut().paragraph(text, &style)?)
        })?;
        *self.shown.borrow_mut() = Shown {
            label: Some(label),
            visible: true,
        };
        self.repaint()
    }

    /// Hides the badge.
    pub fn hide(&self) -> Result {
        self.shown.borrow_mut().visible = false;
        self.repaint()
    }

    fn repaint(&self) -> Result {
        self.node
            .change(|state, id| Ok(state.tree.mark_dirty(id, Dirty::PAINT)?))
    }
}
