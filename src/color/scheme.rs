//! The Material color scheme: 47 color roles from one hue, in light, dark
//! and high-contrast variants, derived from an Aegle [`Theme`].

use std::cell::RefCell;

use aegle_ui::{Color, Theme};

use super::{blend, hct};

/// Which scheme variant [`theme`] builds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Light surfaces, dark content.
    Light,
    /// Dark surfaces, light content.
    Dark,
    /// Light, with roles moved apart for at least 7:1 text contrast.
    LightHighContrast,
    /// Dark, with roles moved apart for at least 7:1 text contrast.
    DarkHighContrast,
}

impl Mode {
    fn dark(self) -> bool {
        matches!(self, Self::Dark | Self::DarkHighContrast)
    }
    fn high_contrast(self) -> bool {
        matches!(self, Self::LightHighContrast | Self::DarkHighContrast)
    }
}

macro_rules! roles {
    ($($field:ident $name:literal,)*) => {
        /// Every Material 3 color role, as resolved for one theme.
        ///
        /// `background` and `on_background` are Material aliases of
        /// `surface` and `on_surface` and are not repeated.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[allow(missing_docs)]
        pub struct Scheme {
            $(pub $field: Color,)*
        }

        /// A color role by name, for tokens and markup.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[allow(missing_docs, non_camel_case_types)]
        pub enum Role {
            $($field,)*
        }

        impl Role {
            /// Every role, in declaration order.
            pub const ALL: &[Role] = &[$(Role::$field,)*];
            /// The token name, such as `am3.color.on-primary`.
            pub const fn token_name(self) -> &'static str {
                match self {
                    $(Role::$field => concat!("am3.color.", $name),)*
                }
            }
        }

        impl Scheme {
            /// The color of `role`.
            pub const fn role(&self, role: Role) -> Color {
                match role {
                    $(Role::$field => self.$field,)*
                }
            }
        }
    };
}

roles! {
    primary "primary",
    on_primary "on-primary",
    primary_container "primary-container",
    on_primary_container "on-primary-container",
    primary_fixed "primary-fixed",
    primary_fixed_dim "primary-fixed-dim",
    on_primary_fixed "on-primary-fixed",
    on_primary_fixed_variant "on-primary-fixed-variant",
    inverse_primary "inverse-primary",
    secondary "secondary",
    on_secondary "on-secondary",
    secondary_container "secondary-container",
    on_secondary_container "on-secondary-container",
    secondary_fixed "secondary-fixed",
    secondary_fixed_dim "secondary-fixed-dim",
    on_secondary_fixed "on-secondary-fixed",
    on_secondary_fixed_variant "on-secondary-fixed-variant",
    tertiary "tertiary",
    on_tertiary "on-tertiary",
    tertiary_container "tertiary-container",
    on_tertiary_container "on-tertiary-container",
    tertiary_fixed "tertiary-fixed",
    tertiary_fixed_dim "tertiary-fixed-dim",
    on_tertiary_fixed "on-tertiary-fixed",
    on_tertiary_fixed_variant "on-tertiary-fixed-variant",
    error "error",
    on_error "on-error",
    error_container "error-container",
    on_error_container "on-error-container",
    surface "surface",
    on_surface "on-surface",
    surface_variant "surface-variant",
    on_surface_variant "on-surface-variant",
    surface_dim "surface-dim",
    surface_bright "surface-bright",
    surface_container_lowest "surface-container-lowest",
    surface_container_low "surface-container-low",
    surface_container "surface-container",
    surface_container_high "surface-container-high",
    surface_container_highest "surface-container-highest",
    inverse_surface "inverse-surface",
    inverse_on_surface "inverse-on-surface",
    outline "outline",
    outline_variant "outline-variant",
    shadow "shadow",
    scrim "scrim",
    surface_tint "surface-tint",
}

/// One hue and chroma, sampled at tones.
#[derive(Clone, Copy)]
struct Palette {
    hue: f64,
    chroma: f64,
}

impl Palette {
    fn tone(self, tone: f64) -> Color {
        hct::solve(self.hue, self.chroma, tone)
    }
}

/// Tones of one accent palette's roles: role, on, container, on container.
struct Accent([f64; 4]);

/// Tones of the neutral roles.
struct Neutral {
    surface: f64,
    on_surface: f64,
    dim: f64,
    bright: f64,
    containers: [f64; 5],
    inverse_surface: f64,
    inverse_on_surface: f64,
    variant: f64,
    on_variant: f64,
    outline: f64,
    outline_variant: f64,
    inverse_primary: f64,
}

fn tones(mode: Mode) -> (Accent, Neutral) {
    match (mode.dark(), mode.high_contrast()) {
        (false, false) => (
            Accent([40.0, 100.0, 90.0, 30.0]),
            Neutral {
                surface: 98.0,
                on_surface: 10.0,
                dim: 87.0,
                bright: 98.0,
                containers: [100.0, 96.0, 94.0, 92.0, 90.0],
                inverse_surface: 20.0,
                inverse_on_surface: 95.0,
                variant: 90.0,
                on_variant: 30.0,
                outline: 50.0,
                outline_variant: 80.0,
                inverse_primary: 80.0,
            },
        ),
        (true, false) => (
            Accent([80.0, 20.0, 30.0, 90.0]),
            Neutral {
                surface: 6.0,
                on_surface: 90.0,
                dim: 6.0,
                bright: 24.0,
                containers: [4.0, 10.0, 12.0, 17.0, 22.0],
                inverse_surface: 90.0,
                inverse_on_surface: 20.0,
                variant: 30.0,
                on_variant: 80.0,
                outline: 60.0,
                outline_variant: 30.0,
                inverse_primary: 40.0,
            },
        ),
        (false, true) => (
            Accent([25.0, 100.0, 35.0, 100.0]),
            Neutral {
                surface: 98.0,
                on_surface: 0.0,
                dim: 87.0,
                bright: 98.0,
                containers: [100.0, 96.0, 94.0, 92.0, 90.0],
                inverse_surface: 20.0,
                inverse_on_surface: 100.0,
                variant: 90.0,
                on_variant: 20.0,
                outline: 30.0,
                outline_variant: 40.0,
                inverse_primary: 90.0,
            },
        ),
        (true, true) => (
            Accent([92.0, 10.0, 75.0, 0.0]),
            Neutral {
                surface: 6.0,
                on_surface: 100.0,
                dim: 6.0,
                bright: 24.0,
                containers: [4.0, 10.0, 12.0, 17.0, 22.0],
                inverse_surface: 90.0,
                inverse_on_surface: 0.0,
                variant: 30.0,
                on_variant: 95.0,
                outline: 85.0,
                outline_variant: 70.0,
                inverse_primary: 30.0,
            },
        ),
    }
}

impl Scheme {
    /// The tonal-spot scheme of `hue` (CAM16 degrees): the hue at moderate
    /// chroma for primary, low chroma for secondary, the hue turned 60° for
    /// tertiary, near-gray neutrals and a fixed red error palette.
    pub fn from_hue(hue: f64, mode: Mode) -> Self {
        let palette = |hue: f64, chroma| Palette {
            hue: hue.rem_euclid(360.0),
            chroma,
        };
        let (primary, secondary, tertiary) = (
            palette(hue, 36.0),
            palette(hue, 16.0),
            palette(hue + 60.0, 24.0),
        );
        let (neutral, variant, error) = (palette(hue, 6.0), palette(hue, 8.0), palette(25.0, 84.0));
        let (Accent([role, on, container, on_container]), n) = tones(mode);
        // Fixed roles keep their tones in every mode.
        let fixed = |p: Palette| [p.tone(90.0), p.tone(80.0), p.tone(10.0), p.tone(30.0)];
        let [
            primary_fixed,
            primary_fixed_dim,
            on_primary_fixed,
            on_primary_fixed_variant,
        ] = fixed(primary);
        let [
            secondary_fixed,
            secondary_fixed_dim,
            on_secondary_fixed,
            on_secondary_fixed_variant,
        ] = fixed(secondary);
        let [
            tertiary_fixed,
            tertiary_fixed_dim,
            on_tertiary_fixed,
            on_tertiary_fixed_variant,
        ] = fixed(tertiary);
        let containers = n.containers.map(|t| neutral.tone(t));
        Self {
            primary: primary.tone(role),
            on_primary: primary.tone(on),
            primary_container: primary.tone(container),
            on_primary_container: primary.tone(on_container),
            primary_fixed,
            primary_fixed_dim,
            on_primary_fixed,
            on_primary_fixed_variant,
            inverse_primary: primary.tone(n.inverse_primary),
            secondary: secondary.tone(role),
            on_secondary: secondary.tone(on),
            secondary_container: secondary.tone(container),
            on_secondary_container: secondary.tone(on_container),
            secondary_fixed,
            secondary_fixed_dim,
            on_secondary_fixed,
            on_secondary_fixed_variant,
            tertiary: tertiary.tone(role),
            on_tertiary: tertiary.tone(on),
            tertiary_container: tertiary.tone(container),
            on_tertiary_container: tertiary.tone(on_container),
            tertiary_fixed,
            tertiary_fixed_dim,
            on_tertiary_fixed,
            on_tertiary_fixed_variant,
            error: error.tone(role),
            on_error: error.tone(on),
            error_container: error.tone(container),
            on_error_container: error.tone(on_container),
            surface: neutral.tone(n.surface),
            on_surface: neutral.tone(n.on_surface),
            surface_variant: variant.tone(n.variant),
            on_surface_variant: variant.tone(n.on_variant),
            surface_dim: neutral.tone(n.dim),
            surface_bright: neutral.tone(n.bright),
            surface_container_lowest: containers[0],
            surface_container_low: containers[1],
            surface_container: containers[2],
            surface_container_high: containers[3],
            surface_container_highest: containers[4],
            inverse_surface: neutral.tone(n.inverse_surface),
            inverse_on_surface: neutral.tone(n.inverse_on_surface),
            outline: variant.tone(n.outline),
            outline_variant: variant.tone(n.outline_variant),
            shadow: Color::BLACK,
            scrim: Color::BLACK,
            surface_tint: primary.tone(role),
        }
    }

    /// The scheme of a seed color: its hue, whatever its chroma.
    pub fn from_seed(seed: Color, mode: Mode) -> Self {
        Self::from_hue(hct::hue_chroma(seed).0, mode)
    }

    /// The scheme an Aegle theme stands for: the hue of its accent, dark
    /// when its background is, high contrast when its foreground is black
    /// or white. Themes from [`theme`] map back to their own scheme; any
    /// other theme gets a coherent one. Recent results are cached per
    /// thread, so skins call this freely.
    pub fn of(theme: &Theme) -> Self {
        let key = [theme.accent, theme.background, theme.foreground];
        CACHE.with_borrow_mut(|cache| {
            if let Some(index) = cache.iter().position(|(k, _)| *k == key) {
                let entry = cache.remove(index);
                cache.insert(0, entry);
                return cache[0].1;
            }
            let dark = hct::tone(theme.background) < 50.0;
            let ink = hct::tone(theme.foreground);
            let mode = match (dark, if dark { ink > 97.5 } else { ink < 2.5 }) {
                (false, false) => Mode::Light,
                (true, false) => Mode::Dark,
                (false, true) => Mode::LightHighContrast,
                (true, true) => Mode::DarkHighContrast,
            };
            let scheme = Self::from_seed(theme.accent, mode);
            cache.truncate(CACHED - 1);
            cache.insert(0, (key, scheme));
            scheme
        })
    }
}

/// Schemes kept per thread: enough for light, dark, high contrast and a
/// local override in use at once.
const CACHED: usize = 6;

thread_local! {
    static CACHE: RefCell<Vec<([Color; 3], Scheme)>> = const { RefCell::new(Vec::new()) };
}

/// An Aegle theme carrying the Material scheme of `seed` in `mode`: its
/// colors are the scheme's surface, on-surface, primary and outline roles,
/// so built-in Aegle controls match, and [`Scheme::of`] recovers the full
/// scheme from it. Metrics follow Material: 14 px body text, 40 px
/// controls, 12 px medium corners.
///
/// Pass the light, dark and high-contrast results to the host's themes;
/// the host switches them with the system preference. Also registers the
/// roles' color tokens, see [`super::register_tokens`].
pub fn theme(seed: Color, mode: Mode) -> Theme {
    super::register_tokens();
    let s = Scheme::from_seed(seed, mode);
    Theme {
        background: s.surface,
        surface: s.surface_container,
        foreground: s.on_surface,
        muted: s.on_surface_variant,
        accent: s.primary,
        border: s.outline,
        hover: blend(s.surface, s.on_surface, 0.08),
        pressed: blend(s.surface, s.on_surface, 0.10),
        selection: s.primary_container,
        font_size: 14.0,
        padding: 12.0,
        gap: 8.0,
        radius: 12.0,
        control_height: 40.0,
    }
}
