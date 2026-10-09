//! Material color: HCT, the color scheme and the color-role tokens.

pub mod hct;
mod scheme;

pub use scheme::{Mode, Role, Scheme, theme};

use aegle_ui::{Color, Theme, Token};

/// `over` at `alpha` composited on opaque `base`, as a state layer or a
/// disabled container over a surface.
pub fn blend(base: Color, over: Color, alpha: f32) -> Color {
    let [r0, g0, b0, a0] = base.to_rgba();
    let [r1, g1, b1, _] = over.to_rgba();
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * alpha).round() as u8;
    Color::rgba(mix(r0, r1), mix(g0, g1), mix(b0, b1), a0)
}

/// `color` with its alpha multiplied by `alpha`, as content at 38 % while
/// disabled or a state layer over a transparent container.
pub fn alpha(color: Color, alpha: f32) -> Color {
    let [r, g, b, a] = color.to_rgba();
    Color::rgba(r, g, b, (f32::from(a) * alpha).round() as u8)
}

macro_rules! defaults {
    ($($field:ident,)*) => {
        /// The token default of each role: its color in the theme's scheme.
        const DEFAULTS: &[fn(&Theme) -> Color] = &[
            $(|theme| Scheme::of(theme).$field,)*
        ];
    };
}

defaults! {
    primary, on_primary, primary_container, on_primary_container, primary_fixed,
    primary_fixed_dim, on_primary_fixed, on_primary_fixed_variant, inverse_primary,
    secondary, on_secondary, secondary_container, on_secondary_container, secondary_fixed,
    secondary_fixed_dim, on_secondary_fixed, on_secondary_fixed_variant, tertiary,
    on_tertiary, tertiary_container, on_tertiary_container, tertiary_fixed,
    tertiary_fixed_dim, on_tertiary_fixed, on_tertiary_fixed_variant, error, on_error,
    error_container, on_error_container, surface, on_surface, surface_variant,
    on_surface_variant, surface_dim, surface_bright, surface_container_lowest,
    surface_container_low, surface_container, surface_container_high,
    surface_container_highest, inverse_surface, inverse_on_surface, outline,
    outline_variant, shadow, scrim, surface_tint,
}

impl Role {
    /// The color token of this role, such as `am3.color.primary`, whose
    /// default follows the theme's scheme. Bind application content to it
    /// with `bind_color` or `token("am3.color.primary")` in markup.
    pub fn token(self) -> Token<Color> {
        aegle_ui::register_token(self.token_name(), DEFAULTS[self as usize])
            .expect("am3 token names are valid and unique")
    }
}

/// Registers the color token of every role on this thread, so markup can
/// name one with `token("am3.color.primary-container")` before Rust code
/// has asked for it. [`theme`] and [`crate::elements`] call it.
pub fn register_tokens() {
    for role in Role::ALL {
        role.token();
    }
}
