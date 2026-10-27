//! Semantic theme tokens: two designed appearances, one token set.
//!
//! Colors are authored on the OKLCH neutral scale (perceptually even
//! lightness steps) and converted to gpui [`Hsla`]. Layout numbers live in
//! [`Metrics`] and never depend on which color is painted.
//!
//! # Light is designed, not inverted
//!
//! Mirroring lightness produces a washed-out inverted look, so each
//! appearance assigns roles explicitly:
//!
//! 1. **Surface order flips meaning.** In dark, content is the deepest plane
//!    and raised surfaces get lighter. In light, content is white and chrome
//!    (sidebars, toolbars) recedes by turning grey. Floating surfaces stay
//!    white and separate with a border and shadow.
//! 2. **Washes keep their alpha.** Hover and active fills use the same alpha
//!    in both appearances; only the ink flips between white and black.
//! 3. **Accents move down the scale.** Accents tuned for contrast against
//!    near-black fail on white, so light mode uses darker siblings at the same
//!    hue, restoring the contrast ratio rather than the lightness.
//!
//! Contrast pairs are asserted in this module's tests, not eyeballed.
//!
//! Install with [`Theme::install`]; read with [`ActiveTheme::theme`].

use gpui::{App, BoxShadow, Global, Hsla, Pixels, SharedString, hsla, point, px};
use serde::{Deserialize, Serialize};

/// Which designed appearance a theme was built for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Appearance {
    #[default]
    Dark,
    Light,
}

impl Appearance {
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}

/// Interactive accent family. One choice is one hue identity: every accent
/// role stays on the same authored hue in both appearances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Accent {
    #[default]
    Indigo,
    Blue,
    Cyan,
    Green,
    Amber,
    Orange,
    Pink,
}

impl Accent {
    pub const ALL: [Self; 7] = [
        Self::Indigo,
        Self::Blue,
        Self::Cyan,
        Self::Green,
        Self::Amber,
        Self::Orange,
        Self::Pink,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Indigo => "Indigo",
            Self::Blue => "Blue",
            Self::Cyan => "Cyan",
            Self::Green => "Green",
            Self::Amber => "Amber",
            Self::Orange => "Orange",
            Self::Pink => "Pink",
        }
    }

    /// `(primary, strong)` for an appearance. Authored pairs: runtime
    /// contrast correction would gamut-clip and shift apparent hue between
    /// appearances. `strong` carries [`Colors::on_accent`] text.
    fn tones(self, appearance: Appearance) -> (Hsla, Hsla) {
        use Appearance::*;
        match (self, appearance) {
            (Self::Indigo, Dark) => (oklch(0.673, 0.182, 276.935), oklch(0.56, 0.24, 277.117)),
            (Self::Indigo, Light) => (oklch(0.511, 0.262, 276.966), oklch(0.511, 0.262, 276.966)),
            (Self::Blue, Dark) => (oklch(0.70, 0.17, 255.0), oklch(0.50, 0.20, 255.0)),
            (Self::Blue, Light) => (oklch(0.47, 0.21, 255.0), oklch(0.47, 0.21, 255.0)),
            (Self::Cyan, Dark) => (oklch(0.76, 0.13, 205.0), oklch(0.49, 0.12, 205.0)),
            (Self::Cyan, Light) => (oklch(0.45, 0.11, 205.0), oklch(0.45, 0.11, 205.0)),
            (Self::Green, Dark) => (oklch(0.75, 0.17, 150.0), oklch(0.50, 0.15, 150.0)),
            (Self::Green, Light) => (oklch(0.46, 0.14, 150.0), oklch(0.46, 0.14, 150.0)),
            (Self::Amber, Dark) => (oklch(0.80, 0.17, 84.0), oklch(0.52, 0.14, 84.0)),
            (Self::Amber, Light) => (oklch(0.48, 0.14, 84.0), oklch(0.48, 0.14, 84.0)),
            (Self::Orange, Dark) => (oklch(0.75, 0.18, 55.0), oklch(0.54, 0.19, 55.0)),
            (Self::Orange, Light) => (oklch(0.50, 0.19, 55.0), oklch(0.50, 0.19, 55.0)),
            (Self::Pink, Dark) => (oklch(0.72, 0.18, 350.0), oklch(0.51, 0.20, 350.0)),
            (Self::Pink, Light) => (oklch(0.48, 0.20, 350.0), oklch(0.48, 0.20, 350.0)),
        }
    }
}

/// Semantic color roles. Names describe purpose, never a component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colors {
    // ---- surfaces, lowest to highest plane ----
    /// Main content plane.
    pub background: Hsla,
    /// Chrome: sidebars, toolbars, status bars.
    pub surface: Hsla,
    /// Opaque plates that sit proud of the content plane (chips, secondary buttons).
    pub surface_raised: Hsla,
    /// Hover tone for [`Self::surface_raised`]; brightens in dark, darkens in light.
    pub surface_raised_hover: Hsla,
    /// Modal dialog plate, floating over [`Self::scrim`].
    pub surface_dialog: Hsla,
    /// Popover and menu plate: the highest plane.
    pub surface_overlay: Hsla,

    // ---- interaction washes (translucent, layered over any surface) ----
    pub hover: Hsla,
    pub active: Hsla,
    /// Persistent selection: current sidebar item, open trigger, chosen row.
    pub selected: Hsla,

    // ---- edges ----
    pub border: Hsla,
    pub border_strong: Hsla,
    /// Keyboard focus indicator.
    pub focus_ring: Hsla,

    // ---- text ----
    /// Body text; ~16:1 on its background in both appearances.
    pub text: Hsla,
    /// Secondary labels, metadata.
    pub text_muted: Hsla,
    /// Placeholders, disabled labels, decorative hints.
    pub text_faint: Hsla,

    // ---- high-contrast solid (primary action plate) ----
    pub solid: Hsla,
    pub solid_hover: Hsla,
    pub on_solid: Hsla,

    // ---- accent ----
    pub accent: Hsla,
    pub accent_strong: Hsla,
    pub accent_wash: Hsla,
    pub on_accent: Hsla,

    // ---- status ----
    pub danger: Hsla,
    pub danger_strong: Hsla,
    pub danger_wash: Hsla,
    pub on_danger: Hsla,
    pub warning: Hsla,
    pub success: Hsla,

    // ---- misc ----
    pub text_selection: Hsla,
    /// Data series, in order: blue, purple, green, orange, pink.
    pub chart: [Hsla; 5],
    /// Modal backdrop.
    pub scrim: Hsla,
    pub shadow: Hsla,
}

/// Layout and typography scale: 32 px controls, 10 px corners on controls
/// and floating surfaces, 18 px on dialogs and content cards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub radius_sm: Pixels,
    pub radius_md: Pixels,
    pub radius_lg: Pixels,
    /// Floating card corners (popovers, menus).
    pub radius_card: Pixels,
    /// Large plate corners (dialogs, content cards, frames).
    pub radius_xl: Pixels,
    /// Inset between a floating card's border and its rows.
    pub card_inset: Pixels,

    pub control_height_sm: Pixels,
    pub control_height_md: Pixels,
    pub control_height_lg: Pixels,

    pub text_xs: Pixels,
    pub text_sm: Pixels,
    pub text_md: Pixels,
    pub text_lg: Pixels,

    pub sidebar_width: Pixels,
    pub panel_header_height: Pixels,
    pub toolbar_height: Pixels,
}

impl Metrics {
    /// Concentric corner radius for rows inside a floating card: the card
    /// radius minus its 1px border and inset.
    pub fn card_row_radius(&self) -> Pixels {
        (self.radius_card - px(1.) - self.card_inset).max(px(0.))
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            radius_sm: px(6.),
            radius_md: px(8.),
            radius_lg: px(10.),
            radius_card: px(10.),
            radius_xl: px(18.),
            card_inset: px(4.),
            control_height_sm: px(28.),
            control_height_md: px(32.),
            control_height_lg: px(36.),
            text_xs: px(12.),
            text_sm: px(13.),
            text_md: px(14.),
            text_lg: px(18.),
            sidebar_width: px(240.),
            panel_header_height: px(44.),
            toolbar_height: px(44.),
        }
    }
}

/// The active theme: appearance, accent, colors, metrics and font families.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub appearance: Appearance,
    pub accent: Accent,
    pub colors: Colors,
    pub metrics: Metrics,
    pub font_sans: SharedString,
    pub font_mono: SharedString,
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Self::dark(Accent::default())
    }
}

impl Theme {
    pub fn new(appearance: Appearance, accent: Accent) -> Self {
        match appearance {
            Appearance::Dark => Self::dark(accent),
            Appearance::Light => Self::light(accent),
        }
    }

    pub fn dark(accent: Accent) -> Self {
        let (primary, strong) = accent.tones(Appearance::Dark);
        let danger = oklch(0.704, 0.191, 22.216);
        let danger_strong = oklch(0.51, 0.20, 25.0);
        Self {
            appearance: Appearance::Dark,
            accent,
            colors: Colors {
                background: grey(0x16),
                surface: grey(0x11),
                surface_raised: grey(0x1b),
                surface_raised_hover: grey(0x24),
                surface_dialog: grey(0x1b),
                surface_overlay: grey(0x1b),
                hover: hsla(0., 0., 1., 0.05),
                active: hsla(0., 0., 1., 0.08),
                selected: hsla(0., 0., 1., 0.07),
                border: hsla(0., 0., 1., 0.07),
                border_strong: hsla(0., 0., 1., 0.10),
                focus_ring: primary,
                text: neutral(0.97),
                text_muted: neutral(0.70),
                text_faint: neutral(0.58),
                solid: neutral(0.97),
                solid_hover: neutral(0.90),
                on_solid: neutral(0.269),
                accent: primary,
                accent_strong: strong,
                accent_wash: primary.opacity(0.16),
                on_accent: neutral(0.985),
                danger,
                danger_strong,
                danger_wash: danger.opacity(0.14),
                on_danger: neutral(0.985),
                warning: oklch(0.828, 0.189, 84.429),
                success: oklch(0.765, 0.177, 163.223),
                text_selection: primary.opacity(0.35),
                chart: [
                    oklch(0.68, 0.17, 252.),
                    oklch(0.63, 0.2, 293.),
                    oklch(0.75, 0.18, 152.),
                    oklch(0.77, 0.16, 62.),
                    oklch(0.71, 0.19, 355.),
                ],
                scrim: hsla(0., 0., 0., 0.48),
                shadow: hsla(0., 0., 0., 0.45),
            },
            metrics: Metrics::default(),
            font_sans: default_sans(),
            font_mono: default_mono(),
        }
    }

    pub fn light(accent: Accent) -> Self {
        let (primary, strong) = accent.tones(Appearance::Light);
        let danger = oklch(0.577, 0.245, 27.325);
        let danger_strong = oklch(0.51, 0.20, 25.0);
        Self {
            appearance: Appearance::Light,
            accent,
            colors: Colors {
                background: grey(0xff),
                surface: neutral(0.985),
                surface_raised: grey(0xff),
                surface_raised_hover: neutral(0.97),
                surface_dialog: grey(0xff),
                surface_overlay: grey(0xff),
                hover: hsla(0., 0., 0., 0.04),
                active: hsla(0., 0., 0., 0.07),
                selected: hsla(0., 0., 0., 0.055),
                border: hsla(0., 0., 0., 0.08),
                border_strong: hsla(0., 0., 0., 0.12),
                focus_ring: primary,
                text: neutral(0.269),
                text_muted: neutral(0.48),
                text_faint: neutral(0.60),
                solid: neutral(0.269),
                solid_hover: neutral(0.33),
                on_solid: neutral(0.985),
                accent: primary,
                accent_strong: strong,
                accent_wash: primary.opacity(0.10),
                on_accent: neutral(0.985),
                danger,
                danger_strong,
                danger_wash: danger.opacity(0.10),
                on_danger: neutral(0.985),
                warning: oklch(0.555, 0.163, 48.998),
                success: oklch(0.596, 0.145, 163.225),
                text_selection: primary.opacity(0.24),
                chart: [
                    oklch(0.58, 0.19, 255.),
                    oklch(0.54, 0.22, 293.),
                    oklch(0.6, 0.16, 152.),
                    oklch(0.66, 0.17, 55.),
                    oklch(0.58, 0.21, 355.),
                ],
                scrim: hsla(0., 0., 0., 0.32),
                shadow: hsla(0., 0., 0., 0.14),
            },
            metrics: Metrics::default(),
            font_sans: default_sans(),
            font_mono: default_mono(),
        }
    }

    /// The active theme, or the default dark theme when none is installed.
    pub fn global(cx: &App) -> &Theme {
        static FALLBACK: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();
        cx.try_global::<Theme>()
            .unwrap_or_else(|| FALLBACK.get_or_init(Theme::default))
    }

    /// Install as the active theme and project the semantic tokens into the
    /// foundation layer so its behavior modules (scrollbars, resize handles)
    /// share the palette. Redraws every window.
    pub fn install(self, cx: &mut App) {
        let base = gpui_base::Theme::global_mut(cx);
        self.project_into(&mut base.tokens);
        base.resizable.handle = self.colors.border;
        base.resizable.active_handle = self.colors.focus_ring;
        cx.set_global(self);
        cx.refresh_windows();
    }

    fn project_into(&self, tokens: &mut gpui_base::SemanticThemeTokens) {
        let c = &self.colors;
        let m = &self.metrics;
        tokens.colors = gpui_base::ColorTokens {
            background: c.background,
            foreground: c.text,
            surface: c.surface,
            surface_foreground: c.text,
            primary: c.solid,
            primary_foreground: c.on_solid,
            secondary: c.surface_raised,
            secondary_foreground: c.text,
            muted: c.surface,
            muted_foreground: c.text_muted,
            accent: c.accent_strong,
            accent_foreground: c.on_accent,
            destructive: c.danger_strong,
            destructive_foreground: c.on_danger,
            border: c.border,
            input: c.border_strong,
            ring: c.focus_ring,
        };
        tokens.radius.sm = m.radius_sm;
        tokens.radius.md = m.radius_md;
        tokens.radius.lg = m.radius_lg;
        tokens.radius.xl = m.radius_card;
        tokens.typography.sans = self.font_sans.clone();
        tokens.typography.mono = self.font_mono.clone();
        tokens.shadow = gpui_base::ShadowTokens::elevations(c.shadow);
    }

    /// Shadow for floating cards (popovers, menus, dialogs): a soft lift
    /// plus a one-pixel edge below (above in dark, where light comes from
    /// the surface itself).
    pub fn overlay_shadow(&self) -> Vec<BoxShadow> {
        let dark = self.appearance.is_dark();
        vec![
            shadow(
                self.colors.shadow.opacity(if dark { 0.5 } else { 0.35 }),
                10.,
                15.,
                -3.,
                false,
            ),
            shadow(
                self.colors.shadow.opacity(if dark { 0.4 } else { 0.25 }),
                4.,
                6.,
                -4.,
                false,
            ),
            self.edge(),
        ]
    }

    /// Resting shadow for bordered controls and content cards.
    pub fn control_shadow(&self) -> Vec<BoxShadow> {
        vec![
            shadow(self.colors.shadow.opacity(0.35), 1., 2., 0., false),
            self.edge(),
        ]
    }

    /// Top highlight and lift for filled buttons (solid, accent, danger).
    pub fn filled_shadow(&self, fill: Hsla) -> Vec<BoxShadow> {
        vec![
            shadow(hsla(0., 0., 1., 0.16), 1., 0., 0., true),
            shadow(fill.opacity(0.24), 1., 2., 0., false),
        ]
    }

    /// A hairline that separates a plate from the plane beneath: dark below
    /// in light, faint white above in dark.
    fn edge(&self) -> BoxShadow {
        if self.appearance.is_dark() {
            shadow(hsla(0., 0., 1., 0.06), 1., 0., 0., true)
        } else {
            shadow(hsla(0., 0., 0., 0.04), 1., 0., 0., false)
        }
    }
}

fn shadow(color: Hsla, y: f32, blur: f32, spread: f32, inset: bool) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset,
    }
}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        Theme::global(self)
    }
}

fn default_sans() -> SharedString {
    ".SystemUIFont".into()
}

fn default_mono() -> SharedString {
    if cfg!(target_os = "macos") {
        "Menlo".into()
    } else if cfg!(target_os = "windows") {
        "Consolas".into()
    } else {
        "DejaVu Sans Mono".into()
    }
}

// ---------------------------------------------------------------------------
// Color math
// ---------------------------------------------------------------------------

/// An exact achromatic tone from an 8-bit channel value (`grey(13)` is `#0d0d0d`).
pub fn grey(value: u8) -> Hsla {
    hsla(0., 0., value as f32 / 255., 1.)
}

/// A neutral (chroma 0) OKLCH tone. Chroma 0 keeps r == g == b exactly, so
/// this skips hue math and its float-noise saturation.
pub fn neutral(lightness: f32) -> Hsla {
    let [v, _, _] = oklch_to_srgb(lightness, 0., 0.);
    hsla(0., 0., v, 1.)
}

/// Convert an OKLCH color (CSS notation: L 0..1, C, H in degrees) to [`Hsla`].
pub fn oklch(l: f32, c: f32, h_deg: f32) -> Hsla {
    let [r, g, b] = oklch_to_srgb(l, c, h_deg);
    let (h, s, l) = rgb_to_hsl(r, g, b);
    hsla(h, s, l, 1.)
}

/// OKLCH to gamma-encoded sRGB (each 0..1, clipped per channel), using the
/// OKLab matrices from CSS Color 4.
fn oklch_to_srgb(l: f32, c: f32, h_deg: f32) -> [f32; 3] {
    let h = h_deg.to_radians();
    let a = c * h.cos();
    let b = c * h.sin();

    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);

    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_93 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;

    [gamma_encode(r), gamma_encode(g), gamma_encode(b)]
}

fn gamma_encode(x: f32) -> f32 {
    let x = x.clamp(0., 1.);
    if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1. / 2.4) - 0.055
    }
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.;
    let delta = max - min;
    if delta < f32::EPSILON {
        return (0., 0., l);
    }
    let s = if l > 0.5 {
        delta / (2. - max - min)
    } else {
        delta / (max + min)
    };
    let h = if (max - r).abs() < f32::EPSILON {
        ((g - b) / delta).rem_euclid(6.)
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / delta + 2.
    } else {
        (r - g) / delta + 4.
    } / 6.;
    (h, s, l)
}

/// Mix two colors in HSLA space; `t = 0` is `from`, `t = 1` is `to`.
/// Hue takes the short way around the wheel.
pub fn mix(from: Hsla, to: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0., 1.);
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    // An achromatic endpoint has no meaningful hue: borrow the other's.
    let (h0, h1) = match (from.s < 1e-4, to.s < 1e-4) {
        (true, false) => (to.h, to.h),
        (false, true) => (from.h, from.h),
        _ => (from.h, to.h),
    };
    let mut dh = h1 - h0;
    if dh > 0.5 {
        dh -= 1.;
    } else if dh < -0.5 {
        dh += 1.;
    }
    hsla(
        (h0 + dh * t).rem_euclid(1.),
        lerp(from.s, to.s),
        lerp(from.l, to.l),
        lerp(from.a, to.a),
    )
}

/// WCAG 2 contrast ratio between two opaque colors (1.0 to 21.0).
/// Translucent foregrounds should be composited first with [`over`].
pub fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Composite a (possibly translucent) color over an opaque backdrop.
pub fn over(top: Hsla, backdrop: Hsla) -> Hsla {
    let (t, b) = (top.to_rgb(), backdrop.to_rgb());
    let a = top.a;
    gpui::Rgba {
        r: t.r * a + b.r * (1. - a),
        g: t.g * a + b.g * (1. - a),
        b: t.b * a + b.b * (1. - a),
        a: 1.,
    }
    .into()
}

fn relative_luminance(color: Hsla) -> f32 {
    let rgb = color.to_rgb();
    let channel = |c: f32| {
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(rgb.r) + 0.7152 * channel(rgb.g) + 0.0722 * channel(rgb.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn both() -> [Theme; 2] {
        [Theme::dark(Accent::Indigo), Theme::light(Accent::Indigo)]
    }

    #[test]
    fn body_and_secondary_text_meet_wcag_on_every_plane() {
        for theme in both() {
            let c = theme.colors;
            for (name, plane) in [
                ("background", c.background),
                ("surface", c.surface),
                ("dialog", c.surface_dialog),
                ("overlay", c.surface_overlay),
            ] {
                let body = contrast_ratio(c.text, plane);
                let muted = contrast_ratio(c.text_muted, plane);
                let faint = contrast_ratio(c.text_faint, plane);
                assert!(body >= 12., "{:?} text on {name}: {body}", theme.appearance);
                assert!(
                    muted >= 4.5,
                    "{:?} muted on {name}: {muted}",
                    theme.appearance
                );
                assert!(
                    faint >= 3.0,
                    "{:?} faint on {name}: {faint}",
                    theme.appearance
                );
            }
        }
    }

    #[test]
    fn filled_action_labels_meet_aa_for_every_accent() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            for accent in Accent::ALL {
                let c = Theme::new(appearance, accent).colors;
                let on_accent = contrast_ratio(c.on_accent, c.accent_strong);
                assert!(
                    on_accent >= 4.5,
                    "{appearance:?}/{accent:?} on_accent: {on_accent}"
                );
                let on_solid = contrast_ratio(c.on_solid, c.solid);
                assert!(on_solid >= 7., "{appearance:?} on_solid: {on_solid}");
                let on_danger = contrast_ratio(c.on_danger, c.danger_strong);
                assert!(on_danger >= 4.5, "{appearance:?} on_danger: {on_danger}");
            }
        }
    }

    #[test]
    fn focus_ring_is_visible_against_content_and_chrome() {
        // WCAG 1.4.11: non-text indicators need 3:1 against adjacent colors.
        for appearance in [Appearance::Dark, Appearance::Light] {
            for accent in Accent::ALL {
                let c = Theme::new(appearance, accent).colors;
                for plane in [c.background, c.surface] {
                    let ratio = contrast_ratio(c.focus_ring, plane);
                    assert!(ratio >= 3., "{appearance:?}/{accent:?} ring: {ratio}");
                }
            }
        }
    }

    #[test]
    fn selected_wash_is_distinguishable_from_hover() {
        // Selection is persistent state; it must not read as a passing hover.
        for theme in both() {
            let c = theme.colors;
            let hover = over(c.hover, c.surface);
            let selected = over(c.selected, c.surface);
            assert!(relative_luminance(hover) != relative_luminance(selected));
        }
    }

    #[test]
    fn neutral_scale_is_achromatic_and_monotonic() {
        let mut previous = -1.;
        for step in 0..=20 {
            let tone = neutral(step as f32 / 20.);
            assert_eq!(tone.s, 0.);
            assert!(tone.l >= previous);
            previous = tone.l;
        }
        assert_eq!(grey(0xff), hsla(0., 0., 1., 1.));
    }

    #[test]
    fn mix_takes_the_short_hue_path_and_hits_endpoints() {
        let red = hsla(0.98, 0.8, 0.5, 1.);
        let orange = hsla(0.06, 0.8, 0.5, 1.);
        let mid = mix(red, orange, 0.5);
        assert!((mid.h - 0.02).abs() < 1e-4, "hue {}", mid.h);
        assert_eq!(mix(red, orange, 0.), red);
        let end = mix(red, orange, 1.);
        assert!((end.h - orange.h).abs() < 1e-5);
    }

    #[test]
    fn projected_foundation_tokens_follow_semantic_roles() {
        let theme = Theme::light(Accent::Green);
        let mut tokens = gpui_base::SemanticThemeTokens::default();
        theme.project_into(&mut tokens);
        assert_eq!(tokens.colors.ring, theme.colors.focus_ring);
        assert_eq!(tokens.colors.destructive, theme.colors.danger_strong);
        assert_eq!(tokens.radius.md, theme.metrics.radius_md);
    }
}
