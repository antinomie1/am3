//! Tooltips. A plain tooltip is Aegle's (shown after the pointer rests,
//! hidden on press, Escape or leaving, and exported as the description) on
//! the inverse surface with 4 dp corners. A rich tooltip is a 12 dp rounded
//! surface below its anchor with an optional subhead, supporting text and
//! actions, shown and hidden by the application.

use aegle_ui::{Appearance, Container, Justify, Node, Result, Theme, VisualState};
use aegle_widgets::{NodePopup, NodeTooltip, Popup, kinds};

use crate::{
    Button, ButtonStyle, Role, Scheme, Text,
    surface::{self, Part, RICH_TOOLTIP, SurfaceControl},
    tokens::typescale,
};

fn plain(theme: &Theme, state: VisualState) -> Appearance {
    let s = Scheme::of(theme);
    Appearance {
        background: s.inverse_surface,
        foreground: s.inverse_on_surface,
        radius: 4.0,
        ..Appearance::base(theme, state)
    }
}

/// Shows `text` in a plain tooltip after the pointer rests on `node`, and
/// describes `node` with it; `None` removes it.
pub fn set_tooltip(node: &Node, text: Option<&str>) -> Result {
    // Tooltips show under the root, so the skin applies there.
    let root = node.change(|state, _| Ok(state.root))?;
    Node {
        state: node.state.clone(),
        id: root,
    }
    .set_kind_skin(&kinds::TOOLTIP, Some(plain))?;
    node.set_tooltip(text)
}

fn clear(theme: &Theme, state: VisualState) -> Appearance {
    Appearance {
        border_width: 0.0,
        ..Appearance::base(theme, state)
    }
}

/// A rich tooltip below its anchor.
#[derive(Clone)]
pub struct RichTooltip {
    popup: Popup,
    surface: Container,
    actions: Container,
}

impl std::ops::Deref for RichTooltip {
    type Target = Container;
    /// The tooltip's surface, holding its text.
    fn deref(&self) -> &Container {
        &self.surface
    }
}

impl RichTooltip {
    /// A hidden rich tooltip for `anchor` with an optional `subhead` and
    /// `text`, up to 320 dp wide.
    pub fn new(anchor: &Node, subhead: Option<&str>, text: &str) -> Result<Self> {
        let popup = anchor.popup()?;
        popup.set_skin(Some(clear))?;
        popup.set_padding(4.0)?;
        let control = SurfaceControl::new(&RICH_TOOLTIP, Part::Tooltip, 12.0, 2.0);
        let surface = surface::add(&popup, control, 0.0)?;
        surface.set_max_width(320.0)?;
        surface.set_gap(4.0)?;
        surface.set_padding(crate::edges(16.0, 12.0, 16.0, 8.0))?;
        if let Some(subhead) = subhead {
            Text::new(
                &surface,
                typescale::TITLE_SMALL,
                Role::on_surface_variant,
                subhead,
            )?;
        }
        Text::new(
            &surface,
            typescale::BODY_MEDIUM,
            Role::on_surface_variant,
            text,
        )?;
        let actions = surface.row()?;
        actions.set_justify_content(Some(Justify::Start))?;
        actions.set_gap(8.0)?;
        actions.set_visible(false)?;
        Ok(Self {
            popup,
            surface,
            actions,
        })
    }

    /// Appends a text-button action; activating it hides the tooltip too.
    pub fn action(&self, text: &str) -> Result<Button> {
        self.actions.set_visible(true)?;
        let button = Button::new(&self.actions, ButtonStyle::Text, text)?;
        let popup = self.popup.clone();
        button.on_click(move |_| popup.hide())?;
        Ok(button)
    }

    /// Shows it below the anchor, or above when only that fits.
    pub fn show(&self) -> Result {
        self.popup.show()
    }

    /// Hides it.
    pub fn hide(&self) -> Result {
        self.popup.hide()
    }

    /// Whether it is shown.
    pub fn is_shown(&self) -> Result<bool> {
        self.popup.is_shown()
    }
}
