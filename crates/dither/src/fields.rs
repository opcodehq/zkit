//! Credits: many of these fields are adapted from the examples in dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT; the chart fields follow the look of dither-kit by ripgrim (https://tripwire.sh/dither-kit).

use crate::finite_tone;
use std::f32::consts::{PI, TAU};

macro_rules! presets {
    ($($name:ident => ($slug:literal, $label:literal, $group:literal)),* $(,)?) => {
        /// Built-in tone fields. Charts use sample data; use a custom field for application data.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub enum Preset { #[default] Aurora, $($name),* }
        impl Preset {
            pub const ALL: &'static [Self] = &[Self::Aurora, $(Self::$name),*];
            pub fn slug(self) -> &'static str { match self { Self::Aurora => "aurora", $(Self::$name => $slug),* } }
            pub fn label(self) -> &'static str { match self { Self::Aurora => "Aurora", $(Self::$name => $label),* } }
            pub fn group(self) -> &'static str { match self { Self::Aurora => "Backgrounds", $(Self::$name => $group),* } }
            pub fn from_slug(slug: &str) -> Option<Self> { Self::ALL.iter().copied().find(|p| p.slug() == slug) }
        }
    }
}
presets! {
    Gradient => ("gradient", "Gradient", "Backgrounds"),
    Glow => ("glow", "Glow", "Backgrounds"),
    Mesh => ("mesh", "Mesh gradient", "Backgrounds"),
    Noise => ("noise", "Noise field", "Backgrounds"),
    Waves => ("waves", "Waves", "Backgrounds"),
    Plasma => ("plasma", "Plasma", "Backgrounds"),
    Beams => ("beams", "Beams", "Backgrounds"),
    Silk => ("silk", "Silk", "Backgrounds"),
    Smoke => ("smoke", "Smoke", "Backgrounds"),
    Contours => ("contours", "Contours", "Backgrounds"),
    Ripples => ("ripples", "Ripples", "Backgrounds"),
    Ridgelines => ("ridgelines", "Ridgelines", "Backgrounds"),
    Metaballs => ("metaballs", "Metaballs", "Backgrounds"),
    Tunnel => ("tunnel", "Tunnel", "Backgrounds"),
    RetroGrid => ("retro-grid", "Retro grid", "Backgrounds"),
    Eclipse => ("eclipse", "Eclipse", "Backgrounds"),
    Horizon => ("horizon", "Horizon", "Backgrounds"),
    Starfield => ("starfield", "Starfield", "Backgrounds"),
    Halftone => ("halftone", "Halftone", "Backgrounds"),
    Globe => ("globe", "Globe", "Backgrounds"),
    DotMap => ("dot-map", "Dot map", "Backgrounds"),
    Orb => ("orb", "Orb", "Backgrounds"),
    Fade => ("fade", "Fade", "Backgrounds"),
    Area => ("area", "Area chart", "Charts"),
    Bar => ("bar", "Bar chart", "Charts"),
    Line => ("line", "Line chart", "Charts"),
    Sparkline => ("sparkline", "Sparkline", "Charts"),
    Heatmap => ("heatmap", "Heatmap", "Charts"),
    Donut => ("donut", "Donut chart", "Charts"),
    Radar => ("radar", "Radar chart", "Charts"),
    Gauge => ("gauge", "Gauge", "Charts"),
    Progress => ("progress", "Progress bar", "Charts"),
    Uptime => ("uptime", "Uptime bar", "Charts"),
    Stat => ("stat", "Stat trend", "Charts"),
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn hash(x: f32, y: f32) -> f32 {
    ((x * 127.1 + y * 311.7).sin() * 43758.547).rem_euclid(1.)
}
fn noise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (u, v) = (smooth(0., 1., x - ix), smooth(0., 1., y - iy));
    let a = hash(ix, iy) * (1. - u) + hash(ix + 1., iy) * u;
    let b = hash(ix, iy + 1.) * (1. - u) + hash(ix + 1., iy + 1.) * u;
    a * (1. - v) + b * v
}
fn fbm(x: f32, y: f32) -> f32 {
    noise(x, y) * 0.57
        + noise(x * 2.03 + 4.1, y * 2.03) * 0.28
        + noise(x * 4.11, y * 4.11 + 7.) * 0.15
}
fn ring(distance: f32, radius: f32, thickness: f32) -> f32 {
    1. - smooth(thickness, thickness + 0.012, (distance - radius).abs())
}
// Sample data for the chart fields, 0..1.
const SERIES: [f32; 12] = [
    0.42, 0.55, 0.5, 0.66, 0.6, 0.74, 0.7, 0.82, 0.76, 0.88, 0.84, 0.95,
];
const BARS: [f32; 8] = [0.48, 0.62, 0.55, 0.78, 0.7, 0.86, 0.66, 0.94];
const SPOKES: [f32; 6] = [0.7, 0.9, 0.45, 0.95, 0.35, 0.6];
const SHARES: [f32; 5] = [0.45, 0.3, 0.16, 0.06, 0.03];

/// Catmull-Rom through `data` at `u` in 0..1, kept between neighbours.
fn spline(data: &[f32], u: f32) -> f32 {
    let n = data.len();
    let x = u.clamp(0., 1.) * (n - 1) as f32;
    let i = (x as usize).min(n - 2);
    let t = x - i as f32;
    let at = |k: isize| data[k.clamp(0, n as isize - 1) as usize];
    let (p0, p1, p2, p3) = (
        at(i as isize - 1),
        at(i as isize),
        at(i as isize + 1),
        at(i as isize + 2),
    );
    let y = 0.5
        * (2. * p1
            + (p2 - p0) * t
            + (2. * p0 - 5. * p1 + 4. * p2 - p3) * t * t
            + (3. * p1 - p0 - 3. * p2 + p3) * t * t * t);
    y.clamp(p1.min(p2), p1.max(p2))
}

/// The chart body: solid towards the base, dotted near the edge.
fn body(depth: f32) -> f32 {
    0.1 + 0.9 * smooth(0., 0.6, depth)
}

/// An area or line over `data` inside the plot box `left..right`, `top..base`.
fn plot_area(
    data: &[f32],
    u: f32,
    v: f32,
    (left, right, top, base): (f32, f32, f32, f32),
    ribbon: bool,
) -> f32 {
    if !(left..right).contains(&u) || v > base {
        return 0.;
    }
    let edge = base - (base - top) * spline(data, (u - left) / (right - left));
    let d = v - edge;
    if (-0.004..0.008).contains(&d) {
        return 1.;
    }
    if ribbon {
        return if (0. ..0.06).contains(&d) {
            0.85 * (1. - d / 0.06)
        } else {
            0.
        };
    }
    if d > 0. {
        body(d / (base - edge).max(1e-3))
    } else {
        0.
    }
}

impl Preset {
    /// Sample a normalized field. Aspect keeps radial forms circular.
    pub fn tone(self, u: f32, v: f32, time: f32, aspect: f32) -> f32 {
        let aspect = if aspect.is_finite() {
            aspect.clamp(0.1, 10.)
        } else {
            1.
        };
        let (x, y) = ((u - 0.5) * aspect, v - 0.5);
        let r = x.hypot(y);
        let t = time * 0.25;
        let tone = match self {
            Self::Gradient => u * 0.7 + v * 0.3,
            Self::Glow => (-((x - 0.35).powi(2) + (y + 0.15).powi(2)) * 5.).exp(),
            Self::Mesh => {
                let mut s = 0.;
                for i in 0..4 {
                    let i = i as f32;
                    let dx = x - (t * 0.3 + i * 2.).sin() * aspect * 0.3;
                    let dy = y - (t * 0.2 + i * 1.7).cos() * 0.3;
                    s += (-(dx * dx + dy * dy) * 12.).exp() * 0.65;
                }
                s
            }
            Self::Aurora => {
                let mut light = 0.;
                for i in 0..3 {
                    let i = i as f32;
                    let edge = 0.12
                        + i * 0.12
                        + 0.16 * (x * 2. + t * 0.3 + i * 1.8).sin()
                        + 0.05 * (x * 7. - t * 0.2 + i).sin();
                    let above = edge - y;
                    let curtain = if above > 0. {
                        (-above * (5. + i * 2.)).exp()
                    } else {
                        (above * 45.).exp()
                    };
                    let rays = 0.25 + 0.75 * fbm(x * 12. + i * 8., t * 0.2 + i);
                    light += (curtain * rays + (-above.abs() * 34.).exp() * 0.5) * (1. - i * 0.2);
                }
                1. - (-light * 1.5).exp()
            }
            Self::Noise => fbm(x * 4. + t * 0.2, y * 4. + t * 0.12),
            Self::Waves => {
                let mut s = 0.;
                for i in 0..5 {
                    let i = i as f32;
                    let wave = 0.05 + i * 0.11 + 0.08 * (x * 3. + t + i * 0.7).sin();
                    s += (-(y - wave).abs() * (18. + i * 4.)).exp() * (0.8 - i * 0.12);
                }
                s
            }
            Self::Plasma => {
                let (px, py) = (x * 4.2, y * 4.2);
                let a = (px * 0.9 + t * 0.31).sin()
                    + (py * 1.1 - t * 0.23).sin()
                    + ((px * 0.6 + py * 0.8) * 1.2 + t * 0.17).sin()
                    + ((px - (t * 0.13).sin() * 1.6).hypot(py - (t * 0.11).cos() * 1.2) * 1.4
                        - t * 0.29)
                        .sin();
                (0.5 + 0.5 * (a * PI * 0.375).sin()).powf(1.7)
            }
            Self::Beams => {
                let angle = x.atan2(v + 0.1);
                let rays = (angle * 35. + t * 0.2).sin() * 0.5 + 0.5;
                rays.powf(4.) * (1. - v * 0.8)
            }
            Self::Silk => {
                (0.5 + 0.5 * (x * 9. + (y * 5. + t * 0.3).sin() * 2. + t).sin()).powf(4.)
                    * (0.5 + 0.5 * fbm(x * 2., y * 2.))
            }
            Self::Smoke => {
                let n = fbm(x * 3. + fbm(x * 2., y * 2. - t) * 2., y * 3. - t * 0.4);
                n.powf(1.5) * (1. - (u - 0.5).abs() * 1.2)
            }
            Self::Contours => {
                let h = fbm(x * 2.3 + t * 0.04, y * 2.3);
                let line = ((h * 14. + 0.5).fract() - 0.5).abs();
                (1. - smooth(0.02, 0.12, line)) * 0.8 + h * 0.12
            }
            Self::Ripples => {
                let a = ((x + 0.35).hypot(y + 0.05) * 28. - t * 2.).sin();
                let b = ((x - 0.4).hypot(y - 0.2) * 23. - t * 1.6).sin();
                ((a + b) * 0.25 + 0.5).powf(2.)
            }
            Self::Ridgelines => {
                let mut value = 0.;
                for i in 0..12 {
                    let i = i as f32;
                    let ridge = -0.35 + i * 0.065
                        - (-x * x * 5.).exp() * (0.15 + noise(x * 4. + i, i + t * 0.1) * 0.22);
                    if y > ridge {
                        value = (-(y - ridge) * 140.).exp();
                    }
                }
                value
            }
            Self::Metaballs => {
                let mut sum = 0.;
                for i in 0..4 {
                    let i = i as f32;
                    let dx = x - (t * 0.6 + i * 2.).sin() * 0.45;
                    let dy = y - (t * 0.4 + i * 1.7).cos() * 0.28;
                    sum += 0.035 / (dx * dx + dy * dy + 0.01);
                }
                smooth(0.5, 1.5, sum)
            }
            Self::Tunnel => {
                (0.5 + 0.5 * (r.max(0.01).ln() * 20. + t * 2.).sin()) * (1. - (-r * 8.).exp())
            }
            Self::RetroGrid => {
                if y > 0.08 {
                    let perspective = 1. / (y + 0.05);
                    let gx = (x * perspective * 2.).fract().abs();
                    let gy = (perspective + t).fract();
                    ((1. - smooth(0., 0.08, gx)).max(1. - smooth(0., 0.08, gy))) * (1. - v * 0.35)
                } else {
                    let sun = (x * x + (y + 0.16).powi(2)).sqrt();
                    if sun < 0.2 && (v * 70.).fract() > 0.25 {
                        0.85
                    } else {
                        0.02
                    }
                }
            }
            Self::Eclipse => {
                if r < 0.27 {
                    0.
                } else {
                    (-(r - 0.27) * 22.).exp() * 0.95
                }
            }
            Self::Horizon => {
                let edge = 0.16 + x * x * 0.22;
                let delta = y - edge;
                if delta > 0. {
                    0.
                } else {
                    let glow = (-delta.abs() * 17.).exp() * 0.85;
                    let drift = 0.5 + 0.5 * (x * 4. + t * 0.4).sin();
                    let haze = (-delta.abs() * 6.).exp() * (0.07 + drift * 0.12);
                    let star = hash((u * 160.).floor(), (v * 100.).floor());
                    glow + haze + if star > 0.998 { 0.35 } else { 0. }
                }
            }
            Self::Starfield => {
                let sx = (u * 100.).floor();
                let sy = (v * 65.).floor();
                let h = hash(sx, sy);
                if h > 0.975 {
                    (0.4 + 0.6 * (t + h * 30.).sin().abs()) * smooth(0.975, 1., h)
                } else {
                    0.
                }
            }
            Self::Halftone => {
                let sx = (u * 30.).fract() - 0.5;
                let sy = (v * 20.).fract() - 0.5;
                1. - smooth(0.05 + u * 0.4, 0.08 + u * 0.4, sx.hypot(sy))
            }
            Self::Globe | Self::Orb => {
                if r > 0.4 {
                    0.
                } else {
                    let z = (0.16 - r * r).sqrt();
                    let light = (-x * 0.6 - y * 0.5 + z * 0.8) / 0.4;
                    if self == Self::Globe {
                        let longitude = x.atan2(z) + t * 0.2;
                        let latitude = (y / 0.4).asin();
                        let land = fbm(longitude * 4. + 4., latitude * 4. + 4.);
                        finite_tone(light) * (if land > 0.48 { 0.95 } else { 0.13 })
                            + ring(r, 0.395, 0.005) * 0.25
                    } else {
                        finite_tone(light).powf(1.5)
                    }
                }
            }
            Self::DotMap => {
                let n = fbm(u * 8. + 2., v * 6. + 4.);
                let edge = smooth(0., 0.12, u)
                    * smooth(1., 0.88, u)
                    * smooth(0., 0.15, v)
                    * smooth(1., 0.85, v);
                if n > 0.48 { edge * 0.65 } else { 0. }
            }
            Self::Fade => (1. - v).powf(2.),
            Self::Area => plot_area(&SERIES, u, v, (0.06, 0.94, 0.14, 0.88), false),
            Self::Sparkline => plot_area(&SERIES, u, v, (0., 1., 0.25, 0.8), false),
            Self::Stat => plot_area(&SERIES, u, v, (0., 1., 0.55, 1.), false),
            Self::Line => {
                let other: [f32; 12] = SERIES.map(|s| s * 0.55);
                plot_area(&SERIES, u, v, (0.06, 0.94, 0.14, 0.88), true).max(plot_area(
                    &other,
                    u,
                    v,
                    (0.06, 0.94, 0.14, 0.88),
                    true,
                ))
            }
            Self::Bar => {
                let (left, right, top, base) = (0.06, 0.94, 0.12, 0.88);
                if !(left..right).contains(&u) || v > base {
                    return 0.;
                }
                let slot = (u - left) / (right - left) * BARS.len() as f32;
                let i = (slot as usize).min(BARS.len() - 1);
                if !(0.12..0.88).contains(&slot.fract()) {
                    return 0.;
                }
                let edge = base - (base - top) * BARS[i];
                match v - edge {
                    d if d < 0. => 0.,
                    d if d < 0.01 => 1.,
                    d => body(d / (base - edge)),
                }
            }
            Self::Heatmap => {
                let (cols, rows) = (24., 7.);
                let (a, b) = ((u * cols).floor(), (v * rows).floor());
                if (u * cols).fract() < 0.12 || (v * rows).fract() < 0.12 {
                    return 0.;
                }
                // Busier mid-week and mid-day; four levels like a contribution graph.
                let day = 1. - ((b - 2.5) / 3.5).abs().powi(2) * 0.6;
                let hour = (-((a - 13.) / 6.).powi(2)).exp();
                let level = (day * hour * 0.8 + hash(a, b) * 0.35).clamp(0., 1.);
                0.12 + 0.88 * (level * 4.).ceil() / 4.
            }
            Self::Donut => {
                let rr = r / 0.42;
                if !(0.62..1.).contains(&rr) {
                    return 0.;
                }
                let angle = (x.atan2(-y) / TAU).rem_euclid(1.);
                let mut start = 0.;
                for (i, share) in SHARES.iter().enumerate() {
                    let end = start + share;
                    if angle < end {
                        if angle - start < 0.006 || end - angle < 0.006 {
                            return 0.;
                        }
                        let weight = [1., 0.72, 0.52, 0.38, 0.26][i];
                        return weight * body((rr - 0.62) / 0.38);
                    }
                    start = end;
                }
                0.
            }
            Self::Radar => {
                let rr = r / 0.42;
                if rr > 1. {
                    return 0.;
                }
                let n = SPOKES.len();
                let angle = x.atan2(-y).rem_euclid(TAU);
                let sector = angle / TAU * n as f32;
                let i = sector as usize % n;
                // Distance to the polygon edge between spokes i and i + 1.
                let (ta, tb) = (i as f32 / n as f32 * TAU, (i + 1) as f32 / n as f32 * TAU);
                let (a, b) = (SPOKES[i], SPOKES[(i + 1) % n]);
                let (ax, ay, bx, by) = (a * ta.sin(), a * ta.cos(), b * tb.sin(), b * tb.cos());
                let (px, py) = (angle.sin(), angle.cos());
                let edge = (ax * (by - ay) - ay * (bx - ax)) / (px * (by - ay) - py * (bx - ax));
                // Hexagonal rings.
                let hex = rr * ((angle.rem_euclid(TAU / n as f32)) - PI / n as f32).cos()
                    / (PI / n as f32).cos();
                let on_ring = (hex * 4.).fract() < 0.035 || (hex * 4.).fract() > 0.985;
                let on_spoke = sector.fract() < 0.012 || sector.fract() > 0.988;
                let grid = if (on_ring || on_spoke) && hex <= 1. {
                    0.45
                } else {
                    0.
                };
                if (rr - edge).abs() < 0.012 {
                    1.
                } else if rr < edge {
                    body(0.3 + 0.7 * rr / edge).max(grid)
                } else {
                    grid
                }
            }
            Self::Gauge => {
                let rr = r / 0.4;
                if y > 0.1 || !(0.7..1.).contains(&rr) {
                    return 0.;
                }
                let frac = 1. - (-y).atan2(x).clamp(0., PI) / PI;
                if (frac * 40.).fract() > 0.72 {
                    return 0.;
                }
                if frac <= 0.68 {
                    body((rr - 0.7) / 0.3)
                } else {
                    0.14
                }
            }
            Self::Progress => {
                if !(0.44..0.56).contains(&v) || !(0.08..0.92).contains(&u) {
                    return 0.;
                }
                let p = (u - 0.08) / 0.84;
                match p {
                    p if p <= 0.66 => body(0.3 + 0.7 * p / 0.66),
                    p if p < 0.68 => 1.,
                    _ => 0.12,
                }
            }
            Self::Uptime => {
                let day = (u * 36.).floor();
                if (u * 36.).fract() < 0.2 || !(0.3..0.7).contains(&v) {
                    return 0.;
                }
                let depth = (v - 0.3) / 0.4;
                if hash(day, 1.) > 0.1 {
                    body(depth)
                } else {
                    0.18
                }
            }
        };
        finite_tone(tone)
    }
}
