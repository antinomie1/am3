//! Vector icons: SVG path data in a 24-unit square, filled in the content
//! color of the control that shows them. [`icons`] has the Material Icons the
//! controls themselves need; applications add their own the same way.

use std::fmt;

use aegle_ui::{
    Color, Point, Result, Size,
    scene::{Affine, FillRule, Path, PathBuilder, SceneBuilder},
};

/// A filled vector icon, cheap to clone and share between controls.
#[derive(Clone)]
pub struct Icon(Path);

impl fmt::Debug for Icon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Icon")
    }
}

impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

/// SVG path data that could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidPath;

impl fmt::Display for InvalidPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid SVG path data")
    }
}

impl std::error::Error for InvalidPath {}

impl Icon {
    /// Reads SVG path data (`M L H V C S Q T A Z`, absolute and relative)
    /// drawn in a `size`-unit square: 24 for Material Icons, 960 for Material
    /// Symbols, whose y axis runs from -960 to 0.
    pub fn from_svg(data: &str, size: f32) -> std::result::Result<Self, InvalidPath> {
        let mut parser = Parser {
            bytes: data.as_bytes(),
            at: 0,
        };
        let mut path = PathBuilder::new();
        let k = 24.0 / size;
        // Material Symbols put the square above the x axis.
        let shift = if data.contains("-960") { 960.0 } else { 0.0 };
        let map = |p: Point| Point::new(p.x * k, (p.y + shift) * k);
        let (mut current, mut start) = (Point::default(), Point::default());
        let mut last_control: Option<(u8, Point)> = None;
        let mut command = 0u8;
        while let Some(next) = parser.command(command)? {
            command = next;
            let relative = command.is_ascii_lowercase();
            let base = if relative { current } else { Point::default() };
            let point = |parser: &mut Parser| -> std::result::Result<Point, InvalidPath> {
                let (x, y) = (parser.number()?, parser.number()?);
                Ok(Point::new(base.x + x, base.y + y))
            };
            let reflected = |kind: u8| match last_control {
                Some((k, c)) if k == kind => {
                    Point::new(2.0 * current.x - c.x, 2.0 * current.y - c.y)
                }
                _ => current,
            };
            let mut control = None;
            match command.to_ascii_uppercase() {
                b'M' => {
                    current = point(&mut parser)?;
                    start = current;
                    path.move_to(map(current));
                    // Further pairs are implicit line-tos.
                    command = if relative { b'l' } else { b'L' };
                }
                b'L' => {
                    current = point(&mut parser)?;
                    path.line_to(map(current));
                }
                b'H' => {
                    let x = parser.number()?;
                    current.x = if relative { current.x + x } else { x };
                    path.line_to(map(current));
                }
                b'V' => {
                    let y = parser.number()?;
                    current.y = if relative { current.y + y } else { y };
                    path.line_to(map(current));
                }
                b'C' => {
                    let (a, b, end) = (
                        point(&mut parser)?,
                        point(&mut parser)?,
                        point(&mut parser)?,
                    );
                    path.cubic_to(map(a), map(b), map(end));
                    (current, control) = (end, Some((b'C', b)));
                }
                b'S' => {
                    let a = reflected(b'C');
                    let (b, end) = (point(&mut parser)?, point(&mut parser)?);
                    path.cubic_to(map(a), map(b), map(end));
                    (current, control) = (end, Some((b'C', b)));
                }
                b'Q' => {
                    let (a, end) = (point(&mut parser)?, point(&mut parser)?);
                    path.quad_to(map(a), map(end));
                    (current, control) = (end, Some((b'Q', a)));
                }
                b'T' => {
                    let a = reflected(b'Q');
                    let end = point(&mut parser)?;
                    path.quad_to(map(a), map(end));
                    (current, control) = (end, Some((b'Q', a)));
                }
                b'A' => {
                    let (rx, ry, rotation) = (parser.number()?, parser.number()?, parser.number()?);
                    let (large, sweep) = (parser.flag()?, parser.flag()?);
                    let end = point(&mut parser)?;
                    arc(
                        &mut path, &map, current, end, rx, ry, rotation, large, sweep,
                    );
                    current = end;
                }
                b'Z' => {
                    path.close();
                    current = start;
                }
                _ => return Err(InvalidPath),
            }
            last_control = control;
        }
        path.finish(FillRule::NonZero)
            .map(Self)
            .map_err(|_| InvalidPath)
    }

    /// Fills the icon in a `size` square whose top-left corner is `at`.
    pub fn paint(&self, builder: &mut SceneBuilder, at: Point, size: f32, color: Color) -> Result {
        let k = size / 24.0;
        builder.push_transform(Affine::scale(k, k)?.then(Affine::translation(at.x, at.y)?)?)?;
        builder.fill_path(&self.0, color)?;
        builder.pop()?;
        Ok(())
    }

    /// Fills the icon centered in a `bounds`-sized box at `origin`.
    pub fn paint_centered(
        &self,
        builder: &mut SceneBuilder,
        origin: Point,
        bounds: Size,
        size: f32,
        color: Color,
    ) -> Result {
        let at = Point::new(
            origin.x + (bounds.width - size) / 2.0,
            origin.y + (bounds.height - size) / 2.0,
        );
        self.paint(builder, at, size, color)
    }
}

/// An SVG elliptical arc as cubic segments (SVG implementation notes, F.6).
#[allow(clippy::too_many_arguments)]
fn arc(
    path: &mut PathBuilder,
    map: &impl Fn(Point) -> Point,
    from: Point,
    to: Point,
    rx: f32,
    ry: f32,
    rotation: f32,
    large: bool,
    sweep: bool,
) {
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx == 0.0 || ry == 0.0 || from == to {
        path.line_to(map(to));
        return;
    }
    let (sin, cos) = rotation.to_radians().sin_cos();
    let (dx, dy) = ((from.x - to.x) / 2.0, (from.y - to.y) / 2.0);
    let (x1, y1) = (cos * dx + sin * dy, -sin * dx + cos * dy);
    let scale = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
    if scale > 1.0 {
        rx *= scale.sqrt();
        ry *= scale.sqrt();
    }
    let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let coef = sign * (num / den).sqrt();
    let (cx1, cy1) = (coef * rx * y1 / ry, -coef * ry * x1 / rx);
    let center = Point::new(
        cos * cx1 - sin * cy1 + (from.x + to.x) / 2.0,
        sin * cx1 + cos * cy1 + (from.y + to.y) / 2.0,
    );
    let angle = |ux: f32, uy: f32| uy.atan2(ux);
    let start = angle((x1 - cx1) / rx, (y1 - cy1) / ry);
    let mut delta = angle((-x1 - cx1) / rx, (-y1 - cy1) / ry) - start;
    let tau = std::f32::consts::TAU;
    if sweep && delta < 0.0 {
        delta += tau;
    } else if !sweep && delta > 0.0 {
        delta -= tau;
    }
    let segments = (delta.abs() / (tau / 4.0)).ceil().max(1.0);
    let step = delta / segments;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let at = |t: f32| {
        let (s, c) = t.sin_cos();
        let (x, y) = (rx * c, ry * s);
        Point::new(center.x + cos * x - sin * y, center.y + sin * x + cos * y)
    };
    let tangent = |t: f32| {
        let (s, c) = t.sin_cos();
        let (x, y) = (-rx * s, ry * c);
        Point::new(cos * x - sin * y, sin * x + cos * y)
    };
    let mut t = start;
    for _ in 0..segments as usize {
        let (p0, p3) = (at(t), at(t + step));
        let (d0, d3) = (tangent(t), tangent(t + step));
        let c1 = Point::new(p0.x + k * d0.x, p0.y + k * d0.y);
        let c2 = Point::new(p3.x - k * d3.x, p3.y - k * d3.y);
        path.cubic_to(map(c1), map(c2), map(p3));
        t += step;
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn skip(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| b.is_ascii_whitespace() || *b == b',')
        {
            self.at += 1;
        }
    }

    /// The next command letter, or `previous` repeated when a number follows.
    fn command(&mut self, previous: u8) -> std::result::Result<Option<u8>, InvalidPath> {
        self.skip();
        match self.bytes.get(self.at) {
            None => Ok(None),
            Some(b) if b.is_ascii_alphabetic() => {
                self.at += 1;
                Ok(Some(*b))
            }
            Some(_) if previous != 0 && !previous.eq_ignore_ascii_case(&b'z') => Ok(Some(previous)),
            Some(_) => Err(InvalidPath),
        }
    }

    fn number(&mut self) -> std::result::Result<f32, InvalidPath> {
        self.skip();
        let start = self.at;
        let mut dot = false;
        let mut exponent = false;
        while let Some(&b) = self.bytes.get(self.at) {
            let sign = (b == b'-' || b == b'+')
                && (self.at == start || matches!(self.bytes[self.at - 1], b'e' | b'E'));
            if b.is_ascii_digit() || sign {
            } else if b == b'.' && !dot && !exponent {
                dot = true;
            } else if (b == b'e' || b == b'E') && !exponent {
                exponent = true;
            } else {
                break;
            }
            self.at += 1;
        }
        std::str::from_utf8(&self.bytes[start..self.at])
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|v: &f32| v.is_finite())
            .ok_or(InvalidPath)
    }

    fn flag(&mut self) -> std::result::Result<bool, InvalidPath> {
        self.skip();
        let flag = match self.bytes.get(self.at) {
            Some(b'0') => false,
            Some(b'1') => true,
            _ => return Err(InvalidPath),
        };
        self.at += 1;
        Ok(flag)
    }
}

macro_rules! icons {
    ($($name:ident $data:literal,)*) => {
        /// Material Icons (Apache License 2.0) used by the controls and handy
        /// for applications. Each is parsed once per thread.
        pub mod icons {
            use super::Icon;
            $(
                #[allow(missing_docs)]
                pub fn $name() -> Icon {
                    thread_local! {
                        static ICON: Icon = Icon::from_svg($data, 24.0).expect("bundled icons parse");
                    }
                    ICON.with(Icon::clone)
                }
            )*
        }
    };
}

icons! {
    add "M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z",
    remove "M19 13H5v-2h14v2z",
    check "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z",
    close "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z",
    menu "M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z",
    search "M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z",
    arrow_back "M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z",
    arrow_forward "M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z",
    more_vert "M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z",
    edit "M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z",
    favorite "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z",
    star "M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z",
    home "M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z",
    delete "M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z",
    share "M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z",
    arrow_drop_down "M7 10l5 5 5-5z",
    arrow_drop_up "M7 14l5-5 5 5z",
    chevron_left "M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z",
    chevron_right "M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z",
    expand_more "M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z",
    expand_less "M12 8l-6 6 1.41 1.41L12 10.83l4.59 4.58L18 14z",
    person "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z",
    mail "M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z",
    calendar "M20 3h-1V1h-2v2H7V1H5v2H4c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 18H4V8h16v13z",
    schedule "M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z",
    keyboard "M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm9 7H8v-2h8v2zm0-4h-2v-2h2v2zm0-3h-2V8h2v2zm3 3h-2v-2h2v2zm0-3h-2V8h2v2z",
    mic "M12 14c1.66 0 2.99-1.34 2.99-3L15 5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.3-3c0 3-2.54 5.1-5.3 5.1S6.7 14 6.7 11H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c3.28-.48 6-3.3 6-6.72h-1.7z",
    info "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z",
    error "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z",
    settings "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z",
    photo "M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z",
    bookmark "M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z",
    download "M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z",
    upload "M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z",
    refresh "M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z",
    send "M2.01 21L23 12 2.01 3 2 10l15 2-15 2z",
    explore "M12 10.9c-.61 0-1.1.49-1.1 1.1s.49 1.1 1.1 1.1c.61 0 1.1-.49 1.1-1.1s-.49-1.1-1.1-1.1zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm2.19 12.19L6 18l3.81-8.19L18 6l-3.81 8.19z",
    notifications "M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z",
}
