//! Native AppKit materials with a portable GPUI backdrop fallback.
//!
//! Call `window.enable_scene_overlay()` once on macOS. Surfaces defer their
//! content above the base scene. Keep material surfaces as siblings: native
//! materials share a single plane, so nested/interleaved materials are not
//! supported. Rectangular ancestor clipping is respected; use uniform radii.

use crate::ActiveTheme;
use gpui::{
    AnyElement, App, Corners, IntoElement, NativeMaterial, ParentElement, Pixels, RenderOnce,
    StyleRefinement, Styled, Window, canvas, deferred, div, px,
};
use gpui_base::StyledExt as _;

/// A rounded material surface. The children remain GPUI elements, preserving
/// their keyboard, focus and accessibility behavior.
#[derive(IntoElement)]
pub struct GlassSurface {
    children: Vec<AnyElement>,
    style: StyleRefinement,
    material: NativeMaterial,
    radius: Pixels,
    opaque: bool,
}

impl GlassSurface {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: StyleRefinement::default(),
            material: NativeMaterial::Glass,
            radius: px(20.),
            opaque: false,
        }
    }

    /// Use the system sidebar material instead of Liquid Glass on macOS.
    pub fn sidebar(mut self) -> Self {
        self.material = NativeMaterial::Sidebar;
        self
    }

    pub fn radius(mut self, radius: Pixels) -> Self {
        self.radius = radius.max(px(0.));
        self
    }

    /// Disable blur and use an opaque surface, for a host's reduced-transparency
    /// preference or a user-selected solid appearance.
    pub fn opaque(mut self, opaque: bool) -> Self {
        self.opaque = opaque;
        self
    }
}

impl Default for GlassSurface {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for GlassSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for GlassSurface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for GlassSurface {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let radius = self.radius;
        let material = self.material;
        let opaque = self.opaque;
        let appearance = if cx.theme().appearance.is_dark() {
            gpui::WindowAppearance::Dark
        } else {
            gpui::WindowAppearance::Light
        };
        let colors = &cx.theme().colors;
        let tint = if opaque {
            colors.surface
        } else {
            colors.surface.opacity(0.24)
        };
        deferred(
            div()
                .relative()
                .rounded(radius)
                .border_1()
                .border_color(colors.border)
                .refine_style(&self.style)
                .child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, _, window, _| {
                            if !opaque {
                                window.paint_native_material(
                                    bounds,
                                    Corners::all(radius),
                                    px(24.),
                                    material,
                                    appearance,
                                );
                            }
                            window.paint_quad(gpui::quad(
                                bounds,
                                radius,
                                tint,
                                px(0.),
                                gpui::transparent_black(),
                                Default::default(),
                            ));
                        },
                    )
                    .absolute()
                    .inset_0()
                    .size_full(),
                )
                .child(div().relative().size_full().children(self.children)),
        )
    }
}
