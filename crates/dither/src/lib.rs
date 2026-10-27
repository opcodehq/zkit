//! Portable tone fields, ordered dithering, palette mapping and image conversion.
//! Coordinates passed to fields are normalized to 0..1. Time is in seconds;
//! callers own animation scheduling, visibility and reduced-motion policy.
//!
//! Credits: the blue-noise threshold data, dithering techniques and field-based approach are adapted from dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT. Chart, button and avatar styles follow dither-kit by ripgrim (https://tripwire.sh/dither-kit).

mod avatar;
mod chart;
mod fields;
mod globe;
mod image;
mod land;
mod loader;
pub use avatar::Avatar;
pub use chart::{ChartFill, Plot, PlotKind};
pub use fields::Preset;
pub use globe::{Globe, LatLon};
pub use image::{ImageError, ImageMethod, dither_image};
pub use land::coverage as land_coverage;
pub use loader::LoaderVariant;

/// Ordered threshold matrix. Blue noise uses a 64×64 void-and-cluster tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Matrix {
    Bayer2,
    Bayer4,
    #[default]
    Bayer8,
    BlueNoise,
}

impl Matrix {
    pub fn threshold(self, x: usize, y: usize) -> f32 {
        if self == Self::BlueNoise {
            return (include_bytes!("blue-noise.bin")[(y % 64) * 64 + x % 64] as f32 + 0.5) / 256.;
        }
        let side = match self {
            Self::Bayer2 => 2,
            Self::Bayer4 => 4,
            _ => 8,
        };
        let mut rank = 0;
        let mut bit = 1;
        while bit < side {
            let a = (x / bit) & 1;
            let b = (y / bit) & 1;
            rank = rank * 4 + 2 * (a ^ b) + b;
            bit *= 2;
        }
        (rank as f32 + 0.5) / (side * side) as f32
    }
}

/// Palette ordered from paper (tone 0) to ink (tone 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette(Vec<[u8; 3]>);
impl Palette {
    /// Returns None for fewer than two colors.
    pub fn new(colors: Vec<[u8; 3]>) -> Option<Self> {
        (colors.len() >= 2).then_some(Self(colors))
    }
    pub fn duo(paper: [u8; 3], ink: [u8; 3]) -> Self {
        Self(vec![paper, ink])
    }
    pub fn colors(&self) -> &[[u8; 3]] {
        &self.0
    }
    pub fn sample(&self, tone: f32, threshold: f32) -> [u8; 3] {
        let position = finite_tone(tone) * (self.0.len() - 1) as f32;
        let lo = position.floor() as usize;
        self.0[(lo + usize::from(position.fract() > threshold)).min(self.0.len() - 1)]
    }
}
impl Default for Palette {
    fn default() -> Self {
        Self::duo([15, 18, 17], [205, 252, 98])
    }
}

pub(crate) fn finite_tone(tone: f32) -> f32 {
    if tone.is_finite() {
        tone.clamp(0., 1.)
    } else {
        0.
    }
}

/// A row-major RGBA raster. Constructed only through checked dimensions.
#[derive(Clone, Debug)]
pub struct Raster {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}
impl Raster {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    /// Compact SVG with horizontal runs instead of a rectangle per pixel.
    pub fn svg(&self) -> String {
        use std::fmt::Write;
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" shape-rendering=\"crispEdges\">",
            self.width, self.height, self.width, self.height
        );
        for y in 0..self.height {
            let mut x = 0;
            while x < self.width {
                let offset = (y * self.width + x) * 4;
                let rgba = &self.pixels[offset..offset + 4];
                let mut end = x + 1;
                while end < self.width
                    && &self.pixels[(y * self.width + end) * 4..(y * self.width + end) * 4 + 4]
                        == rgba
                {
                    end += 1;
                }
                if rgba[3] > 0 {
                    write!(svg, "<path fill=\"#{:02x}{:02x}{:02x}\" fill-opacity=\"{:.3}\" d=\"M{x} {y}h{}v1H{x}z\"/>", rgba[0], rgba[1], rgba[2], rgba[3] as f32 / 255., end-x).unwrap();
                }
                x = end;
            }
        }
        svg.push_str("</svg>");
        svg
    }
}

/// Configuration shared by native GPUI and the website's WebAssembly renderer.
#[derive(Clone, Debug)]
pub struct Dither {
    pub matrix: Matrix,
    pub palette: Palette,
    /// Tone contrast around the midpoint. Non-finite values use 1.
    pub contrast: f32,
}
impl Default for Dither {
    fn default() -> Self {
        Self {
            matrix: Matrix::default(),
            palette: Palette::default(),
            contrast: 1.,
        }
    }
}
impl Dither {
    /// Evaluate one cell. Field samples outside 0..1 are clamped.
    pub fn sample(&self, x: usize, y: usize, tone: f32) -> [u8; 3] {
        let contrast = if self.contrast.is_finite() {
            self.contrast.max(0.)
        } else {
            1.
        };
        self.palette.sample(
            (finite_tone(tone) - 0.5) * contrast + 0.5,
            self.matrix.threshold(x, y),
        )
    }
    /// Render a custom normalized tone field. Maximum 4M cells per raster.
    pub fn render(
        &self,
        width: usize,
        height: usize,
        field: impl Fn(f32, f32) -> f32,
    ) -> Result<Raster, ImageError> {
        let len = image::checked_len(width, height)?;
        let mut pixels = Vec::with_capacity(len);
        for y in 0..height {
            for x in 0..width {
                let rgb = self.sample(
                    x,
                    y,
                    field(
                        (x as f32 + 0.5) / width as f32,
                        (y as f32 + 0.5) / height as f32,
                    ),
                );
                pixels.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        Ok(Raster {
            width,
            height,
            pixels,
        })
    }
    pub fn render_preset(
        &self,
        width: usize,
        height: usize,
        preset: Preset,
        time: f32,
    ) -> Result<Raster, ImageError> {
        let aspect = width as f32 / height.max(1) as f32;
        let time = if time.is_finite() { time } else { 0. };
        self.render(width, height, |x, y| preset.tone(x, y, time, aspect))
    }
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod web;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrices_have_exact_coverage_and_tile() {
        for (matrix, side, levels) in [
            (Matrix::Bayer2, 2, 4),
            (Matrix::Bayer4, 4, 16),
            (Matrix::Bayer8, 8, 64),
            (Matrix::BlueNoise, 64, 256),
        ] {
            for level in 0..=levels {
                let tone = level as f32 / levels as f32;
                let count = (0..side)
                    .flat_map(|y| (0..side).map(move |x| (x, y)))
                    .filter(|&(x, y)| tone > matrix.threshold(x, y))
                    .count();
                assert_eq!(count, level * side * side / levels);
            }
            assert_eq!(matrix.threshold(0, 0), matrix.threshold(side, side));
        }
    }
    #[test]
    fn palette_endpoints_and_invalid_fields_are_safe() {
        let d = Dither::default();
        for tone in [f32::NAN, f32::NEG_INFINITY, -1., 0.] {
            assert_eq!(d.sample(0, 0, tone), d.palette.colors()[0]);
        }
        assert_eq!(d.sample(0, 0, 1.), d.palette.colors()[1]);
        assert!(Palette::new(vec![[0; 3]]).is_none());
        assert!(d.render(0, 10, |_, _| 0.).is_err());
        assert!(d.render(usize::MAX, 2, |_, _| 0.).is_err());
    }
    #[test]
    fn presets_are_finite_bounded_and_exportable() {
        for preset in Preset::ALL {
            for time in [0., 1., 97.] {
                for y in 0..16 {
                    for x in 0..32 {
                        let tone = preset.tone(x as f32 / 31., y as f32 / 15., time, 2.);
                        assert!(
                            tone.is_finite() && (0. ..=1.).contains(&tone),
                            "{preset:?}: {tone}"
                        );
                    }
                }
            }
            let raster = Dither::default()
                .render_preset(32, 16, *preset, 0.)
                .unwrap();
            assert_eq!(raster.pixels().len(), 32 * 16 * 4);
            assert!(raster.svg().ends_with("</svg>"));
        }
    }
}
