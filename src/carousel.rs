//! Carousels: a row of 28 dp rounded items scrolling horizontally past the
//! edge (the uncontained layout), 8 dp apart, with vertical wheel motion
//! scrolling them sideways. Items clip their content, such as images.

use aegle_layout::{Overflow, Style};
use aegle_ui::{
    Align, Appearance, Container, ControlKind, Point, Result, Theme, VisualState,
    control::{Control, Input, InputCx, Outcome},
};

use crate::{
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
};

/// The scrolling row of a [`Carousel`].
pub struct CarouselControl;

impl Control for CarouselControl {
    fn kind(&self) -> &'static ControlKind {
        &CAROUSEL
    }
    fn viewport(&self) -> bool {
        true
    }
    fn takes_wheel(&self) -> bool {
        true
    }
    fn handle(&mut self, cx: &mut InputCx<'_>, input: Input<'_>) -> Result<Outcome> {
        let Input::Wheel { delta, .. } = input else {
            return Ok(Outcome::default());
        };
        let by = delta.x + delta.y;
        cx.deferred.push(Box::new(move |state, id| {
            let old = state.tree.get(id).unwrap().context.scroll;
            state.scroll_to(id, Point::new(old.x + by, 0.0))?;
            state.update_geometry()
        }));
        Ok(Outcome {
            handled: true,
            ..Outcome::default()
        })
    }
}

fn carousel(theme: &Theme, state: VisualState) -> Appearance {
    Appearance::base(theme, state)
}

/// A carousel's scrolling row.
pub static CAROUSEL: ControlKind = ControlKind {
    name: "Carousel",
    skin: carousel,
    accepts: aegle_ui::Accepts::NONE,
    container: true,
};

kinds! { container
    /// A carousel item.
    CAROUSEL_ITEM = "CarouselItem", item => |s, _on| {
        Paint::new(s.surface_container_highest, s.on_surface)
    };
}

/// A horizontally scrolling row of items.
#[derive(Clone)]
pub struct Carousel {
    row: Container,
    width: f32,
}

impl std::ops::Deref for Carousel {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.row
    }
}

impl Carousel {
    /// Appends an empty carousel `height` dp tall whose items are
    /// `item_width` dp wide, spanning its parent.
    pub fn new(parent: &Container, item_width: f32, height: f32) -> Result<Self> {
        aegle_ui::valid(item_width)?;
        let node = parent.add(|_, _| {
            let mut style = Style {
                flex_direction: aegle_layout::FlexDirection::Row,
                flex_shrink: 0.0,
                align_self: Some(Align::Stretch.items()),
                ..Default::default()
            };
            style.overflow.x = Overflow::Scroll;
            Ok((Box::new(CarouselControl) as Box<dyn Control>, style))
        })?;
        let row = Container(node);
        row.set_height(height)?;
        row.set_gap(8.0)?;
        row.set_padding(crate::edges(16.0, 0.0, 16.0, 0.0))?;
        Ok(Self {
            row,
            width: item_width,
        })
    }

    /// Appends an item to fill with content; a clickable one is a button.
    pub fn item(&self, clickable: bool) -> Result<crate::Card> {
        let mut control = SurfaceControl::new(&CAROUSEL_ITEM, Part::Card, 28.0, 0.0);
        if clickable {
            control = control.clickable();
        }
        let item = surface::add(&self.row, control, 16.0)?;
        item.set_width(self.width)?;
        item.set_clip(true)?;
        item.set_justify_content(Some(aegle_ui::Justify::End))?;
        Ok(crate::Card::wrap(item))
    }

    /// The horizontal scroll offset.
    pub fn offset(&self) -> Result<f32> {
        self.row
            .change(|state, id| Ok(state.tree.get(id).unwrap().context.scroll.x))
    }

    /// Scrolls to `x`, clamped to the items.
    pub fn scroll_to(&self, x: f32) -> Result {
        aegle_ui::valid(x)?;
        self.row.change(|state, id| {
            state.refresh()?;
            state.scroll_to(id, Point::new(x, 0.0))?;
            state.update_geometry()
        })
    }
}
