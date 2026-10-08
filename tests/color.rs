//! The scheme matches Material Color Utilities for the baseline seed, every
//! mode keeps Material's contrast, and a theme maps back to its own scheme.

use aegle_ui::{Color, Theme};
use am3::{Mode, Role, Scheme, theme};

fn close(a: Color, hex: u32) -> bool {
    let [r, g, b, _] = a.to_rgba();
    let want = [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8];
    [r, g, b].iter().zip(want).all(|(x, y)| x.abs_diff(y) <= 2)
}

/// WCAG contrast ratio.
fn contrast(a: Color, b: Color) -> f64 {
    let luminance = |c: Color| {
        let [r, g, b, _] = c.to_rgba();
        let l = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * l(r) + 0.7152 * l(g) + 0.0722 * l(b)
    };
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[test]
fn baseline_seed_matches_material_color_utilities() {
    // Material Theme Builder's tonal-spot scheme for the baseline seed.
    let seed = Color::rgb(0x67, 0x50, 0xA4);
    let light = Scheme::from_seed(seed, Mode::Light);
    for (color, want) in [
        (light.primary, 0x65558F),
        (light.secondary, 0x625B71),
        (light.tertiary, 0x7D5260),
        (light.surface, 0xFDF7FF),
        (light.on_surface, 0x1D1B20),
        (light.surface_container, 0xF2ECF4),
        (light.primary_container, 0xE9DDFF),
        (light.error, 0xBA1A1A),
    ] {
        assert!(close(color, want), "{color:?} != {want:06X}");
    }
    let dark = Scheme::from_seed(seed, Mode::Dark);
    for (color, want) in [
        (dark.primary, 0xCFBDFE),
        (dark.surface, 0x141218),
        (dark.on_surface, 0xE6E0E9),
        (dark.error, 0xFFB4AB),
    ] {
        assert!(close(color, want), "{color:?} != {want:06X}");
    }
}

#[test]
fn every_mode_keeps_text_contrast() {
    for seed in [0x6750A4, 0x006E1C, 0xB3261E, 0x0061A4, 0x808080] {
        let seed = Color::rgb((seed >> 16) as u8, (seed >> 8) as u8, seed as u8);
        for (mode, minimum) in [
            (Mode::Light, 4.5),
            (Mode::Dark, 4.5),
            (Mode::LightHighContrast, 7.0),
            (Mode::DarkHighContrast, 7.0),
        ] {
            let s = Scheme::from_seed(seed, mode);
            for (on, under) in [
                (s.on_primary, s.primary),
                (s.on_primary_container, s.primary_container),
                (s.on_secondary_container, s.secondary_container),
                (s.on_tertiary_container, s.tertiary_container),
                (s.on_error, s.error),
                (s.on_surface, s.surface),
                (s.on_surface, s.surface_container_highest),
                (s.on_surface_variant, s.surface_container_highest),
                (s.inverse_on_surface, s.inverse_surface),
            ] {
                let ratio = contrast(on, under);
                assert!(ratio >= minimum, "{mode:?} {on:?} on {under:?}: {ratio:.2}");
            }
        }
    }
}

#[test]
fn themes_map_back_to_their_scheme() {
    let seed = Color::rgb(0x00, 0x6E, 0x1C);
    for mode in [
        Mode::Light,
        Mode::Dark,
        Mode::LightHighContrast,
        Mode::DarkHighContrast,
    ] {
        let t = theme(seed, mode);
        assert!(t.validate().is_ok());
        let (want, got) = (Scheme::from_seed(seed, mode), Scheme::of(&t));
        for &role in Role::ALL {
            let [a, b] = [want.role(role), got.role(role)].map(Color::to_rgba);
            let off = a.iter().zip(b).map(|(x, y)| x.abs_diff(y)).max().unwrap();
            assert!(off <= 2, "{mode:?} {role:?}: {a:?} vs {b:?}");
        }
    }
    // Any theme gets a coherent scheme: Aegle's own dark theme is dark, its
    // high-contrast theme is dark high contrast.
    let dark = Scheme::of(&Theme::dark());
    let want = Scheme::from_seed(Theme::dark().accent, Mode::Dark);
    assert_eq!(dark.surface, want.surface);
    let high = Scheme::of(&Theme::high_contrast());
    let want = Scheme::from_seed(Theme::high_contrast().accent, Mode::DarkHighContrast);
    assert_eq!(high, want);
}
