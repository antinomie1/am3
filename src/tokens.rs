//! Material design tokens other than color: the type scale, the corner
//! scale, elevation, spring motion and state-layer opacities.

use aegle_ui::{Color, Point, Shadow};

/// One style of the Material type scale. Sizes are relative to body medium
/// (14 sp): controls scale them from their theme's font size, so the host's
/// text scale applies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    /// Font size at a 14 px theme font size.
    pub size: f32,
    /// Line height at a 14 px theme font size.
    pub line_height: f32,
    /// CSS weight.
    pub weight: u16,
    /// Letter spacing in logical pixels.
    pub tracking: f32,
}

impl TypeStyle {
    const fn new(size: f32, line_height: f32, weight: u16, tracking: f32) -> Self {
        Self {
            size,
            line_height,
            weight,
            tracking,
        }
    }

    /// The emphasized variant of the expressive type scale: heavier, for
    /// selected, unread or primary text.
    pub const fn emphasized(self) -> Self {
        Self {
            weight: if self.weight >= 500 { 700 } else { 500 },
            ..self
        }
    }

    /// Applies this role to a control's text style, scaling it from the
    /// style's size as the 14 px body size.
    pub fn apply(self, style: &mut aegle_text::TextStyle<'_>) {
        let k = style.size / 14.0;
        style.size = self.size * k;
        style.line_height = aegle_text::LineHeight::Absolute(self.line_height * k);
        style.weight = aegle_text::FontWeight::new(f32::from(self.weight));
        style.letter_spacing = self.tracking;
    }
}

/// The baseline type scale.
#[allow(missing_docs)]
pub mod typescale {
    use super::TypeStyle;
    pub const DISPLAY_LARGE: TypeStyle = TypeStyle::new(57.0, 64.0, 400, -0.25);
    pub const DISPLAY_MEDIUM: TypeStyle = TypeStyle::new(45.0, 52.0, 400, 0.0);
    pub const DISPLAY_SMALL: TypeStyle = TypeStyle::new(36.0, 44.0, 400, 0.0);
    pub const HEADLINE_LARGE: TypeStyle = TypeStyle::new(32.0, 40.0, 400, 0.0);
    pub const HEADLINE_MEDIUM: TypeStyle = TypeStyle::new(28.0, 36.0, 400, 0.0);
    pub const HEADLINE_SMALL: TypeStyle = TypeStyle::new(24.0, 32.0, 400, 0.0);
    pub const TITLE_LARGE: TypeStyle = TypeStyle::new(22.0, 28.0, 400, 0.0);
    pub const TITLE_MEDIUM: TypeStyle = TypeStyle::new(16.0, 24.0, 500, 0.15);
    pub const TITLE_SMALL: TypeStyle = TypeStyle::new(14.0, 20.0, 500, 0.1);
    pub const BODY_LARGE: TypeStyle = TypeStyle::new(16.0, 24.0, 400, 0.5);
    pub const BODY_MEDIUM: TypeStyle = TypeStyle::new(14.0, 20.0, 400, 0.25);
    pub const BODY_SMALL: TypeStyle = TypeStyle::new(12.0, 16.0, 400, 0.4);
    pub const LABEL_LARGE: TypeStyle = TypeStyle::new(14.0, 20.0, 500, 0.1);
    pub const LABEL_MEDIUM: TypeStyle = TypeStyle::new(12.0, 16.0, 500, 0.5);
    pub const LABEL_SMALL: TypeStyle = TypeStyle::new(11.0, 16.0, 500, 0.5);
}

/// A corner of the shape scale: a radius, or fully rounded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Corner {
    /// Half the shorter side.
    Full,
    /// A radius in logical pixels.
    Dp(f32),
}

impl Corner {
    /// The radius for a box whose shorter side is `side`.
    pub fn radius(self, side: f32) -> f32 {
        match self {
            Self::Full => side / 2.0,
            Self::Dp(r) => r.min(side / 2.0),
        }
    }
}

/// The corner scale.
#[allow(missing_docs)]
pub mod shape {
    use super::Corner;
    pub const NONE: Corner = Corner::Dp(0.0);
    pub const EXTRA_SMALL: Corner = Corner::Dp(4.0);
    pub const SMALL: Corner = Corner::Dp(8.0);
    pub const MEDIUM: Corner = Corner::Dp(12.0);
    pub const LARGE: Corner = Corner::Dp(16.0);
    pub const LARGE_INCREASED: Corner = Corner::Dp(20.0);
    pub const EXTRA_LARGE: Corner = Corner::Dp(28.0);
    pub const EXTRA_LARGE_INCREASED: Corner = Corner::Dp(32.0);
    pub const EXTRA_EXTRA_LARGE: Corner = Corner::Dp(48.0);
    pub const FULL: Corner = Corner::Full;
}

/// The shadow of elevation level 0–5 (0, 1, 3, 6, 8 and 12 dp), in one layer
/// close to Material's key and ambient pair.
pub fn elevation(level: f32, shadow: Color) -> Option<Shadow> {
    if level <= 0.0 {
        return None;
    }
    let dp = level_dp(level);
    let [r, g, b, _] = shadow.to_rgba();
    Some(Shadow {
        offset: Point::new(0.0, (dp * 0.5).max(1.0)),
        blur: dp * 0.6 + 1.0,
        spread: 0.0,
        color: Color::rgba(r, g, b, (48.0 + dp * 2.0).min(80.0) as u8),
    })
}

/// Elevation in dp of a fractional level, interpolating the level table.
pub fn level_dp(level: f32) -> f32 {
    const DP: [f32; 6] = [0.0, 1.0, 3.0, 6.0, 8.0, 12.0];
    let level = level.clamp(0.0, 5.0);
    let i = (level as usize).min(4);
    DP[i] + (DP[i + 1] - DP[i]) * (level - i as f32)
}

/// A spring of the motion physics system: damping ratio and stiffness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// 1 is critically damped; below 1 overshoots.
    pub damping: f32,
    /// Stiffness of a unit mass.
    pub stiffness: f32,
}

impl Spring {
    /// The equivalent Aegle spring, whose damping is a coefficient.
    pub fn aegle(self) -> aegle_motion::Spring {
        let coefficient = 2.0 * self.damping * self.stiffness.sqrt();
        aegle_motion::Spring::new(self.stiffness, coefficient).expect("positive constants")
    }

    /// A transition following this spring for its settling time.
    pub fn transition(self) -> aegle_motion::Transition {
        aegle_motion::Transition::spring(self.aegle())
    }
}

/// The expressive motion scheme, which the controls use: spatial springs
/// move and reshape, effects springs fade colors and opacity.
#[allow(missing_docs)]
pub mod motion {
    use super::Spring;
    pub const FAST_SPATIAL: Spring = Spring {
        damping: 0.6,
        stiffness: 800.0,
    };
    pub const DEFAULT_SPATIAL: Spring = Spring {
        damping: 0.8,
        stiffness: 380.0,
    };
    pub const SLOW_SPATIAL: Spring = Spring {
        damping: 0.8,
        stiffness: 200.0,
    };
    pub const FAST_EFFECTS: Spring = Spring {
        damping: 1.0,
        stiffness: 3800.0,
    };
    pub const DEFAULT_EFFECTS: Spring = Spring {
        damping: 1.0,
        stiffness: 1600.0,
    };
    pub const SLOW_EFFECTS: Spring = Spring {
        damping: 1.0,
        stiffness: 800.0,
    };

    /// The standard scheme, for calmer products.
    pub mod standard {
        use super::Spring;
        pub const FAST_SPATIAL: Spring = Spring {
            damping: 0.9,
            stiffness: 1400.0,
        };
        pub const DEFAULT_SPATIAL: Spring = Spring {
            damping: 0.9,
            stiffness: 700.0,
        };
        pub const SLOW_SPATIAL: Spring = Spring {
            damping: 0.9,
            stiffness: 300.0,
        };
    }
}

/// State-layer and disabled opacities.
#[allow(missing_docs)]
pub mod state {
    pub const HOVER: f32 = 0.08;
    pub const FOCUS: f32 = 0.10;
    pub const PRESSED: f32 = 0.10;
    pub const DRAGGED: f32 = 0.16;
    pub const DISABLED_CONTENT: f32 = 0.38;
    pub const DISABLED_CONTAINER: f32 = 0.12;
}
