//! Dithered loading indicators: a ring with a sweeping tail, three breathing
//! dots, a pulse of expanding rings and a bar with a travelling head.
//!
//! Credits: the four loaders are adapted from dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT.

use crate::{Matrix, Raster};
use std::f32::consts::TAU;

/// Tone of the resting track, so the shape stays legible between beats.
const TRACK: f32 = 0.13;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoaderVariant {
    #[default]
    Ring,
    Dots,
    Pulse,
    Bar,
}

impl LoaderVariant {
    pub const ALL: [Self; 4] = [Self::Ring, Self::Dots, Self::Pulse, Self::Bar];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Ring => "ring",
            Self::Dots => "dots",
            Self::Pulse => "pulse",
            Self::Bar => "bar",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.slug() == slug)
    }

    /// The frame held under reduced motion, in seconds.
    pub fn still_time(self) -> f32 {
        match self {
            Self::Ring => 0.4,
            Self::Dots => 0.2,
            Self::Pulse => 0.5,
            Self::Bar => 0.9,
        }
    }

    /// Grid in cells for a loader `height` cells tall: square for the ring
    /// and pulse, wide for the dots and bar.
    pub fn grid(self, height: usize) -> (usize, usize) {
        let rows = height.max(6);
        match self {
            Self::Bar => (rows * 3, (rows as f32 / 5.).round().max(2.) as usize),
            Self::Dots => (rows * 2, (rows as f32 * 0.62).round() as usize),
            Self::Ring | Self::Pulse => (rows, rows),
        }
    }

    /// Tone of cell `(x, y)` on a `cols × rows` grid at `t` seconds.
    pub fn tone(self, cols: usize, rows: usize, x: usize, y: usize, t: f32) -> f32 {
        let (cols, rows) = (cols as f32, rows as f32);
        let (x, y) = (x as f32 + 0.5, y as f32 + 0.5);
        let fract = |v: f32| v - v.floor();
        match self {
            Self::Ring => {
                let dx = (x - cols / 2.) / (cols / 2.);
                let dy = (y - rows / 2.) / (rows / 2.);
                let radius = dx.hypot(dy);
                if !(0.58..=1.).contains(&radius) {
                    return 0.;
                }
                let angle = fract(dx.atan2(-dy) / TAU);
                let tail = 0.5 + 0.3 * (0.5 - 0.5 * (t * 2.1).cos());
                let behind = fract(t * 0.85 - angle);
                if behind > tail {
                    TRACK
                } else {
                    ((1. - behind / tail).powf(1.4) * 1.1).max(TRACK)
                }
            }
            Self::Dots => {
                let pitch = cols / 3.;
                let index = ((x - 0.5) / pitch).floor().min(2.);
                let dx = (x - (index + 0.5) * pitch) / (pitch / 2.);
                let dy = (y - rows / 2.) / (rows / 2.);
                if dx.hypot(dy) > 0.86 {
                    return 0.;
                }
                let wave = 0.5 + 0.5 * (t * TAU * 0.8 - index * 1.1).sin();
                (wave.powf(1.6) * 1.05).max(TRACK)
            }
            Self::Pulse => {
                let radius = ((x - cols / 2.) / (cols / 2.)).hypot((y - rows / 2.) / (rows / 2.));
                if radius > 1. {
                    return 0.;
                }
                let beat = 0.5 - 0.5 * (t * TAU * 0.6).cos();
                if radius < 0.18 + 0.08 * beat {
                    return 1.;
                }
                let band = 1.6 / cols;
                (0..3)
                    .map(|ring| {
                        let front = 0.25 + 0.75 * fract(t * 0.6 + ring as f32 / 3.);
                        if (radius - front).abs() < band {
                            (1. - front).powf(0.6) * 1.3
                        } else {
                            0.
                        }
                    })
                    .fold(0., f32::max)
            }
            Self::Bar => {
                let u = x / cols;
                let p = fract(t * 0.55);
                let eased = if p < 0.5 {
                    4. * p * p * p
                } else {
                    1. - (-2. * p + 2.).powi(3) / 2.
                };
                let behind = 1.45 * eased - u;
                if !(0. ..=0.45).contains(&behind) {
                    TRACK
                } else {
                    (1. - behind / 0.45).powf(1.1).max(TRACK)
                }
            }
        }
    }

    /// Render one frame with `ink` on transparent paper.
    pub fn render(
        self,
        cols: usize,
        rows: usize,
        t: f32,
        matrix: Matrix,
        ink: [u8; 3],
    ) -> Result<Raster, crate::ImageError> {
        let len = crate::image::checked_len(cols, rows)?;
        let mut pixels = vec![0u8; len];
        for y in 0..rows {
            for x in 0..cols {
                if self.tone(cols, rows, x, y, t) > matrix.threshold(x, y) {
                    let at = (y * cols + x) * 4;
                    pixels[at..at + 4].copy_from_slice(&[ink[0], ink[1], ink[2], 255]);
                }
            }
        }
        Ok(Raster {
            width: cols,
            height: rows,
            pixels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_variant_draws_something_and_keeps_its_track() {
        for variant in LoaderVariant::ALL {
            let (cols, rows) = variant.grid(20);
            let raster = variant
                .render(cols, rows, variant.still_time(), Matrix::Bayer4, [1, 2, 3])
                .unwrap();
            let inked = raster
                .pixels()
                .chunks_exact(4)
                .filter(|p| p[3] == 255)
                .count();
            assert!(inked > 0, "{variant:?} draws");
            assert!(inked < cols * rows, "{variant:?} leaves paper");
        }
        assert_eq!(
            LoaderVariant::Ring.tone(20, 20, 10, 10, 0.),
            0.,
            "ring has a hole"
        );
    }
}
