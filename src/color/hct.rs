//! HCT: CAM16 hue and chroma with CIE L* tone, the color space of Material's
//! dynamic color, following Material Color Utilities.
//!
//! Only what tonal palettes need: hue and chroma of an sRGB color, and the
//! sRGB color of a hue, chroma and tone, reducing chroma until it fits the
//! gamut (the reference bisection solver).

use aegle_ui::Color;

const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41233895, 0.35762064, 0.18051042],
    [0.2126, 0.7152, 0.0722],
    [0.01932141, 0.11916382, 0.95034478],
];
const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [
        3.2413774792388685,
        -1.5376652402851851,
        -0.49885366846268053,
    ],
    [-0.9691452513005321, 1.8758853451067872, 0.04156585616912061],
    [
        0.05562093689691305,
        -0.20395524564742123,
        1.0571799111220335,
    ],
];
const WHITE: [f64; 3] = [95.047, 100.0, 108.883];
const XYZ_TO_CAM: [[f64; 3]; 3] = [
    [0.401288, 0.650173, -0.051461],
    [-0.250268, 1.204414, 0.045854],
    [-0.002079, 0.048952, 0.953127],
];
const CAM_TO_XYZ: [[f64; 3]; 3] = [
    [1.86206786, -1.01125463, 0.14918677],
    [0.38752654, 0.62144744, -0.00897398],
    [-0.01584150, -0.03412294, 1.04996444],
];

fn mul(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    m.map(|row| row[0] * v[0] + row[1] * v[1] + row[2] * v[2])
}

/// An sRGB channel 0–255 as linear 0–100.
fn linearized(channel: u8) -> f64 {
    let n = f64::from(channel) / 255.0;
    100.0
        * if n <= 0.040449936 {
            n / 12.92
        } else {
            ((n + 0.055) / 1.055).powf(2.4)
        }
}

/// A linear channel 0–100 as sRGB 0–255.
fn delinearized(linear: f64) -> u8 {
    let n = linear / 100.0;
    let v = if n <= 0.0031308 {
        n * 12.92
    } else {
        1.055 * n.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

fn lab_f(t: f64) -> f64 {
    const E: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;
    if t > E {
        t.cbrt()
    } else {
        (KAPPA * t + 16.0) / 116.0
    }
}

fn lab_inv_f(ft: f64) -> f64 {
    const E: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;
    let cube = ft * ft * ft;
    if cube > E {
        cube
    } else {
        (116.0 * ft - 16.0) / KAPPA
    }
}

/// Relative luminance 0–100 of an L* tone.
pub(crate) fn y_from_lstar(lstar: f64) -> f64 {
    100.0 * lab_inv_f((lstar + 16.0) / 116.0)
}

fn xyz(color: Color) -> [f64; 3] {
    let [r, g, b, _] = color.to_rgba();
    mul(&SRGB_TO_XYZ, [r, g, b].map(linearized))
}

/// CIE L* of an sRGB color: Material's tone.
pub fn tone(color: Color) -> f64 {
    116.0 * lab_f(xyz(color)[1] / 100.0) - 16.0
}

fn from_xyz(v: [f64; 3]) -> Color {
    let [r, g, b] = mul(&XYZ_TO_SRGB, v).map(delinearized);
    Color::rgb(r, g, b)
}

fn gray(lstar: f64) -> Color {
    let y = y_from_lstar(lstar);
    from_xyz([y * WHITE[0] / 100.0, y, y * WHITE[2] / 100.0])
}

/// The default viewing conditions: sRGB white, 50 L* background, average
/// surround.
struct Viewing {
    n: f64,
    aw: f64,
    nbb: f64,
    c: f64,
    nc: f64,
    rgb_d: [f64; 3],
    fl: f64,
    fl_root: f64,
    z: f64,
}

impl Viewing {
    fn standard() -> Self {
        let adapting = 200.0 / std::f64::consts::PI * y_from_lstar(50.0) / 100.0;
        let rgb_w = mul(&XYZ_TO_CAM, WHITE);
        let f = 1.0;
        let c = 0.69;
        let d = (f * (1.0 - (1.0 / 3.6) * ((-adapting - 42.0) / 92.0).exp())).clamp(0.0, 1.0);
        let rgb_d = rgb_w.map(|w| d * (100.0 / w) + 1.0 - d);
        let k = 1.0 / (5.0 * adapting + 1.0);
        let k4 = k * k * k * k;
        let fl = k4 * adapting + 0.1 * (1.0 - k4) * (1.0 - k4) * (5.0 * adapting).cbrt();
        let n = y_from_lstar(50.0) / WHITE[1];
        let z = 1.48 + n.sqrt();
        let nbb = 0.725 / n.powf(0.2);
        let rgb_a = [0, 1, 2].map(|i| {
            let factor = (fl * rgb_d[i] * rgb_w[i] / 100.0).powf(0.42);
            400.0 * factor / (factor + 27.13)
        });
        let aw = (2.0 * rgb_a[0] + rgb_a[1] + 0.05 * rgb_a[2]) * nbb;
        Self {
            n,
            aw,
            nbb,
            c,
            nc: f,
            rgb_d,
            fl,
            fl_root: fl.powf(0.25),
            z,
        }
    }
}

thread_local! {
    static VIEWING: Viewing = Viewing::standard();
}

/// A CAM16 color: lightness J, chroma and hue in degrees, with the UCS
/// coordinates used for distances.
#[derive(Clone, Copy, Debug)]
struct Cam {
    hue: f64,
    chroma: f64,
    j: f64,
    jstar: f64,
    astar: f64,
    bstar: f64,
}

impl Cam {
    fn ucs(hue: f64, chroma: f64, j: f64, vc: &Viewing) -> Self {
        let m = chroma * vc.fl_root;
        let jstar = 1.7 * j / (1.0 + 0.007 * j);
        let mstar = (1.0 + 0.0228 * m).ln() / 0.0228;
        let radians = hue.to_radians();
        Self {
            hue,
            chroma,
            j,
            jstar,
            astar: mstar * radians.cos(),
            bstar: mstar * radians.sin(),
        }
    }

    fn from_color(color: Color) -> Self {
        VIEWING.with(|vc| {
            let rgb_c = mul(&XYZ_TO_CAM, xyz(color));
            let rgb_a = [0, 1, 2].map(|i| {
                let d = vc.rgb_d[i] * rgb_c[i];
                let af = (vc.fl * d.abs() / 100.0).powf(0.42);
                d.signum() * 400.0 * af / (af + 27.13)
            });
            let [r, g, b] = rgb_a;
            let a = (11.0 * r - 12.0 * g + b) / 11.0;
            let bb = (r + g - 2.0 * b) / 9.0;
            let u = (20.0 * r + 20.0 * g + 21.0 * b) / 20.0;
            let p2 = (40.0 * r + 20.0 * g + b) / 20.0;
            let hue = bb.atan2(a).to_degrees().rem_euclid(360.0);
            let ac = p2 * vc.nbb;
            let j = 100.0 * (ac / vc.aw).powf(vc.c * vc.z);
            let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
            let e_hue = 0.25 * ((hue_prime.to_radians() + 2.0).cos() + 3.8);
            let p1 = 50000.0 / 13.0 * e_hue * vc.nc * vc.nbb;
            let t = p1 * a.hypot(bb) / (u + 0.305);
            let alpha = t.powf(0.9) * (1.64 - 0.29f64.powf(vc.n)).powf(0.73);
            Self::ucs(hue, alpha * (j / 100.0).sqrt(), j, vc)
        })
    }

    fn from_jch(j: f64, chroma: f64, hue: f64) -> Self {
        VIEWING.with(|vc| Self::ucs(hue, chroma, j, vc))
    }

    fn distance(&self, other: &Self) -> f64 {
        let (dj, da, db) = (
            self.jstar - other.jstar,
            self.astar - other.astar,
            self.bstar - other.bstar,
        );
        1.41 * (dj * dj + da * da + db * db).sqrt().powf(0.63)
    }

    /// The sRGB color, with channels clipped to the gamut.
    fn color(&self) -> Color {
        VIEWING.with(|vc| {
            let alpha = if self.chroma == 0.0 || self.j == 0.0 {
                0.0
            } else {
                self.chroma / (self.j / 100.0).sqrt()
            };
            let t = (alpha / (1.64 - 0.29f64.powf(vc.n)).powf(0.73)).powf(1.0 / 0.9);
            let radians = self.hue.to_radians();
            let e_hue = 0.25 * ((radians + 2.0).cos() + 3.8);
            let ac = vc.aw * (self.j / 100.0).powf(1.0 / vc.c / vc.z);
            let p1 = e_hue * (50000.0 / 13.0) * vc.nc * vc.nbb;
            let p2 = ac / vc.nbb;
            let (sin, cos) = radians.sin_cos();
            let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * cos + 108.0 * t * sin);
            let (a, b) = (gamma * cos, gamma * sin);
            let rgb_a = [
                (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0,
                (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0,
                (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0,
            ];
            let rgb_f = [0, 1, 2].map(|i| {
                let v = rgb_a[i];
                let base = (27.13 * v.abs() / (400.0 - v.abs())).max(0.0);
                v.signum() * (100.0 / vc.fl) * base.powf(1.0 / 0.42) / vc.rgb_d[i]
            });
            from_xyz(mul(&CAM_TO_XYZ, rgb_f))
        })
    }
}

/// CAM16 hue in degrees and chroma of an sRGB color.
pub fn hue_chroma(color: Color) -> (f64, f64) {
    let cam = Cam::from_color(color);
    (cam.hue, cam.chroma)
}

/// The sRGB color nearest to `hue`, `chroma` and `tone`: the tone is kept
/// and chroma reduced until the color is in gamut.
pub fn solve(hue: f64, chroma: f64, tone: f64) -> Color {
    if chroma < 1.0 || tone.round() <= 0.0 || tone.round() >= 100.0 {
        return gray(tone);
    }
    let hue = hue.rem_euclid(360.0);
    let (mut low, mut high, mut mid) = (0.0, chroma, chroma);
    let mut answer = None;
    let mut first = true;
    while (low - high).abs() >= 0.4 {
        let found = find_by_j(hue, mid, tone);
        if first {
            if let Some(cam) = found {
                return cam.color();
            }
            first = false;
        } else if let Some(cam) = found {
            answer = Some(cam);
            low = mid;
        } else {
            high = mid;
        }
        mid = low + (high - low) / 2.0;
    }
    answer.map_or_else(|| gray(tone), |cam| cam.color())
}

/// A CAM16 color of `hue` and `chroma` whose clipped sRGB value has `tone`,
/// if one stays close to the requested hue and chroma.
fn find_by_j(hue: f64, chroma: f64, tone: f64) -> Option<Cam> {
    let (mut low, mut high) = (0.0f64, 100.0f64);
    let (mut best_dl, mut best_de) = (1000.0, 1000.0);
    let mut best = None;
    while (low - high).abs() > 0.01 {
        let mid = low + (high - low) / 2.0;
        let clipped = Cam::from_jch(mid, chroma, hue).color();
        let clipped_tone = self::tone(clipped);
        let dl = (tone - clipped_tone).abs();
        if dl < 0.2 {
            let cam = Cam::from_color(clipped);
            let de = cam.distance(&Cam::from_jch(cam.j, cam.chroma, hue));
            if de <= 1.0 && de <= best_de {
                (best_dl, best_de) = (dl, de);
                best = Some(cam);
            }
        }
        if best_dl == 0.0 && best_de == 0.0 {
            break;
        }
        if clipped_tone < tone {
            low = mid;
        } else {
            high = mid;
        }
    }
    best
}
