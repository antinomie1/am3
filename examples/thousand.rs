//! Measures a 1000-control headless screen of am3 components with a
//! counting allocator, like Aegle's `thousand` example: 125 rows of text,
//! a filled button, checkbox, switch, slider, filter chip, icon button and
//! progress indicator. Prints the first frame (build, layout, paint
//! recording), steady refreshes (idle, a value change, a hover move), a
//! theme switch, scheme generation, the same screen loaded from markup,
//! and whether a settled screen still asks for frames. Run in release:
//!
//! `cargo run --example thousand --release`

// The counting allocator forwards to `System`, which needs unsafe.
#![allow(unsafe_code)]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use aegle_loader::Program;
use aegle_text::{Blob, GenericFamily, TextSystem};
use aegle_ui::{Color, Modifiers, Point, PointerId, PointerKind, Result, Size, Ui};
use aegle_widgets::Widgets;
use am3::*;

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        LIVE.fetch_add(layout.size(), Ordering::Relaxed);
        // SAFETY: forwards the caller's layout contract to the system allocator.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: `ptr` was allocated by `System` with this layout.
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        LIVE.fetch_add(size, Ordering::Relaxed);
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: as for `dealloc`, with the caller's new size.
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const SEED: Color = Color::rgb(0x67, 0x50, 0xA4);
const RUNS: u32 = 200;

fn counted<T>(run: impl FnOnce() -> Result<T>) -> Result<(T, Duration, usize)> {
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let start = Instant::now();
    let value = run()?;
    let time = start.elapsed();
    Ok((value, time, ALLOCATIONS.load(Ordering::Relaxed) - before))
}

fn frame(ui: &Ui) -> Result {
    ui.refresh()?;
    ui.visit_scenes(|_| Ok(()))
}

/// Runs `step` `RUNS` times and prints its average cost.
fn steady(name: &str, mut step: impl FnMut(u32) -> Result) -> Result {
    let (_, time, allocations) = counted(|| (0..RUNS).try_for_each(&mut step))?;
    println!(
        "{name}: {:?}, {} allocations",
        time / RUNS,
        allocations as f64 / f64::from(RUNS)
    );
    Ok(())
}

/// Lets every transition and spring finish: moves the animation clock,
/// counted from `start`, 3 s on.
fn settle(ui: &Ui, start: Instant, clock: &mut Duration) -> Result {
    *clock = (*clock).max(start.elapsed()) + Duration::from_secs(3);
    ui.advance_animations(*clock)?;
    ui.run_frame(start + *clock)?;
    frame(ui)
}

const MARKUP: &str = r#"
ScrollView {
    state blocks: list<int> = [0, 1, 2, 3, 4]
    state rows: list<int> = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]
    for block in blocks {
        for row in rows {
            Row {
                MdText { text: "Row " + str(block * 25 + row) }
                MdButton { text: "Open" }
                MdCheckbox { text: "Done"; checked: row % 2 == 0 }
                MdSwitch { checked: row % 3 == 0 }
                MdSlider { value: 0.5 }
                MdChip { text: "Tag"; kind: filter }
                MdIconButton { icon: "star"; label: "Star" }
                MdProgress { value: 0.5 }
            }
        }
    }
}
"#;

fn main() -> Result {
    let mut text = TextSystem::new();
    let font = include_bytes!("../tests/assets/aegle-test-cjk.otf");
    let families = text.register_fonts(Blob::new(Arc::new(font.as_slice())))?;
    let ids = families.iter().map(|(id, _)| *id);
    text.collection_mut()
        .set_generic_families(GenericFamily::SansSerif, ids);
    let fonts = Rc::new(RefCell::new(text));

    let seeds = [
        SEED,
        Color::rgb(0x00, 0x6A, 0x6A),
        Color::rgb(0x8B, 0x50, 0x00),
    ];
    let (_, time, allocations) = counted(|| {
        for run in 0..1000 {
            std::hint::black_box(Scheme::from_seed(seeds[run % 3], Mode::Dark));
        }
        Ok(())
    })?;
    println!(
        "scheme from seed: {:?}, {} allocations",
        time / 1000,
        allocations / 1000
    );
    let light = theme(SEED, Mode::Light);
    Scheme::of(&light);
    let (_, time, _) = counted(|| {
        for _ in 0..1000 {
            std::hint::black_box(Scheme::of(std::hint::black_box(&light)));
        }
        Ok(())
    })?;
    println!("cached Scheme::of: {:?}", time / 1000);

    // The same screen from Aegle's default controls, for comparison.
    let live = LIVE.load(Ordering::Relaxed);
    let (plain, time, allocations) = counted(|| {
        let ui = Ui::with_fonts(fonts.clone(), aegle_ui::Theme::light())?;
        let list = ui.root().scroll_view()?;
        for row in 0..125 {
            let line = list.row()?;
            line.text(&format!("Row {row}"))?;
            line.button("Open")?;
            line.check_box("Done", row % 2 == 0)?;
            line.switch("", row % 3 == 0)?;
            line.slider(0.0, 1.0, 0.5)?;
            line.button("Tag")?;
            line.button("Star")?;
            line.progress(0.0, 1.0, 0.5)?;
        }
        ui.resize(Size::new(1280.0, 800.0))?;
        frame(&ui)?;
        Ok(ui)
    })?;
    println!(
        "Aegle default controls: first frame {time:?}, {allocations} allocations, {} KiB live",
        (LIVE.load(Ordering::Relaxed) - live) / 1024
    );
    drop(plain);

    let live = LIVE.load(Ordering::Relaxed);
    let (ui, sliders, buttons) = {
        let (built, time, allocations) = counted(|| {
            let ui = Ui::with_fonts(fonts.clone(), light)?;
            let list = ui.root().scroll_view()?;
            let (mut sliders, mut buttons) = (Vec::new(), Vec::new());
            for row in 0..125 {
                let line = list.row()?;
                line.set_align_items(Some(aegle_ui::Align::Center))?;
                Text::new(
                    &line,
                    tokens::typescale::BODY_MEDIUM,
                    Role::on_surface,
                    &format!("Row {row}"),
                )?;
                buttons.push(Button::new(&line, ButtonStyle::Filled, "Open")?);
                Checkbox::new(&line, "Done", row % 2 == 0)?;
                Switch::new(&line, "", row % 3 == 0)?;
                sliders.push(Slider::new(&line, 0.0, 1.0, 0.5)?);
                Chip::new(&line, ChipKind::Filter, "Tag")?;
                IconButton::new(&line, IconStyle::Standard, icons::star(), "Star")?;
                Progress::linear(&line, Some(0.5))?;
            }
            ui.resize(Size::new(1280.0, 800.0))?;
            frame(&ui)?;
            Ok((ui, sliders, buttons))
        })?;
        println!("first frame: {time:?}, {allocations} allocations");
        built
    };
    let (start, mut clock) = (Instant::now(), Duration::ZERO);
    settle(&ui, start, &mut clock)?;
    println!(
        "live heap after building and settling: {} KiB; settled screen wants frames: {}",
        (LIVE.load(Ordering::Relaxed) - live) / 1024,
        ui.wants_frames()?
    );

    steady("idle refresh", |_| frame(&ui))?;
    steady("value change refresh", |run| {
        sliders[run as usize % 16].set_value(f64::from(run % 10) / 10.0)?;
        frame(&ui)
    })?;
    let mut targets = Vec::new();
    for button in &buttons[..8] {
        let bounds = button.bounds()?;
        targets.push(Point::new(bounds.origin.x + 4.0, bounds.origin.y + 4.0));
    }
    steady("hover refresh", |run| {
        let at = targets[run as usize % targets.len()];
        ui.pointer(PointerId(1), PointerKind::Move, at, Modifiers::default())?;
        frame(&ui)
    })?;
    ui.pointer(
        PointerId(1),
        PointerKind::Leave,
        Point::new(0.0, 0.0),
        Modifiers::default(),
    )?;
    settle(&ui, start, &mut clock)?;

    let themes = [theme(SEED, Mode::Dark), light];
    let (_, time, allocations) = counted(|| {
        for run in 0..20 {
            ui.set_theme(themes[run % 2])?;
            frame(&ui)?;
        }
        Ok(())
    })?;
    println!(
        "theme switch and frame: {:?}, {} allocations",
        time / 20,
        allocations / 20
    );
    settle(&ui, start, &mut clock)?;
    println!(
        "settled after switching, wants frames: {}",
        ui.wants_frames()?
    );
    drop(ui);

    let (_, time, allocations) = counted(|| {
        let source = MARKUP.to_owned();
        let program = Program::from_sources("thousand.aegle", &am3::elements(), &mut |_| {
            Ok(source.clone())
        })?;
        let ui = Ui::with_fonts(fonts.clone(), light)?;
        let view = program.build(&ui.root())?;
        ui.resize(Size::new(1280.0, 800.0))?;
        frame(&ui)?;
        drop(view);
        Ok(())
    })?;
    println!("markup check, build and first frame: {time:?}, {allocations} allocations");
    Ok(())
}
