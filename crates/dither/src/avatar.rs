//! Generative avatars: an 8 × 8 pattern and a hue, both derived from a name.
//! Thirty-two pattern bits are mirrored left/right or top/bottom (the name
//! decides which), so every avatar is symmetric and recognisable.
//!
//! Credits: the folded-pattern, hue-per-name design follows dither-kit by ripgrim (https://tripwire.sh/dither-kit). Written from scratch; no code copied.

use crate::{Matrix, Raster};

/// FNV-1a, independent of platform hashers so avatars never change.
fn fnv(bytes: &[u8], seed: u32) -> u32 {
    bytes.iter().fold(2_166_136_261 ^ seed, |h, &b| {
        (h ^ b as u32).wrapping_mul(16_777_619)
    })
}

/// The avatar for a name: which of the 8 × 8 cells are on, and a hue in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Avatar {
    pub cells: [[bool; 8]; 8],
    /// One of 180 hues, in degrees.
    pub hue: u16,
}

impl Avatar {
    pub fn from_name(name: &str) -> Self {
        let bits = fnv(name.as_bytes(), 0);
        let meta = fnv(name.as_bytes(), 0x9e37_79b9);
        let fold_left_right = meta & 1 == 0;
        let mut cells = [[false; 8]; 8];
        for (i, cell) in (0..32).map(|i| (i, bits >> i & 1 == 1)) {
            let (row, col) = if fold_left_right {
                (i / 4, i % 4)
            } else {
                (i / 8, i % 8)
            };
            if fold_left_right {
                cells[row][col] = cell;
                cells[row][7 - col] = cell;
            } else {
                cells[row][col] = cell;
                cells[7 - row][col] = cell;
            }
        }
        Self {
            cells,
            hue: ((meta >> 1) % 180 * 2) as u16,
        }
    }

    /// Render at `scale` dither cells per pattern cell, inked with `color`.
    /// On cells carry a light dithered texture so blocks read as pixels.
    pub fn render(&self, scale: usize, color: [u8; 3]) -> Raster {
        let scale = scale.max(1);
        let side = 8 * scale;
        let mut pixels = vec![0u8; side * side * 4];
        for y in 0..side {
            for x in 0..side {
                if !self.cells[y / scale][x / scale] {
                    continue;
                }
                // Denser towards the bottom of each block.
                let depth = ((y % scale) as f32 + 0.5) / scale as f32;
                if scale > 2 && 0.62 + 0.38 * depth <= Matrix::Bayer4.threshold(x, y) {
                    continue;
                }
                let at = (y * side + x) * 4;
                pixels[at..at + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
            }
        }
        Raster {
            width: side,
            height: side,
            pixels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatars_are_stable_symmetric_and_distinct() {
        let a = Avatar::from_name("Ada Lovelace");
        assert_eq!(a, Avatar::from_name("Ada Lovelace"));
        assert_ne!(a, Avatar::from_name("Grace Hopper"));
        let lr = (0..8).all(|r| (0..8).all(|c| a.cells[r][c] == a.cells[r][7 - c]));
        let tb = (0..8).all(|r| (0..8).all(|c| a.cells[r][c] == a.cells[7 - r][c]));
        assert!(lr || tb);
        assert!(a.hue < 360 && a.hue.is_multiple_of(2));
    }
}
