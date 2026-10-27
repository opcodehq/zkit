//! Credits: image dithering techniques are adapted from dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT.

use crate::{Dither, Matrix, Palette, Raster};
use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageError {
    InvalidDimensions,
    InvalidBuffer,
}
impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidDimensions => "dimensions must be nonzero and at most 4 million cells",
            Self::InvalidBuffer => "RGBA buffer does not match dimensions",
        })
    }
}
impl Error for ImageError {}
pub(crate) fn checked_len(width: usize, height: usize) -> Result<usize, ImageError> {
    let cells = width
        .checked_mul(height)
        .filter(|&n| n > 0 && n <= 4_000_000)
        .ok_or(ImageError::InvalidDimensions)?;
    Ok(cells * 4)
}
#[derive(Clone, Copy, Debug)]
pub enum ImageMethod {
    Ordered(Matrix),
    FloydSteinberg,
    Atkinson,
    SierraLite,
}

/// Convert an RGBA image, preserving alpha and input bytes. Error diffusion
/// alternates scan direction each row to avoid directional artifacts.
pub fn dither_image(
    width: usize,
    height: usize,
    rgba: &[u8],
    palette: Palette,
    method: ImageMethod,
) -> Result<Raster, ImageError> {
    let len = checked_len(width, height)?;
    if rgba.len() != len {
        return Err(ImageError::InvalidBuffer);
    }
    let mut pixels = vec![0; len];
    let mut work: Vec<[f32; 3]> = rgba
        .chunks_exact(4)
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let kernel: &[(isize, usize, f32)] = match method {
        ImageMethod::Ordered(_) => &[],
        ImageMethod::FloydSteinberg => &[
            (1, 0, 7. / 16.),
            (-1, 1, 3. / 16.),
            (0, 1, 5. / 16.),
            (1, 1, 1. / 16.),
        ],
        ImageMethod::Atkinson => &[
            (1, 0, 0.125),
            (2, 0, 0.125),
            (-1, 1, 0.125),
            (0, 1, 0.125),
            (1, 1, 0.125),
            (0, 2, 0.125),
        ],
        ImageMethod::SierraLite => &[(1, 0, 0.5), (-1, 1, 0.25), (0, 1, 0.25)],
    };
    let dither = Dither {
        palette: palette.clone(),
        ..Default::default()
    };
    for y in 0..height {
        let reverse = y % 2 == 1 && !matches!(method, ImageMethod::Ordered(_));
        for step in 0..width {
            let x = if reverse { width - 1 - step } else { step };
            let index = y * width + x;
            let color = work[index];
            let out = match method {
                ImageMethod::Ordered(matrix) => {
                    // Palette entries represent increasing tone; for photos map
                    // luminance between their endpoint luminances.
                    let luma = |c: [u8; 3]| {
                        (c[0] as f32 * 0.2126 + c[1] as f32 * 0.7152 + c[2] as f32 * 0.0722) / 255.
                    };
                    let lo = luma(palette.colors()[0]);
                    let hi = luma(*palette.colors().last().unwrap());
                    let lum = (color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722) / 255.;
                    let tone = if (hi - lo).abs() > 0.001 {
                        (lum - lo) / (hi - lo)
                    } else {
                        lum
                    };
                    palette.sample(tone, matrix.threshold(x, y))
                }
                _ => *dither
                    .palette
                    .colors()
                    .iter()
                    .min_by(|a, b| {
                        let dist = |p: &&[u8; 3]| {
                            (color[0] - p[0] as f32).powi(2) * 0.3
                                + (color[1] - p[1] as f32).powi(2) * 0.59
                                + (color[2] - p[2] as f32).powi(2) * 0.11
                        };
                        dist(a).total_cmp(&dist(b))
                    })
                    .unwrap(),
            };
            pixels[index * 4..index * 4 + 4].copy_from_slice(&[
                out[0],
                out[1],
                out[2],
                rgba[index * 4 + 3],
            ]);
            for &(dx, dy, weight) in kernel {
                let nx = x as isize + if reverse { -dx } else { dx };
                let ny = y + dy;
                if nx >= 0 && (nx as usize) < width && ny < height {
                    let target = ny * width + nx as usize;
                    // Fully transparent pixels neither receive nor emit error.
                    if rgba[index * 4 + 3] > 0 && rgba[target * 4 + 3] > 0 {
                        for c in 0..3 {
                            work[target][c] += (color[c] - out[c] as f32) * weight;
                        }
                    }
                }
            }
        }
    }
    Ok(Raster {
        width,
        height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diffusion_preserves_alpha_input_and_only_uses_palette_colors() {
        let input: Vec<u8> = (0..64)
            .flat_map(|i| [i * 4, i * 4, i * 4, if i % 3 == 0 { 0 } else { 173 }])
            .collect();
        let original = input.clone();
        for method in [
            ImageMethod::FloydSteinberg,
            ImageMethod::Atkinson,
            ImageMethod::SierraLite,
            ImageMethod::Ordered(Matrix::Bayer8),
        ] {
            let palette = Palette::duo([0; 3], [255; 3]);
            let image = dither_image(8, 8, &input, palette.clone(), method).unwrap();
            for (out, src) in image.pixels().chunks_exact(4).zip(input.chunks_exact(4)) {
                assert_eq!(out[3], src[3]);
                assert!(palette.colors().iter().any(|c| c == &out[..3]));
            }
        }
        assert_eq!(input, original);
        assert_eq!(
            dither_image(8, 8, &input[..4], Palette::default(), ImageMethod::Atkinson).unwrap_err(),
            ImageError::InvalidBuffer
        );
    }
}
