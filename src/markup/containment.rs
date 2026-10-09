//! Elements of text, icons and the containment and communication
//! components: cards, lists, dividers, dialogs, sheets and snackbars.

use std::time::Duration;

use aegle_loader::{Elements, element};
use aegle_ui::{Container, Result};

use super::{adopt, choices, icon, maybe_icon, owned, role};
use crate::{
    Button, Card, CardStyle, Dialog, Divider, IconView, List, ListItem, Sheet, Snackbar, Text,
    tokens::{TypeStyle, typescale as t},
};

choices! {
    type_style -> TypeStyle {
        "display_large" => t::DISPLAY_LARGE,
        "display_medium" => t::DISPLAY_MEDIUM,
        "display_small" => t::DISPLAY_SMALL,
        "headline_large" => t::HEADLINE_LARGE,
        "headline_medium" => t::HEADLINE_MEDIUM,
        "headline_small" => t::HEADLINE_SMALL,
        "title_large" => t::TITLE_LARGE,
        "title_medium" => t::TITLE_MEDIUM,
        "title_small" => t::TITLE_SMALL,
        "body_large" => t::BODY_LARGE,
        "body_medium" => t::BODY_MEDIUM,
        "body_small" => t::BODY_SMALL,
        "label_large" => t::LABEL_LARGE,
        "label_medium" => t::LABEL_MEDIUM,
        "label_small" => t::LABEL_SMALL,
    }
    card_style -> CardStyle {
        "elevated" => CardStyle::Elevated,
        "filled" => CardStyle::Filled,
        "outlined" => CardStyle::Outlined,
    }
}

fn list_item(
    parent: &Container,
    headline: &str,
    clickable: bool,
    [supporting, leading, trailing, trailing_text]: [&str; 4],
) -> Result<ListItem> {
    let list = owned::<List>(parent)?;
    let item = if clickable {
        list.clickable_item(headline)?
    } else {
        list.item(headline)?
    };
    if !supporting.is_empty() {
        item.supporting(supporting)?;
    }
    if let Some(icon) = maybe_icon(leading)? {
        item.leading_icon(icon)?;
    }
    if let Some(icon) = maybe_icon(trailing)? {
        item.trailing_icon(icon)?;
    }
    if !trailing_text.is_empty() {
        item.trailing_text(trailing_text)?;
    }
    Ok(item)
}

fn sheet(parent: &Container, kind: &str, headline: &str) -> Result<Sheet> {
    match kind {
        "side" => Sheet::side(parent, headline),
        "standard_bottom" => Sheet::standard_bottom(parent),
        "standard_side" => Sheet::standard_side(parent, headline),
        _ => Sheet::bottom(parent),
    }
}

element! {
    /// Text in a Material type role and color role.
    pub MdText(Text) {
        create |parent, text: string = "", typescale: choice(display_large, display_medium, display_small, headline_large, headline_medium, headline_small, title_large, title_medium, title_small, body_large, body_medium, body_small, label_large, label_medium, label_small) = "body_medium", color: line = "on_surface"| Text::new(parent, type_style(typescale), role(color)?, text);
        set text: string => |text, value| text.set_text(value);
        set color: line => |text, name| text.set_color(role(name)?);
        get text: string => |text| text.text();
    }
    /// A bundled icon, `size` dp square, in a color role.
    pub MdIcon(IconView) {
        create |parent, icon: line, size: float(0) = 24, color: line = "on_surface_variant"| IconView::new(parent, super::icon(icon)?, size as f32, role(color)?);
        set icon: line => |view, name| view.set_icon(icon(name)?);
        set color: line => |view, name| view.set_color(role(name)?);
    }
    /// A horizontal or vertical divider.
    pub MdDivider(Divider) {
        create |parent, vertical: bool = false| if vertical { Divider::vertical(parent) } else { Divider::new(parent) };
    }
    /// A card holding its children; a clickable card is a button.
    pub MdCard(Card) {
        layout flex;
        create |parent, style: choice(elevated, filled, outlined) = "filled", clickable: bool = false| if clickable { Card::clickable(parent, card_style(style)) } else { Card::new(parent, card_style(style)) };
        event clicked => |card, run| card.on_click(move |_| run());
    }
    /// A list of `MdListItem` children.
    pub MdList(List) {
        layout flex;
        children only MdListItem;
        create |parent| {
            let list = List::new(parent)?;
            adopt(list.clone(), &list)
        };
    }
    /// A list item: a headline with optional supporting text, a leading
    /// and a trailing icon or trailing text.
    pub MdListItem(ListItem) {
        parent MdList;
        create |parent, headline: line, clickable: bool = false, supporting: line = "", leading: line = "", trailing: line = "", trailing_text: line = ""| list_item(parent, headline, clickable, [supporting, leading, trailing, trailing_text]);
        event clicked => |item, run| item.on_click(move |_| run());
    }
    /// A dialog holding its children between the headline and its
    /// `MdDialogAction` buttons, shown while `open`.
    pub MdDialog(Dialog) {
        layout flex;
        create |parent, headline: line = "", icon: line = ""| {
            let dialog = Dialog::new(parent, maybe_icon(icon)?, headline)?;
            adopt(dialog.clone(), &dialog)
        };
        set headline: line => |dialog, text| dialog.headline().set_text(text);
        set open: bool => |dialog, open| if open { dialog.show() } else { dialog.close() };
        event dismissed => |dialog, run| dialog.on_dismiss(run);
    }
    /// A text button at a dialog's end.
    pub MdDialogAction(Button) {
        parent MdDialog;
        create |parent, text: line| owned::<Dialog>(parent)?.action(text);
        event clicked => |button, run| button.on_click(move |_| run());
    }
    /// A bottom or side sheet holding its children; modal ones show while
    /// `open`.
    pub MdSheet(Sheet) {
        layout flex;
        create |parent, kind: choice(bottom, side, standard_bottom, standard_side) = "bottom", headline: line = ""| sheet(parent, kind, headline);
        set open: bool => |sheet, open| if open { sheet.show() } else { sheet.close() };
        event dismissed => |sheet, run| sheet.on_dismiss(run);
    }
    /// A snackbar, shown while `open` for `duration` ms (0 until closed),
    /// with an optional `MdSnackbarAction`.
    pub MdSnackbar(Snackbar) {
        layout flex;
        children only MdSnackbarAction;
        create |parent, text: line, closable: bool = false, duration: int(0, 600000) = 4000| {
            let bar = Snackbar::new(parent, text)?;
            if closable {
                bar.closable()?;
            }
            // The duration stays with the handle until `open` uses it.
            adopt((bar.clone(), duration as u64), &bar)?;
            Ok(bar)
        };
        set text: line => |bar, text| bar.set_text(text);
        set open: bool => |bar, open| if open {
            let (_, ms) = owned::<(Snackbar, u64)>(bar)?;
            bar.show((ms > 0).then(|| Duration::from_millis(ms)))
        } else {
            bar.close()
        };
        event timeout => |bar, run| bar.on_timeout(run);
    }
    /// The action button of an `MdSnackbar`.
    pub MdSnackbarAction(Button) {
        parent MdSnackbar;
        create |parent, text: line| owned::<(Snackbar, u64)>(parent)?.0.action(text);
        event clicked => |button, run| button.on_click(move |_| run());
    }
}

pub(super) fn add(elements: Elements) -> Elements {
    elements
        .with::<MdText>()
        .with::<MdIcon>()
        .with::<MdDivider>()
        .with::<MdCard>()
        .with::<MdList>()
        .with::<MdListItem>()
        .with::<MdDialog>()
        .with::<MdDialogAction>()
        .with::<MdSheet>()
        .with::<MdSnackbar>()
        .with::<MdSnackbarAction>()
}
