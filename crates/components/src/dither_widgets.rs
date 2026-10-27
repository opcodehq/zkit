//! Dithered controls, data-driven charts and cached image textures.
//!
//! Credits: adapted from components in dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT; button and avatar styles follow dither-kit by ripgrim (https://tripwire.sh/dither-kit).

use crate::{
    ActiveTheme, ChartFill, DitherSurface, Preset,
    dither::{Globe, LatLon, LoaderVariant, Matrix, Raster, paint_raster, theme_bytes},
    pulse::{self, Rate},
};
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, ClickEvent, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderImage, RenderOnce, Role, SharedString, StyleRefinement, Styled, Window, canvas, div, img,
    prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use std::{rc::Rc, sync::Arc};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A button whose face is a dithered fill: denser towards the bottom for
/// `Gradient`, or dotted, hatched or solid. It grows denser on hover, and keeps the foundation button's keyboard, focus and
/// accessibility behaviour.
#[derive(IntoElement)]
pub struct DitherButton {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    fill: ChartFill,
    color: Option<Hsla>,
    on_click: Option<ClickHandler>,
}
impl DitherButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            fill: ChartFill::Gradient,
            color: None,
            on_click: None,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Texture of the face. Default `Gradient`.
    pub fn fill(mut self, fill: ChartFill) -> Self {
        self.fill = fill;
        self
    }
    /// Ink colour. Default: the theme accent.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

/// Paint a dithered face; `boost` deepens it (hover).
fn button_face(fill: ChartFill, color: Hsla, boost: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let cell = 2.;
            let cols = (f32::from(bounds.size.width) / cell).ceil() as usize;
            let rows = (f32::from(bounds.size.height) / cell).ceil() as usize;
            for y in 0..rows {
                let depth = (0.15 + boost + 0.85 * (y as f32 + 0.5) / rows as f32).min(1.);
                for x in 0..cols {
                    let on = match fill {
                        ChartFill::Hatched => (x + y) % 4 < 2 || depth > 0.95,
                        ChartFill::Dotted => (0.3 + boost) > Matrix::Bayer4.threshold(x, y),
                        _ => fill.inks(x, y, depth),
                    };
                    if on {
                        window.paint_quad(gpui::fill(
                            gpui::Bounds {
                                origin: gpui::point(
                                    bounds.origin.x + px(x as f32 * cell),
                                    bounds.origin.y + px(y as f32 * cell),
                                ),
                                size: gpui::size(px(cell), px(cell)),
                            },
                            color.opacity(0.85),
                        ));
                    }
                }
            }
        },
    )
    .absolute()
    .inset_0()
}

impl RenderOnce for DitherButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.theme().colors;
        let color = self.color.unwrap_or(colors.accent);
        let group: SharedString = format!("dither-button-{}", self.id).into();
        let (fill, disabled) = (self.fill, self.disabled);
        gpui_base::Button::new(self.id)
            .group(group.clone())
            .accessibility_label(self.label.clone())
            .disabled(disabled)
            .relative()
            .h(px(36.))
            .px(px(16.))
            .flex()
            .items_center()
            .rounded(px(8.))
            .border_1()
            .border_color(color.opacity(0.6))
            .bg(color.opacity(0.12))
            .overflow_hidden()
            .text_color(colors.text)
            .text_size(px(14.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .when(disabled, |this| this.opacity(0.45))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |s| s.border_color(color))
                    .active(|s| s.opacity(0.85))
            })
            .focus_visible(move |s| s.border_color(colors.focus_ring))
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            })
            .child(button_face(fill, color, 0.))
            // The denser face shows on hover.
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .invisible()
                    .when(!disabled, |this| this.group_hover(group, |s| s.visible()))
                    .child(button_face(fill, color, 0.3)),
            )
            .child(div().relative().child(self.label))
    }
}

/// Cached GPU image converted from a dither raster. Create once and clone it
/// in renders; the image atlas then reuses the texture instead of uploading it.
#[derive(Clone, IntoElement)]
pub struct DitherImage {
    id: ElementId,
    image: Arc<RenderImage>,
    label: SharedString,
    style: StyleRefinement,
}
impl DitherImage {
    pub fn new(id: impl Into<ElementId>, raster: &Raster, label: impl Into<SharedString>) -> Self {
        let mut bgra = raster.pixels().to_vec();
        for pixel in bgra.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let buffer =
            image::RgbaImage::from_raw(raster.width() as u32, raster.height() as u32, bgra)
                .expect("validated raster dimensions");
        let frames = smallvec::smallvec![image::Frame::new(buffer)];
        Self {
            id: id.into(),
            image: Arc::new(RenderImage::new(frames)),
            label: label.into(),
            style: Default::default(),
        }
    }
}
impl Styled for DitherImage {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for DitherImage {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let size = self.image.size(0);
        div()
            .id(self.id)
            .role(Role::Image)
            .aria_label(self.label)
            .w(px(size.width.0 as f32))
            .h(px(size.height.0 as f32))
            .refine_style(&self.style)
            .child(img(self.image).size_full())
    }
}

/// A generative avatar: an 8 × 8 symmetric pattern and a hue, both derived
/// from the name, drawn in dithered blocks. Stable across platforms.
#[derive(IntoElement)]
pub struct DitherAvatar {
    id: ElementId,
    name: SharedString,
    size: gpui::Pixels,
}

impl DitherAvatar {
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            size: px(40.),
        }
    }
    pub fn size(mut self, size: gpui::Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for DitherAvatar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let avatar = zkit_dither::Avatar::from_name(&self.name);
        let lightness = if cx.theme().appearance.is_dark() {
            0.74
        } else {
            0.56
        };
        let ink = theme_bytes(crate::theme::oklch(lightness, 0.17, avatar.hue as f32));
        let side = f32::from(self.size);
        // Roughly 2 px dither cells, whole cells per block.
        let scale = ((side / 16.).round() as usize).max(1);
        let cell = side / (8 * scale) as f32;
        div()
            .id(self.id)
            .role(Role::Image)
            .aria_label(format!("Avatar for {}", self.name))
            .flex_none()
            .size(self.size)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        paint_raster(window, bounds.origin, &avatar.render(scale, ink), cell);
                    },
                )
                .size_full(),
            )
    }
}

/// A 40 px generative avatar for a name.
pub fn dither_avatar(id: impl Into<ElementId>, name: impl Into<SharedString>) -> DitherAvatar {
    DitherAvatar::new(id, name)
}

/// Dithered card; compose any native controls into its children.
pub fn dither_card() -> DitherSurface {
    DitherSurface::new(Preset::Glow)
        .animated(false)
        .contrast(0.25)
        .rounded(px(8.))
}

/// Decorative dithered separator whose ends fade to paper.
pub fn dither_divider() -> DitherSurface {
    DitherSurface::new(Preset::Gradient)
        .animated(false)
        .cell_size(2.)
        .w_full()
        .h(px(4.))
        .field(|u, _, _, _| (1. - (u * 2. - 1.).abs()) * 0.7)
}

/// A skeleton placeholder with a shared, reduced-motion-aware shimmer.
pub fn dither_skeleton() -> DitherSurface {
    DitherSurface::new(Preset::Gradient)
        .w_full()
        .h(px(22.))
        .cell_size(2.)
        .rounded(px(4.))
        .field(|u, _, t, _| 0.08 + 0.2 * (-((u - (t * 0.3).fract()) * 5.).powi(2)).exp())
}

/// Loader size: 24, 40 or 64 px tall.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoaderSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// A dithered loading indicator: ring, dots, pulse or bar. Announces itself
/// as a busy progress indicator; holds a still frame under reduced motion.
#[derive(IntoElement)]
pub struct DitherLoader {
    id: ElementId,
    label: SharedString,
    variant: LoaderVariant,
    size: LoaderSize,
}

impl DitherLoader {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: LoaderVariant::Ring,
            size: LoaderSize::Medium,
        }
    }
    pub fn variant(mut self, variant: LoaderVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: LoaderSize) -> Self {
        self.size = size;
        self
    }
    fn metrics(&self) -> (usize, usize, f32) {
        let (height, cell) = match self.size {
            LoaderSize::Small => (24., 1.),
            LoaderSize::Medium => (40., 2.),
            LoaderSize::Large => (64., 2.),
        };
        let (cols, rows) = self.variant.grid((height / cell) as usize);
        (cols, rows, cell)
    }
}

impl RenderOnce for DitherLoader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = window.current_view();
        let (cols, rows, cell) = self.metrics();
        let ink = theme_bytes(cx.theme().colors.accent);
        let variant = self.variant;
        div()
            .id(self.id)
            .role(Role::ProgressIndicator)
            .aria_label(self.label)
            .a11y_synthetic_children(|tree| tree.parent_node().set_busy())
            .flex_none()
            .w(px(cols as f32 * cell))
            .h(px(rows as f32 * cell))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        let time = if cx.reduce_motion() {
                            variant.still_time()
                        } else {
                            pulse::seconds(view, Rate::Smooth, cx)
                        };
                        if let Ok(raster) = variant.render(cols, rows, time, Matrix::Bayer4, ink) {
                            paint_raster(window, bounds.origin, &raster, cell);
                        }
                    },
                )
                .size_full(),
            )
    }
}

/// The default loader: a medium ring.
pub fn dither_loader(id: impl Into<ElementId>, label: impl Into<SharedString>) -> DitherLoader {
    DitherLoader::new(id, label)
}

/// A rotating dithered globe with optional markers and arcs. The globe
/// stays round and centred in its box; land uses the accent and markers the
/// warning colour. Decorative unless given a label.
#[derive(IntoElement)]
pub struct DitherGlobe {
    id: ElementId,
    style: StyleRefinement,
    globe: Globe,
    cell: f32,
    label: Option<SharedString>,
}

impl DitherGlobe {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            globe: Globe::default(),
            cell: 2.,
            label: None,
        }
    }
    pub fn markers(mut self, markers: impl IntoIterator<Item = LatLon>) -> Self {
        self.globe.markers = markers.into_iter().collect();
        self
    }
    pub fn arcs(mut self, arcs: impl IntoIterator<Item = (LatLon, LatLon)>) -> Self {
        self.globe.arcs = arcs.into_iter().collect();
        self
    }
    /// Degrees per second; negative spins the other way.
    pub fn speed(mut self, degrees: f32) -> Self {
        self.globe.speed = degrees;
        self
    }
    pub fn tilt(mut self, degrees: f32) -> Self {
        self.globe.tilt = degrees;
        self
    }
    pub fn cell_size(mut self, cell: f32) -> Self {
        self.cell = cell.max(1.);
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Styled for DitherGlobe {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for DitherGlobe {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = window.current_view();
        let colors = &cx.theme().colors;
        let (ink, accent) = (theme_bytes(colors.accent), theme_bytes(colors.warning));
        let (globe, cell) = (self.globe, self.cell);
        div()
            .id(self.id)
            .relative()
            .overflow_hidden()
            .when_some(self.label, |this, label| {
                this.role(Role::Image).aria_label(label)
            })
            .refine_style(&self.style)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
                        if w < 8. || h < 8. || !bounds.intersects(&window.content_mask().bounds) {
                            return;
                        }
                        let time = if cx.reduce_motion() {
                            0.
                        } else {
                            pulse::seconds(view, Rate::Coarse, cx)
                        };
                        // Stay near 40K cells on large surfaces.
                        let cell = cell.max((w * h / 40_000.).sqrt());
                        let (cols, rows) = ((w / cell) as usize, (h / cell) as usize);
                        if let Ok(raster) = globe.render(cols, rows, time, ink, accent) {
                            paint_raster(window, bounds.origin, &raster, cell);
                        }
                    },
                )
                .absolute()
                .inset_0()
                .size_full(),
            )
    }
}

/// Empty-state layout with an optional host-provided action.
pub fn dither_empty_state(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    action: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(14.))
        .p(px(32.))
        .child(
            DitherSurface::new(Preset::Orb)
                .animated(false)
                .size(px(72.)),
        )
        .child(div().text_size(px(20.)).child(title.into()))
        .child(div().text_size(px(13.)).child(description.into()))
        .child(action)
}

/// Not-found layout with a native action supplied by the host.
pub fn dither_not_found(action: impl IntoElement) -> impl IntoElement {
    dither_empty_state(
        "404 — Not found",
        "There is nothing at this address.",
        action,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_texture_swaps_red_blue_once_and_clones_share_the_cache() {
        let palette = crate::Palette::duo([1, 2, 3], [40, 50, 60]);
        let raster = crate::Dither {
            palette,
            ..Default::default()
        }
        .render(2, 1, |_, _| 1.)
        .unwrap();
        let image = DitherImage::new("test-image", &raster, "A test image");
        assert_eq!(
            image.image.as_bytes(0).unwrap(),
            &[60, 50, 40, 255, 60, 50, 40, 255]
        );
        assert!(Arc::ptr_eq(&image.image, &image.clone().image));
        assert_eq!(raster.pixels(), &[40, 50, 60, 255, 40, 50, 60, 255]);
    }
}
