//! Dividers: a 1 dp outline-variant line between groups of content, across
//! a column or down a row, optionally inset from its start and end.

use aegle_layout::Style;
use aegle_ui::{
    Align, Container, ControlKind, Result, Size,
    control::{Control, MeasureCx, PaintCx},
    handle,
    scene::{Rect, RoundedRect},
};

use crate::Scheme;

/// The control inside a [`Divider`].
pub struct DividerControl {
    vertical: bool,
}

impl Control for DividerControl {
    fn kind(&self) -> &'static ControlKind {
        &crate::glyph::ICON
    }
    fn frame(&self) -> aegle_ui::control::Frame {
        aegle_ui::control::Frame {
            background: false,
            border: false,
        }
    }
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(if self.vertical {
            Size::new(1.0, 0.0)
        } else {
            Size::new(0.0, 1.0)
        })
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let rect = Rect::new(0.0, 0.0, cx.size.width, cx.size.height);
        let color = Scheme::of(cx.theme).outline_variant;
        cx.builder.fill(RoundedRect::new(rect, 0.0)?, color)?;
        Ok(())
    }
}

handle! {
    /// A thin line separating content.
    pub Divider(DividerControl)
}

impl Divider {
    /// Appends a full-width divider to a column.
    pub fn new(parent: &Container) -> Result<Self> {
        Self::create(parent, false)
    }

    /// Appends a full-height divider to a row.
    pub fn vertical(parent: &Container) -> Result<Self> {
        Self::create(parent, true)
    }

    fn create(parent: &Container, vertical: bool) -> Result<Self> {
        parent
            .add(|_, _| {
                let style = Style {
                    flex_shrink: 0.0,
                    align_self: Some(Align::Stretch.items()),
                    ..Default::default()
                };
                Ok((
                    Box::new(DividerControl { vertical }) as Box<dyn Control>,
                    style,
                ))
            })
            .map(Self)
    }

    /// Insets it from its start and end (16 dp for an inset divider).
    pub fn set_inset(&self, start: f32, end: f32) -> Result {
        let vertical = self.read(|c| c.vertical)?;
        let insets = if vertical {
            crate::edges(0.0, start, 0.0, end)
        } else {
            crate::edges(start, 0.0, end, 0.0)
        };
        self.set_margin(insets)
    }
}
