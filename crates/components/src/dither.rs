//! Dither surfaces painted as GPUI quads, shared across native and web backends.
//! Put live controls in children; the texture itself is decorative.
//!
//! Credits: the dither engine adapts dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT.

use crate::{
    ActiveTheme,
    pulse::{self, Rate},
};
use gpui::{
    AnyElement, App, Bounds, IntoElement, ParentElement, Point, RenderOnce, Size, StyleRefinement,
    Styled, Window, canvas, div, fill, px, rgb,
};
use gpui_base::StyledExt as _;
use std::rc::Rc;
pub use zkit_dither::{
    Dither, Globe, ImageMethod, LatLon, LoaderVariant, Matrix, Palette, Preset, Raster,
    dither_image,
};

type Field = Rc<dyn Fn(f32, f32, f32, f32) -> f32>;

/// A styled dither texture with optional foreground children.
/// Fields run on CPU and submit horizontal color runs as batched GPU quads.
/// Approximately 32K cells are sampled per surface, independent of window size.
#[derive(IntoElement)]
pub struct DitherSurface {
    style: StyleRefinement,
    children: Vec<AnyElement>,
    preset: Preset,
    custom: Option<Field>,
    matrix: Matrix,
    palette: Option<Palette>,
    cell_size: f32,
    contrast: f32,
    animated: bool,
    still_time: f32,
}
impl DitherSurface {
    pub fn new(preset: Preset) -> Self {
        Self {
            style: Default::default(),
            children: Vec::new(),
            preset,
            custom: None,
            matrix: Matrix::Bayer8,
            palette: None,
            cell_size: 4.,
            contrast: 1.,
            animated: true,
            still_time: 6.,
        }
    }
    /// Field parameters are normalized x, y, seconds and width/height.
    pub fn field(mut self, field: impl Fn(f32, f32, f32, f32) -> f32 + 'static) -> Self {
        self.custom = Some(Rc::new(field));
        self
    }
    pub fn matrix(mut self, matrix: Matrix) -> Self {
        self.matrix = matrix;
        self
    }
    /// Without an override, paper and ink come from the active theme.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }
    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell_size = if size.is_finite() {
            size.clamp(1., 32.)
        } else {
            4.
        };
        self
    }
    pub fn contrast(mut self, contrast: f32) -> Self {
        self.contrast = contrast;
        self
    }
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
    pub fn still_time(mut self, seconds: f32) -> Self {
        self.still_time = if seconds.is_finite() { seconds } else { 0. };
        self
    }
}
impl Styled for DitherSurface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for DitherSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl RenderOnce for DitherSurface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = window.current_view();
        let to_bytes = |color: gpui::Hsla| {
            let c = gpui::Rgba::from(color);
            [(c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8]
        };
        let palette = self.palette.unwrap_or_else(|| {
            Palette::duo(
                to_bytes(cx.theme().colors.background),
                to_bytes(cx.theme().colors.accent),
            )
        });
        let dither = Dither {
            matrix: self.matrix,
            palette,
            contrast: self.contrast,
        };
        div()
            .relative()
            .overflow_hidden()
            .refine_style(&self.style)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
                            return;
                        }
                        let mask = window.content_mask().bounds;
                        if !bounds.intersects(&mask) {
                            return;
                        }
                        let time = if self.animated && !cx.reduce_motion() {
                            pulse::seconds(view, Rate::Coarse, cx)
                        } else {
                            self.still_time
                        };
                        let w = f32::from(bounds.size.width);
                        let h = f32::from(bounds.size.height);
                        let cell = self.cell_size.max((w * h / 32_000.).sqrt());
                        let cols = (w / cell).ceil() as usize;
                        let rows = (h / cell).ceil() as usize;
                        for y in 0..rows {
                            let mut start = 0;
                            let color_at = |x: usize| {
                                let u = (x as f32 + 0.5) / cols as f32;
                                let v = (y as f32 + 0.5) / rows as f32;
                                let tone = if let Some(field) = &self.custom {
                                    field(u, v, time, w / h)
                                } else {
                                    self.preset.tone(u, v, time, w / h)
                                };
                                dither.sample(x, y, tone)
                            };
                            while start < cols {
                                let color = color_at(start);
                                let mut end = start + 1;
                                while end < cols && color_at(end) == color {
                                    end += 1;
                                }
                                let left = start as f32 * cell;
                                let top = y as f32 * cell;
                                let rect = Bounds {
                                    origin: Point::new(
                                        bounds.origin.x + px(left),
                                        bounds.origin.y + px(top),
                                    ),
                                    size: Size::new(
                                        px((end as f32 * cell).min(w) - left),
                                        px(cell.min(h - top)),
                                    ),
                                };
                                window.paint_quad(fill(
                                    rect,
                                    rgb(((color[0] as u32) << 16)
                                        | ((color[1] as u32) << 8)
                                        | color[2] as u32),
                                ));
                                start = end;
                            }
                        }
                    },
                )
                .absolute()
                .inset_0()
                .size_full(),
            )
            .child(div().relative().size_full().children(self.children))
    }
}

/// Paint a raster at `origin`, one `cell` per pixel, as horizontal runs of
/// equal colour. Transparent pixels are skipped.
pub(crate) fn paint_raster(
    window: &mut Window,
    origin: Point<gpui::Pixels>,
    raster: &Raster,
    cell: f32,
) {
    let (w, pixels) = (raster.width(), raster.pixels());
    for y in 0..raster.height() {
        let row = &pixels[y * w * 4..(y + 1) * w * 4];
        let mut x = 0;
        while x < w {
            let px_at = |i: usize| &row[i * 4..i * 4 + 4];
            if px_at(x)[3] == 0 {
                x += 1;
                continue;
            }
            let color = px_at(x);
            let mut end = x + 1;
            while end < w && px_at(end) == color {
                end += 1;
            }
            window.paint_quad(fill(
                Bounds {
                    origin: Point::new(
                        origin.x + px(x as f32 * cell),
                        origin.y + px(y as f32 * cell),
                    ),
                    size: Size::new(px((end - x) as f32 * cell), px(cell)),
                },
                rgb(((color[0] as u32) << 16) | ((color[1] as u32) << 8) | color[2] as u32),
            ));
            x = end;
        }
    }
}

pub(crate) fn theme_bytes(color: gpui::Hsla) -> [u8; 3] {
    let c = gpui::Rgba::from(color);
    [
        (c.r * 255.).round() as u8,
        (c.g * 255.).round() as u8,
        (c.b * 255.).round() as u8,
    ]
}
