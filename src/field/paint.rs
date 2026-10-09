//! Painting a text field: its container or outline, the floating label,
//! icons, the editor's text and the supporting line.

use aegle_ui::{
    Point, Rect, Result,
    control::PaintCx,
    scene::{Affine, FillRule, PathBuilder, RoundedRect, Stroke},
};

use super::FieldControl;
use crate::{
    Scheme,
    color::alpha,
    icons,
    pressable::shape::{corner, rounded},
    tokens::{motion, state},
};

/// The floating label's scale: body small over body large.
const SMALL: f32 = 0.75;

/// An outline with corners `r`, open along the top edge from `gap.0` to
/// `gap.1` for the label.
fn outline(rect: Rect, r: f32, gap: (f32, f32)) -> Result<aegle_ui::scene::Path> {
    let (x0, y0) = (rect.origin.x, rect.origin.y);
    let (x1, y1) = (x0 + rect.size.width, y0 + rect.size.height);
    let p = Point::new;
    let mut b = PathBuilder::new();
    b.move_to(p(gap.1.max(x0 + r), y0)).line_to(p(x1 - r, y0));
    corner(&mut b, p(x1 - r, y0), p(x1, y0 + r), p(x1, y0), r);
    b.line_to(p(x1, y1 - r));
    corner(&mut b, p(x1, y1 - r), p(x1 - r, y1), p(x1, y1), r);
    b.line_to(p(x0 + r, y1));
    corner(&mut b, p(x0 + r, y1), p(x0, y1 - r), p(x0, y1), r);
    b.line_to(p(x0, y0 + r));
    corner(&mut b, p(x0, y0 + r), p(x0 + r, y0), p(x0, y0), r);
    b.line_to(p(gap.0.max(x0 + r), y0));
    Ok(b.finish(FillRule::NonZero)?)
}

pub(super) fn paint(c: &mut FieldControl, cx: &mut PaintCx<'_>) -> Result {
    let (now, reduced, size) = (cx.time, cx.reduced_motion, cx.size);
    let a = *cx.appearance;
    let s = Scheme::of(cx.theme);
    let h = c.container_height();
    let w = size.width;
    let focused = cx.visual.focused;
    let floated = focused || !c.empty();
    let (t, moving) = c.float.at(
        if floated { 1.0 } else { 0.0 },
        motion::FAST_EFFECTS,
        now,
        reduced,
    );
    let label = c.label.as_ref().map(|l| l.size()).unwrap_or_default();
    let rest = if c.field.editor().is_multiline() {
        16.0
    } else {
        (h - label.height) / 2.0
    };
    let (float_x, float_y) = if c.filled() {
        (c.text_x(), 8.0)
    } else {
        (16.0, -label.height * SMALL / 2.0)
    };
    let label_at = Point::new(
        c.text_x() + (float_x - c.text_x()) * t,
        rest + (float_y - rest) * t,
    );
    let scale = 1.0 + (SMALL - 1.0) * t;

    let line = a.border_width;
    if c.style == super::FieldStyle::Search {
        let pill = RoundedRect::new(Rect::new(0.0, 0.0, w, h), h / 2.0)?;
        cx.builder.fill(pill, a.background)?;
    } else if c.filled() {
        let box_ = rounded(Rect::new(0.0, 0.0, w, h), [4.0, 4.0, 0.0, 0.0])?;
        cx.builder.fill_path(&box_, a.background)?;
        let indicator = Rect::new(0.0, h - line, w, line);
        cx.builder
            .fill(RoundedRect::new(indicator, 0.0)?, a.border_color)?;
    } else {
        let edge = Rect::new(line / 2.0, line / 2.0, w - line, h - line);
        let gap = match c.label {
            Some(_) if t > 0.01 => {
                let width = (label.width * SMALL + 8.0) * t;
                let center = float_x + label.width * SMALL / 2.0;
                (center - width / 2.0, center + width / 2.0)
            }
            _ => (0.0, 0.0),
        };
        let path = outline(edge, 4.0, gap)?;
        cx.builder
            .stroke_path(&path, a.border_color, Stroke::new(line))?;
    }

    let muted = if cx.visual.enabled {
        s.on_surface_variant
    } else {
        alpha(s.on_surface, state::DISABLED_CONTENT)
    };
    let icon_y = (h - 24.0) / 2.0;
    if let Some(icon) = &c.leading {
        icon.paint(cx.builder, Point::new(12.0, icon_y), 24.0, muted)?;
    }
    let trailing = match (&c.trailing, c.clearable && !c.empty()) {
        (_, true) => Some(icons::close()),
        (Some(icon), false) if !c.clearable => Some(icon.clone()),
        _ => None,
    };
    if let Some(icon) = trailing {
        let color = if c.error && cx.visual.enabled {
            s.error
        } else {
            muted
        };
        icon.paint(cx.builder, Point::new(w - 36.0, icon_y), 24.0, color)?;
    }

    if let Some(text) = &c.label {
        let place =
            Affine::translation(label_at.x, label_at.y)?.then(Affine::scale(scale, scale)?)?;
        cx.builder.push_transform(place)?;
        text.paint_with_color(cx.builder, a.indicator)?;
        cx.builder.pop()?;
    }

    let clip = Rect::new(c.text_x(), 0.0, (w - c.text_x() - c.text_end()).max(0.0), h);
    cx.builder.push_clip(RoundedRect::new(clip, 0.0)?)?;
    cx.builder.push_transform(Affine::translation(
        c.text_x() - cx.scroll.x,
        c.text_y() - cx.scroll.y,
    )?)?;
    if c.empty()
        && (c.label.is_none() || floated)
        && let Some(placeholder) = &c.placeholder
    {
        placeholder.paint_with_color(cx.builder, muted)?;
    }
    c.field.editor().paint(
        cx.builder,
        aegle_text::EditorPaint {
            foreground: Some(a.foreground),
            caret: focused.then_some(a.caret),
            preedit: Some(a.caret),
            selection: Some(a.selection),
            caret_width: 2.0,
        },
    )?;
    cx.builder.pop()?.pop()?;

    let below = if c.error && cx.visual.enabled {
        s.error
    } else {
        muted
    };
    if let Some(text) = &c.supporting {
        cx.builder
            .push_transform(Affine::translation(16.0, h + 4.0)?)?;
        text.paint_with_color(cx.builder, below)?;
        cx.builder.pop()?;
    }
    if let Some((_, text)) = &c.counter {
        let x = w - 16.0 - text.size().width;
        cx.builder
            .push_transform(Affine::translation(x, h + 4.0)?)?;
        text.paint_with_color(cx.builder, muted)?;
        cx.builder.pop()?;
    }
    if moving {
        cx.request_frame();
    }
    Ok(())
}
