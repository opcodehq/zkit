//! The marks of a data chart, dithered: areas, lines, bars, donuts and
//! radars, each series in its own colour and fill style. Axes, labels and
//! legends belong to the host; this draws the plot area only.
//!
//! Credits: fill styles, stacking, ribbon lines and the radar follow the design of dither-kit by ripgrim (https://tripwire.sh/dither-kit). Written from scratch; no code copied.

use crate::{Matrix, Raster};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlotKind {
    #[default]
    Area,
    Line,
    Bar,
    Donut,
    Radar,
    /// One row per series, one column per category; density follows value.
    Heatmap,
}

/// How a series' body is filled. Edges and lines are always solid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartFill {
    /// Dense at the base, airy towards the line.
    #[default]
    Gradient,
    /// Sparse, even dots.
    Dotted,
    /// Diagonal stripes.
    Hatched,
    Solid,
}

impl ChartFill {
    /// The default fill for the `index`-th series: gradient, hatched, dotted, solid.
    pub fn nth(index: usize) -> Self {
        [Self::Gradient, Self::Hatched, Self::Dotted, Self::Solid][index % 4]
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "gradient" => Some(Self::Gradient),
            "dotted" => Some(Self::Dotted),
            "hatched" => Some(Self::Hatched),
            "solid" => Some(Self::Solid),
            _ => None,
        }
    }

    /// Whether cell `(x, y)` is inked at `depth` (0 at the edge, 1 at the base).
    pub fn inks(self, x: usize, y: usize, depth: f32) -> bool {
        match self {
            // Solid through the lower half, breaking into dots near the edge.
            Self::Gradient => {
                let d = (depth / 0.6).clamp(0., 1.);
                0.1 + 0.9 * d * d * (3. - 2. * d) > Matrix::Bayer4.threshold(x, y)
            }
            Self::Dotted => 0.2 > Matrix::Bayer4.threshold(x, y),
            // Thin one-cell diagonals, rising to the right.
            Self::Hatched => (x + y).is_multiple_of(3),
            Self::Solid => true,
        }
    }
}

/// A plot to draw. Cartesian kinds read `series[s][category]`; a donut reads
/// the first series by category; a radar reads one polygon per series with
/// categories as spokes.
#[derive(Clone, Debug)]
pub struct Plot<'a> {
    pub kind: PlotKind,
    pub series: &'a [Vec<f32>],
    pub fills: &'a [ChartFill],
    /// Stack areas and bars instead of overlaying or grouping them.
    pub stacked: bool,
    /// The value at the top of the axis (or the radar's rim).
    pub top: f32,
    /// Reveal from 0 to 1, for entrance animations.
    pub progress: f32,
}

impl Plot<'_> {
    fn count(&self) -> usize {
        self.series.iter().map(Vec::len).max().unwrap_or(0)
    }

    fn fill(&self, series: usize) -> ChartFill {
        self.fills
            .get(series)
            .copied()
            .unwrap_or(ChartFill::nth(series))
    }

    fn value(&self, series: usize, index: usize) -> f32 {
        self.series
            .get(series)
            .and_then(|s| s.get(index))
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.)
            .max(0.)
    }

    /// Sum of series `0..=series` at `index` when stacked, else the value.
    fn level(&self, series: usize, index: usize) -> f32 {
        if self.stacked {
            (0..=series).map(|s| self.value(s, index)).sum()
        } else {
            self.value(series, index)
        }
    }

    /// A series' level between categories on a Catmull-Rom spline, never
    /// below zero, so curves pass through every point smoothly.
    fn level_at(&self, series: usize, u: f32) -> f32 {
        let count = self.count();
        if count < 2 {
            return self.level(series, 0);
        }
        let x = u.clamp(0., 1.) * (count - 1) as f32;
        let i = (x as usize).min(count - 2);
        let t = x - i as f32;
        let at = |k: isize| self.level(series, k.clamp(0, count as isize - 1) as usize);
        let (p0, p1, p2, p3) = (
            at(i as isize - 1),
            at(i as isize),
            at(i as isize + 1),
            at(i as isize + 2),
        );
        let (t2, t3) = (t * t, t * t * t);
        let y = 0.5
            * (2. * p1
                + (p2 - p0) * t
                + (2. * p0 - 5. * p1 + 4. * p2 - p3) * t2
                + (3. * p1 - p0 - 3. * p2 + p3) * t3);
        // Keep the overshoot between the neighbouring points.
        y.clamp(p1.min(p2), p1.max(p2)).max(0.)
    }

    /// Series and whether it inks cell `(x, y)` of a `cols × rows` grid.
    fn cell(&self, cols: usize, rows: usize, x: usize, y: usize) -> Option<usize> {
        let top = if self.top > 0. && self.top.is_finite() {
            self.top
        } else {
            1.
        };
        let u = (x as f32 + 0.5) / cols as f32;
        let v = (y as f32 + 0.5) / rows as f32;
        let row = 1. / rows as f32;
        let n = self.series.len();
        let count = self.count();
        if n == 0 || count == 0 {
            return None;
        }
        match self.kind {
            PlotKind::Area | PlotKind::Line => {
                if u > self.progress {
                    return None;
                }
                let edges: Vec<f32> = (0..n).map(|s| 1. - self.level_at(s, u) / top).collect();
                if self.kind == PlotKind::Line {
                    // Ribbons: a solid edge over a short band that fades out.
                    for s in (0..n).rev() {
                        let d = (v - edges[s]) / row;
                        if (-0.6..1.).contains(&d) {
                            return Some(s);
                        }
                        if (1. ..8.).contains(&d)
                            && 0.85 * (1. - (d - 1.) / 7.) > Matrix::Bayer4.threshold(x, y)
                        {
                            return Some(s);
                        }
                    }
                    return None;
                }
                // Edges first: the frontmost series is the last one drawn.
                for s in (0..n).rev() {
                    if (v - edges[s]).abs() < row * 0.7 {
                        return Some(s);
                    }
                }
                // The body below the nearest line above belongs to that series.
                let (s, edge) = (0..n)
                    .filter(|&s| v > edges[s])
                    .map(|s| (s, edges[s]))
                    .max_by(|a, b| a.1.total_cmp(&b.1))?;
                let base = if self.stacked {
                    (0..n)
                        .filter(|&o| edges[o] > edge + 1e-6)
                        .map(|o| edges[o])
                        .fold(1., f32::min)
                } else {
                    1.
                };
                let depth = (v - edge) / (base - edge).max(1e-3);
                self.fill(s).inks(x, y, depth).then_some(s)
            }
            PlotKind::Bar => {
                let slot = u * count as f32;
                let index = (slot as usize).min(count - 1);
                if (index as f32 + 0.5) / count as f32 > self.progress {
                    return None;
                }
                let local = slot - index as f32;
                let (start, width) = (0.12, 0.76);
                if !(start..start + width).contains(&local) {
                    return None;
                }
                let (s, low, high) = if self.stacked {
                    let level = (1. - v) * top;
                    let s = (0..n).find(|&s| level <= self.level(s, index))?;
                    let low = if s == 0 { 0. } else { self.level(s - 1, index) };
                    (s, low, self.level(s, index))
                } else {
                    let within = (local - start) / width * n as f32;
                    let s = (within as usize).min(n - 1);
                    if n > 1 && within - s as f32 > 0.86 {
                        return None;
                    }
                    (s, 0., self.value(s, index))
                };
                let (edge, base) = (1. - high / top, 1. - low / top);
                if v < edge || high <= low {
                    return None;
                }
                if v - edge < row {
                    return Some(s);
                }
                self.fill(s)
                    .inks(x, y, (v - edge) / (base - edge).max(1e-3))
                    .then_some(s)
            }
            PlotKind::Donut => {
                let values: Vec<f32> = (0..count).map(|i| self.value(0, i)).collect();
                let total: f32 = values.iter().sum();
                let (dx, dy) = (u - 0.5, v - 0.5);
                let r = dx.hypot(dy) * 2.;
                if !(0.62..=1.).contains(&r) || total <= 0. {
                    return None;
                }
                let angle = (dx.atan2(-dy) / TAU).rem_euclid(1.);
                if angle > self.progress {
                    return None;
                }
                let mut start = 0.;
                for (i, value) in values.iter().enumerate() {
                    let end = start + value / total;
                    if angle < end {
                        if angle - start < 0.005 || end - angle < 0.005 {
                            return None;
                        }
                        // Dense at the rim, airy towards the hole.
                        let depth = (r - 0.62) / 0.38;
                        return ChartFill::Gradient.inks(x, y, depth).then_some(i);
                    }
                    start = end;
                }
                None
            }
            PlotKind::Heatmap => {
                let (col, row) = (u * count as f32, v * n as f32);
                let (i, s) = ((col as usize).min(count - 1), (row as usize).min(n - 1));
                if col.fract() < 0.1
                    || row.fract() < 0.12
                    || (i as f32 + 0.5) / count as f32 > self.progress
                {
                    return None;
                }
                let value = self.value(s, i);
                // Four steps like a contribution graph; empty cells keep a faint grid.
                let level = if value <= 0. {
                    0.
                } else {
                    (value / top * 4.).ceil() / 4.
                };
                let tone = if level == 0. {
                    0.07
                } else {
                    0.18 + 0.82 * level
                };
                (tone > Matrix::Bayer4.threshold(x, y)).then_some(0)
            }
            PlotKind::Radar => {
                let (dx, dy) = ((u - 0.5) * 2., (v - 0.5) * 2.);
                let r = dx.hypot(dy) / 0.9;
                if r > 1. {
                    return None;
                }
                let spokes = count.max(3);
                let angle = dx.atan2(-dy).rem_euclid(TAU);
                let sector = angle / TAU * spokes as f32;
                let i = sector as usize % spokes;
                let f = sector - sector.floor();
                for s in (0..n).rev() {
                    let a = self.value(s, i) / top;
                    let b = self.value(s, (i + 1) % spokes) / top;
                    // Radius of the straight edge between spokes i and i+1.
                    let (ta, tb) = (
                        i as f32 / spokes as f32 * TAU,
                        (i + 1) as f32 / spokes as f32 * TAU,
                    );
                    let (ax, ay, bx, by) = (a * ta.sin(), a * ta.cos(), b * tb.sin(), b * tb.cos());
                    let (px, py) = (angle.sin(), angle.cos());
                    let denom = px * (by - ay) - py * (bx - ax);
                    let edge = if denom.abs() < 1e-6 {
                        a + (b - a) * f
                    } else {
                        (ax * (by - ay) - ay * (bx - ax)) / denom
                    };
                    let edge = edge * self.progress.clamp(0., 1.);
                    if (r - edge).abs() < 1.4 / rows as f32 / 0.9 {
                        return Some(s);
                    }
                    if r < edge {
                        return self
                            .fill(s)
                            // Dense at the rim, airy at the centre.
                            .inks(x, y, r / edge.max(1e-3))
                            .then_some(s);
                    }
                }
                None
            }
        }
    }

    /// Render the marks; paper is transparent. `colors[s]` inks series `s`
    /// (for a donut, category `s`).
    pub fn render(
        &self,
        cols: usize,
        rows: usize,
        colors: &[[u8; 3]],
    ) -> Result<Raster, crate::ImageError> {
        let len = crate::image::checked_len(cols, rows)?;
        let mut pixels = vec![0u8; len];
        if colors.is_empty() {
            return Ok(Raster {
                width: cols,
                height: rows,
                pixels,
            });
        }
        for y in 0..rows {
            for x in 0..cols {
                if let Some(s) = self.cell(cols, rows, x, y) {
                    let c = colors[s % colors.len()];
                    let at = (y * cols + x) * 4;
                    pixels[at..at + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
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

    fn inked(r: &Raster, x: usize, y: usize) -> Option<[u8; 3]> {
        let p = &r.pixels()[(y * r.width() + x) * 4..][..4];
        (p[3] == 255).then(|| [p[0], p[1], p[2]])
    }

    #[test]
    fn stacked_bars_put_each_series_in_its_own_segment() {
        let series = vec![vec![10., 10.], vec![10., 0.]];
        let plot = Plot {
            kind: PlotKind::Bar,
            series: &series,
            fills: &[ChartFill::Solid; 2],
            stacked: true,
            top: 20.,
            progress: 1.,
        };
        let r = plot.render(40, 40, &[[1, 0, 0], [0, 2, 0]]).unwrap();
        assert_eq!(
            inked(&r, 10, 35),
            Some([1, 0, 0]),
            "bottom of the first bar is series 0"
        );
        assert_eq!(
            inked(&r, 10, 5),
            Some([0, 2, 0]),
            "top of the first bar is series 1"
        );
        assert_eq!(inked(&r, 30, 5), None, "second bar stops at 10");
        assert_eq!(inked(&r, 1, 35), None, "gutter is paper");
    }

    #[test]
    fn hatched_fill_is_striped_and_reveal_hides_the_rest() {
        let series = vec![vec![10., 10., 10.]];
        let plot = Plot {
            kind: PlotKind::Area,
            series: &series,
            fills: &[ChartFill::Hatched],
            stacked: false,
            top: 10.,
            progress: 0.5,
        };
        let r = plot.render(40, 40, &[[9, 9, 9]]).unwrap();
        let row: Vec<bool> = (0..8).map(|x| inked(&r, x, 30).is_some()).collect();
        assert!(
            row.contains(&true) && row.contains(&false),
            "stripes alternate"
        );
        assert!(
            (30..40).all(|x| inked(&r, x, 30).is_none()),
            "unrevealed half is paper"
        );
    }

    #[test]
    fn donut_and_radar_stay_inside_their_disc() {
        let series = vec![vec![3., 2., 1.]];
        for kind in [PlotKind::Donut, PlotKind::Radar] {
            let plot = Plot {
                kind,
                series: &series,
                fills: &[],
                stacked: false,
                top: 3.,
                progress: 1.,
            };
            let r = plot
                .render(30, 30, &[[1, 1, 1], [2, 2, 2], [3, 3, 3]])
                .unwrap();
            assert_eq!(inked(&r, 0, 0), None, "{kind:?} corner");
            assert!(
                r.pixels().chunks_exact(4).any(|p| p[3] == 255),
                "{kind:?} draws"
            );
        }
    }
}
