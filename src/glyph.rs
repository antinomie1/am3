//! A standalone icon in a color role, as in dialog headers, list items and
//! app bars.

use aegle_layout::Style;
use aegle_ui::{
    Appearance, Container, ControlKind, Point, Result, Size, Theme, VisualState,
    control::{Control, MeasureCx, PaintCx},
    handle,
};

use crate::{Icon, Role, Scheme, color::alpha, tokens::state};

/// The control inside an [`IconView`].
pub struct IconControl {
    pub(crate) icon: Icon,
    pub(crate) size: f32,
    pub(crate) color: Role,
}

impl Control for IconControl {
    fn kind(&self) -> &'static ControlKind {
        &ICON
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
    fn measure(&mut self, _: &MeasureCx<'_>) -> Result<Size> {
        Ok(Size::new(self.size, self.size))
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) -> Result {
        let mut color = Scheme::of(cx.theme).role(self.color);
        if !cx.visual.enabled {
            color = alpha(color, state::DISABLED_CONTENT);
        }
        let at = Point::new(
            (cx.size.width - self.size) / 2.0,
            (cx.size.height - self.size) / 2.0,
        );
        self.icon.paint(cx.builder, at, self.size, color)
    }
    #[cfg(feature = "accessibility")]
    fn semantics(&self, cx: &mut aegle_ui::control::SemanticsCx<'_>) {
        cx.node.set_role(aegle_ui::accesskit::Role::Image);
    }
}

fn icon_skin(theme: &Theme, state: VisualState) -> Appearance {
    Appearance::base(theme, state)
}

/// A standalone icon.
pub static ICON: ControlKind = ControlKind {
    name: "Icon",
    skin: icon_skin,
    accepts: aegle_ui::Accepts::NONE,
    container: false,
};

handle! {
    /// A standalone icon.
    pub IconView(IconControl)
}

impl IconView {
    /// Appends `icon` at `size` dp in the `color` role.
    pub fn new(parent: &Container, icon: Icon, size: f32, color: Role) -> Result<Self> {
        parent
            .add(|_, _| {
                let style = Style {
                    flex_shrink: 0.0,
                    ..Default::default()
                };
                let control = IconControl { icon, size, color };
                Ok((Box::new(control) as Box<dyn Control>, style))
            })
            .map(Self)
    }

    /// Shows another icon.
    pub fn set_icon(&self, icon: Icon) -> Result {
        self.update(|c| c.icon = icon)
    }

    /// Colors it with another role.
    pub fn set_color(&self, color: Role) -> Result {
        self.update(|c| c.color = color)
    }
}
