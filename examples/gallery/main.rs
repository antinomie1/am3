//! Renders every am3 component, in light and dark, to PNG for the docs.
//!
//! Headless: each cell is its own retained `Ui` under an am3 theme, drawn by
//! Aegle's software renderer at a 2x scale, with Inter when the system has
//! it. Colors and springs are captured after they settle, and a held press
//! after its ripple has grown. No compositor is needed.
//!
//! `cargo run --example gallery [-- OUTPUT_DIR]`
mod shots;
mod surfaces;

use std::{
    cell::RefCell,
    fs::File,
    io::BufWriter,
    ops::Deref,
    path::Path,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use aegle_render_software::{Renderer, Surface};
use aegle_text::{Blob, GenericFamily, TextSystem};
use aegle_ui::{
    Color, Container, Modifiers, Node, Point, PointerId, PointerKind, Result, Size, Theme,
    Transition, Ui, Visit,
    scene::{Affine, Rect},
};
use aegle_widgets::Widgets;
use am3::Mode;

const SCALE: f32 = 2.0;
/// The seed of the previews: Material's baseline purple.
const SEED: Color = Color::rgb(0x67, 0x50, 0xA4);

/// Builds one cell's content into `host` and puts it into the shown state.
pub(crate) type Setup = fn(&Ui, &Container) -> Result;

fn center(node: &Node) -> Result<Point> {
    let bounds = node.bounds()?;
    Ok(Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    ))
}

pub(crate) fn hover(ui: &Ui, node: &impl Deref<Target = Node>) -> Result {
    ui.refresh()?;
    let at = center(node)?;
    ui.pointer(PointerId(1), PointerKind::Move, at, Modifiers::default())
}

pub(crate) fn press(ui: &Ui, node: &impl Deref<Target = Node>) -> Result {
    hover(ui, node)?;
    let at = center(node)?;
    let down = PointerKind::Down { clicks: 1 };
    ui.pointer(PointerId(1), down, at, Modifiers::default())
}

/// A click on the window's corner, so focus moved afterwards shows no
/// ring, as after pointer input.
pub(crate) fn pointer_mode(ui: &Ui) -> Result {
    let at = Point::new(1.0, 1.0);
    let down = PointerKind::Down { clicks: 1 };
    ui.pointer(PointerId(1), down, at, Modifiers::default())?;
    ui.pointer(PointerId(1), PointerKind::Up, at, Modifiers::default())
}

/// Lays out, settles and records one cell.
fn cell(
    fonts: &Rc<RefCell<TextSystem>>,
    theme: Theme,
    size: Size,
    caption: &str,
    setup: Setup,
) -> Result<Ui> {
    let ui = Ui::with_fonts(fonts.clone(), theme)?;
    ui.set_default_transition(Some(Transition::default()))?;
    ui.resize(size)?;
    let root = ui.root();
    root.set_padding(12.0)?;
    root.set_gap(10.0)?;
    let label = root.text(caption)?;
    label.set_font_size(11.0)?;
    label.set_foreground(theme.muted)?;
    let host = root.column()?;
    host.set_gap(8.0)?;
    host.set_align_items(Some(aegle_ui::Align::Start))?;
    setup(&ui, &host)?;
    ui.refresh()?;
    ui.dispatch_callbacks()?;
    ui.refresh()?;
    // Settle the engine's color transitions and the controls' own springs.
    ui.advance_animations(Duration::from_secs(2))?;
    ui.run_frame(Instant::now() + Duration::from_secs(2))?;
    ui.refresh()?;
    Ok(ui)
}

/// Renders the cells side by side, a light row over a dark row.
fn shot(
    fonts: &Rc<RefCell<TextSystem>>,
    dir: &Path,
    name: &str,
    size: (f32, f32),
    cells: &[(&str, Setup)],
) -> Result {
    let cell_size = Size::new(size.0, size.1);
    let themes = [am3::theme(SEED, Mode::Light), am3::theme(SEED, Mode::Dark)];
    let (width, height) = (
        (cell_size.width * cells.len() as f32 * SCALE) as u32,
        (cell_size.height * themes.len() as f32 * SCALE) as u32,
    );
    let mut pixels = vec![0; width as usize * height as usize * 4];
    let mut surface = Surface::new(&mut pixels, width, height)?;
    let mut renderer = Renderer::default();
    let mut frame = renderer.begin_frame(&mut surface, Color::TRANSPARENT);
    for (row, theme) in themes.into_iter().enumerate() {
        let y = row as f32 * cell_size.height;
        // The row's background.
        let band = aegle_ui::scene::RoundedRect::new(
            Rect::new(0.0, y * SCALE, width as f32, cell_size.height * SCALE),
            0.0,
        )?;
        let mut builder = aegle_ui::scene::SceneBuilder::new();
        builder.fill(band, theme.background)?;
        frame.draw(&builder.finish()?, Affine::IDENTITY)?;
        for (index, (caption, setup)) in cells.iter().enumerate() {
            let ui = cell(fonts, theme, cell_size, caption, *setup)?;
            let x = index as f32 * cell_size.width;
            let place = Affine::translation(x, y)?.then(Affine::scale(SCALE, SCALE)?)?;
            let clip = Rect::new(
                x * SCALE,
                y * SCALE,
                cell_size.width * SCALE,
                cell_size.height * SCALE,
            );
            let cell_clip = |inner: Option<Rect>| {
                let Some(inner) = inner else { return clip };
                let left = (x + inner.origin.x.max(0.0)) * SCALE;
                let top = (y + inner.origin.y.max(0.0)) * SCALE;
                let right = (x + (inner.origin.x + inner.size.width).min(cell_size.width)) * SCALE;
                let bottom =
                    (y + (inner.origin.y + inner.size.height).min(cell_size.height)) * SCALE;
                Rect::new(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
            };
            ui.visit_scenes(|visit| {
                match visit {
                    Visit::Scene {
                        scene,
                        transform,
                        clip,
                    } => {
                        frame.draw_clipped(scene, transform.then(place)?, Some(cell_clip(clip)))?
                    }
                    Visit::PushLayer(layer) => {
                        let clip = cell_clip(layer.clip());
                        frame.push_layer(&layer.then(place)?.with_clip(Some(clip))?)?
                    }
                    Visit::PopLayer => frame.pop_layer()?,
                }
                Ok(())
            })?;
        }
    }
    drop(frame);
    let path = dir.join(format!("{name}.png"));
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(&path)?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(surface.data())?;
    println!("wrote {}", path.display());
    Ok(())
}

/// Inter from the system when present, else Aegle's bundled test font.
fn fonts() -> Result<TextSystem> {
    let mut text = TextSystem::new();
    let dir = Path::new("/usr/share/fonts/opentype/inter");
    let mut files: Vec<_> = ["Regular", "Medium", "SemiBold", "Bold"]
        .iter()
        .map(|face| dir.join(format!("Inter-{face}.otf")))
        .filter(|path| path.exists())
        .collect();
    if files.is_empty() {
        files.push(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../aegle/tests/assets/aegle-test-cjk.otf"
            )
            .into(),
        );
    }
    let mut ids = Vec::new();
    for file in files {
        let families = text.register_fonts(Blob::new(Arc::new(std::fs::read(file)?)))?;
        ids.extend(families.iter().map(|(id, _)| *id));
    }
    ids.dedup();
    text.collection_mut()
        .set_generic_families(GenericFamily::SansSerif, ids.into_iter());
    Ok(text)
}

fn main() -> Result {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/docs/images").into());
    let dir = Path::new(&output);
    std::fs::create_dir_all(dir)?;
    let fonts = Rc::new(RefCell::new(fonts()?));
    for (name, size, cells) in shots::ALL.iter().chain(surfaces::SURFACES) {
        shot(&fonts, dir, name, *size, cells)?;
    }
    Ok(())
}
