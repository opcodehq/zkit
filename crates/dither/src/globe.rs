//! A rotating dithered globe: land from the built-in mask, a lit terminator,
//! a thin atmosphere, pulsing markers and dotted arcs between places.
//! Renders at cell resolution into a transparent raster.
//!
//! Credits: adapted from the dithered globe in dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT.

use crate::{Matrix, Raster, land};
use std::f32::consts::PI;

const RADIANS: f32 = PI / 180.;
/// Share of the shorter side taken by the globe's radius.
const RADIUS_SHARE: f32 = 0.42;
/// Seconds per marker pulse.
const PULSE: f32 = 2.6;

/// A place on Earth, `(latitude, longitude)` in degrees.
pub type LatLon = (f32, f32);

/// Globe settings. Angles are in degrees; `speed` is degrees per second.
#[derive(Clone, Debug, PartialEq)]
pub struct Globe {
    pub speed: f32,
    /// Longitude facing the viewer at time zero.
    pub start: f32,
    /// How far the axis leans towards the viewer.
    pub tilt: f32,
    /// Where the light comes from, clockwise from the top.
    pub light: f32,
    /// Brightness of the oceans, 0 to 1.
    pub ocean: f32,
    pub atmosphere: bool,
    pub matrix: Matrix,
    pub markers: Vec<LatLon>,
    pub arcs: Vec<(LatLon, LatLon)>,
}

impl Default for Globe {
    fn default() -> Self {
        Self {
            speed: 6.,
            start: 10.,
            tilt: 18.,
            light: 300.,
            ocean: 0.18,
            atmosphere: true,
            matrix: Matrix::Bayer4,
            markers: Vec::new(),
            arcs: Vec::new(),
        }
    }
}

type Vec3 = [f32; 3];

struct View {
    cos_s: f32,
    sin_s: f32,
    cos_t: f32,
    sin_t: f32,
}

impl View {
    fn new(spin: f32, tilt: f32) -> Self {
        Self {
            cos_s: spin.cos(),
            sin_s: spin.sin(),
            cos_t: tilt.cos(),
            sin_t: tilt.sin(),
        }
    }
    /// View space to the globe's frame: undo the tilt, then the spin.
    fn to_world(&self, [x, y, z]: Vec3) -> Vec3 {
        let y1 = y * self.cos_t - z * self.sin_t;
        let z1 = y * self.sin_t + z * self.cos_t;
        [
            x * self.cos_s + z1 * self.sin_s,
            y1,
            -x * self.sin_s + z1 * self.cos_s,
        ]
    }
    fn to_view(&self, [x, y, z]: Vec3) -> Vec3 {
        let x1 = x * self.cos_s - z * self.sin_s;
        let z1 = x * self.sin_s + z * self.cos_s;
        [
            x1,
            y * self.cos_t + z1 * self.sin_t,
            -y * self.sin_t + z1 * self.cos_t,
        ]
    }
}

fn from_lat_lon((lat, lon): LatLon) -> Vec3 {
    let (phi, lambda) = (lat * RADIANS, lon * RADIANS);
    [
        phi.cos() * lambda.sin(),
        phi.sin(),
        phi.cos() * lambda.cos(),
    ]
}

fn light_vector(degrees: f32) -> Vec3 {
    let angle = degrees * RADIANS;
    let (x, y) = (angle.sin() * 0.75, angle.cos() * 0.75);
    let length = (x * x + y * y + 0.62 * 0.62).sqrt();
    [x / length, y / length, 0.62 / length]
}

/// A point on the great circle from `a` to `b`, lifted in the middle.
fn arc_point(a: Vec3, b: Vec3, angle: f32, s: f32) -> Vec3 {
    let sin = if angle.sin() == 0. { 1. } else { angle.sin() };
    let wa = ((1. - s) * angle).sin() / sin;
    let wb = (s * angle).sin() / sin;
    let lift = 1. + (PI * s).sin() * (0.06 + angle * 0.1);
    [
        (a[0] * wa + b[0] * wb) * lift,
        (a[1] * wa + b[1] * wb) * lift,
        (a[2] * wa + b[2] * wb) * lift,
    ]
}

impl Globe {
    /// Render one frame. Land and atmosphere use `ink`; markers and arcs use
    /// `accent`. Paper cells are fully transparent.
    pub fn render(
        &self,
        columns: usize,
        rows: usize,
        time: f32,
        ink: [u8; 3],
        accent: [u8; 3],
    ) -> Result<Raster, crate::ImageError> {
        let len = crate::image::checked_len(columns, rows)?;
        let mut pixels = vec![0u8; len];
        let time = if time.is_finite() { time } else { 0. };
        let (cx, cy) = (columns as f32 / 2., rows as f32 / 2.);
        let radius = columns.min(rows) as f32 * RADIUS_SHARE;
        let view = View::new(
            (self.start + self.speed * time) * RADIANS,
            -self.tilt * RADIANS,
        );
        let light = light_vector(self.light);
        let mut put = |x: i64, y: i64, color: [u8; 3]| {
            if x < 0 || y < 0 || x >= columns as i64 || y >= rows as i64 {
                return;
            }
            let at = (y as usize * columns + x as usize) * 4;
            pixels[at..at + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
        };
        let threshold =
            |x: i64, y: i64| self.matrix.threshold(x.max(0) as usize, y.max(0) as usize);

        for y in 0..rows {
            for x in 0..columns {
                let nx = (x as f32 + 0.5 - cx) / radius;
                let ny = (cy - y as f32 - 0.5) / radius;
                let r2 = nx * nx + ny * ny;
                let tone = if r2 <= 1. {
                    let normal = [nx, ny, (1. - r2).sqrt()];
                    let lit = normal[0] * light[0] + normal[1] * light[1] + normal[2] * light[2];
                    // Lambert light with a short soft edge past the terminator.
                    let shade = ((lit + 0.08) / 1.08).clamp(0., 1.);
                    let [wx, wy, wz] = view.to_world(normal);
                    let lat = wy.clamp(-1., 1.).asin() / RADIANS;
                    let lon = wx.atan2(wz) / RADIANS;
                    if land::coverage(lat, lon) >= 0.5 {
                        0.07 + 0.93 * shade
                    } else {
                        let rim = (1. - normal[2]).powi(3);
                        shade * (self.ocean * (0.5 + rim) + rim * 0.1)
                    }
                } else if self.atmosphere && r2 < 1.16 {
                    // A thin glow, strongest towards the light.
                    let strength = 1. - (r2.sqrt() - 1.) / 0.077;
                    let facing = ((nx * light[0] + ny * light[1]) / r2.sqrt()).max(0.);
                    strength.max(0.).powi(2) * (0.12 + 0.5 * facing)
                } else {
                    0.
                };
                if tone > threshold(x as i64, y as i64) {
                    put(x as i64, y as i64, ink);
                }
            }
        }

        let project = |point: Vec3| {
            let [x, y, z] = view.to_view(point);
            let visible = z > 0. || x * x + y * y > 1.;
            (cx + x * radius, cy - y * radius, z, visible)
        };

        // Arcs: dotted, with a bright stretch travelling along each.
        for (index, &(from, to)) in self.arcs.iter().enumerate() {
            let (a, b) = (from_lat_lon(from), from_lat_lon(to));
            let angle = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).min(1.).acos();
            let steps = ((angle * radius * 1.3).ceil() as usize).max(12);
            let head = (time / 3.2 + index as f32 * 0.41).rem_euclid(1.5);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let (px, py, _, visible) = project(arc_point(a, b, angle, t));
                let lit = t <= head && t > head - 0.25;
                if visible && (lit || s % 3 == 0) {
                    put(px.round() as i64, py.round() as i64, accent);
                }
            }
        }

        // Markers: a 3-cell dot with a dithered ring that grows and fades.
        for (index, &place) in self.markers.iter().enumerate() {
            let (px, py, z, _) = project(from_lat_lon(place));
            if z <= 0.05 {
                continue;
            }
            let phase = (time / PULSE + index as f32 * 0.37).rem_euclid(1.);
            let ring = 2. + phase * (radius * 0.1).max(6.);
            let fade = (1. - phase).powf(1.5);
            let steps = (ring * 6.).ceil() as usize;
            for s in 0..steps {
                let angle = s as f32 / steps as f32 * PI * 2.;
                let x = (px + angle.cos() * ring).round() as i64;
                // Flattened by depth, so rings near the limb lie on the surface.
                let y = (py + angle.sin() * ring * z).round() as i64;
                if fade > threshold(x, y) {
                    put(x, y, accent);
                }
            }
            let (x0, y0) = (px.round() as i64 - 1, py.round() as i64 - 1);
            for dy in 0..3 {
                for dx in 0..3 {
                    put(x0 + dx, y0 + dy, accent);
                }
            }
        }

        Ok(Raster {
            width: columns,
            height: rows,
            pixels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn globe_is_round_transparent_outside_and_draws_markers() {
        let globe = Globe {
            markers: vec![(51.51, -0.13)],
            start: 0.,
            tilt: 0.,
            ..Globe::default()
        };
        let raster = globe
            .render(120, 80, 0., [10, 20, 30], [200, 100, 0])
            .unwrap();
        let alpha = |x: usize, y: usize| raster.pixels()[(y * 120 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "corners are paper");
        assert_eq!(alpha(119, 79), 0);
        let accent = raster
            .pixels()
            .chunks_exact(4)
            .filter(|p| p[..3] == [200, 100, 0])
            .count();
        assert!(accent >= 9, "London is in view and marked");
        let inked = raster
            .pixels()
            .chunks_exact(4)
            .filter(|p| p[3] == 255)
            .count();
        assert!(inked > 400, "the lit hemisphere draws land");
    }
}
