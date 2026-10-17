//! Motion catalog and reduced-motion policy.
//!
//! Durations and curves are named once here and reused, so every surface
//! animates with the same vocabulary:
//!
//! | spec            | duration | curve           | use                         |
//! |-----------------|----------|-----------------|-----------------------------|
//! | [`ENTER`]       | 320ms    | expo-out        | content entrances           |
//! | [`FADE_QUICK`]  | 150ms    | ease            | opacity-only swaps          |
//! | [`MENU_IN`]     | 140ms    | ease            | popovers, menus             |
//! | [`MENU_OUT`]    | 100ms    | ease            | popover exits               |
//! | [`DIALOG_IN`]   | 180ms    | ease            | modal dialogs               |
//! | [`RESIZE`]      | 200ms    | ease-out        | sidebar and pane sizes      |
//! | [`COLLAPSE`]    | 180ms    | ease-out        | disclosure height           |
//! | [`HOVER_FADE`]  | 150ms    | standard        | color transitions           |
//! | [`PULSE`]       | 2400ms   | (per-cell)      | ambient loaders             |
//! | [`SPIN`]        | 750ms    | (per-cell)      | activity indicators         |
//!
//! Custom curves are CSS `cubic-bezier()` functions evaluated exactly by
//! [`CubicBezier`].
//!
//! # Reduced motion
//!
//! gpui's [`App::reduce_motion`] flag is honored automatically by every
//! `with_animation` element: oneshot animations snap to their end state,
//! repeating ones to their start state, and no frames are scheduled. The
//! [`MotionPolicy`] global decides that flag from the user's
//! [`MotionPreference`], the operating-system setting, and (optionally)
//! whether the main window is focused.
//!
//! The toolkit never reads the operating system itself: platform layers
//! report it with [`set_system_reduces_motion`], which keeps this module
//! identical on native and browser targets.

use std::time::Duration;

use gpui::{
    Animation, AnimationElement, AnimationExt as _, App, ElementId, Global, IntoElement, Styled, px,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Cubic bezier
// ---------------------------------------------------------------------------

/// A CSS `cubic-bezier(x1, y1, x2, y2)` timing function with endpoints fixed
/// at (0,0) and (1,1). Evaluation solves `x(t) = input` by Newton iteration
/// with a bisection fallback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    fn coefficients(a: f32, b: f32) -> (f32, f32, f32) {
        let c = 3. * a;
        let bb = 3. * (b - a) - c;
        let aa = 1. - c - bb;
        (aa, bb, c)
    }

    fn sample_x(&self, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(self.x1, self.x2);
        ((a * t + b) * t + c) * t
    }

    fn sample_y(&self, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(self.y1, self.y2);
        ((a * t + b) * t + c) * t
    }

    fn sample_x_derivative(&self, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(self.x1, self.x2);
        (3. * a * t + 2. * b) * t + c
    }

    fn solve_t_for_x(&self, x: f32) -> f32 {
        let mut t = x;
        for _ in 0..8 {
            let err = self.sample_x(t) - x;
            if err.abs() < 1e-6 {
                return t;
            }
            let d = self.sample_x_derivative(t);
            if d.abs() < 1e-6 {
                break;
            }
            t -= err / d;
        }
        // x(t) is monotonic for valid CSS curves, so bisection converges.
        let (mut lo, mut hi) = (0_f32, 1_f32);
        for _ in 0..32 {
            let mid = (lo + hi) / 2.;
            if self.sample_x(mid) < x {
                lo = mid
            } else {
                hi = mid
            }
        }
        (lo + hi) / 2.
    }

    /// Eased output for input progress `x` (clamped to `0..=1`).
    pub fn eval(&self, x: f32) -> f32 {
        if x <= 0. {
            return 0.;
        }
        if x >= 1. {
            return 1.;
        }
        // f32 rounding can push the sample a hair past 1.0, and gpui's
        // animation element requires a delta within 0..=1.
        self.sample_y(self.solve_t_for_x(x)).clamp(0., 1.)
    }

    /// This curve as a gpui easing closure.
    pub fn easing(self) -> impl Fn(f32) -> f32 + 'static {
        move |x| self.eval(x)
    }
}

/// `cubic-bezier(0.16, 1, 0.3, 1)`: fast, soft-landing entrances.
pub const EASE_OUT_EXPO: CubicBezier = CubicBezier::new(0.16, 1., 0.3, 1.);
/// CSS `ease-out`.
pub const EASE_OUT: CubicBezier = CubicBezier::new(0., 0., 0.58, 1.);
/// CSS `ease`.
pub const EASE: CubicBezier = CubicBezier::new(0.25, 0.1, 0.25, 1.);
/// CSS `ease-in-out`.
pub const EASE_IN_OUT: CubicBezier = CubicBezier::new(0.42, 0., 0.58, 1.);
/// `cubic-bezier(0.22, 1, 0.36, 1)`: list reorders and glides.
pub const EASE_OUT_QUINT: CubicBezier = CubicBezier::new(0.22, 1., 0.36, 1.);
/// `cubic-bezier(0.4, 0, 0.2, 1)`: the standard color-transition curve.
pub const EASE_STANDARD: CubicBezier = CubicBezier::new(0.4, 0., 0.2, 1.);

// ---------------------------------------------------------------------------
// Motion specs
// ---------------------------------------------------------------------------

/// Duration, optional delay and curve. gpui's [`Animation`] has no native
/// delay, so the delay is folded into the timeline: the animation runs for
/// `delay + duration` and [`progress`](Self::progress) holds 0 until the delay
/// has elapsed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSpec {
    pub duration_ms: u64,
    pub delay_ms: u64,
    pub curve: CubicBezier,
}

impl MotionSpec {
    pub const fn new(duration_ms: u64, curve: CubicBezier) -> Self {
        Self {
            duration_ms,
            delay_ms: 0,
            curve,
        }
    }

    pub const fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Wall-clock span of the whole timeline (delay + duration).
    pub fn total(&self) -> Duration {
        Duration::from_millis(self.delay_ms + self.duration_ms)
    }

    /// Eased progress for a raw timeline delta (both `0..=1`).
    pub fn progress(&self, raw_delta: f32) -> f32 {
        let total = (self.delay_ms + self.duration_ms) as f32;
        if total <= 0. || self.duration_ms == 0 {
            return 1.;
        }
        let t = (raw_delta.clamp(0., 1.) * total - self.delay_ms as f32) / self.duration_ms as f32;
        self.curve.eval(t.clamp(0., 1.))
    }

    /// A oneshot gpui [`Animation`] for this spec.
    pub fn animation(&self) -> Animation {
        let spec = *self;
        Animation::new(spec.total()).with_easing(move |d| spec.progress(d))
    }

    /// A repeating, linear gpui [`Animation`] over the raw period, for loaders
    /// whose per-cell easing happens in the animator.
    pub fn repeating(&self) -> Animation {
        Animation::new(self.total()).repeat()
    }
}

pub const ENTER: MotionSpec = MotionSpec::new(320, EASE_OUT_EXPO);
pub const FADE_QUICK: MotionSpec = MotionSpec::new(150, EASE);
pub const MENU_IN: MotionSpec = MotionSpec::new(140, EASE);
pub const MENU_OUT: MotionSpec = MotionSpec::new(100, EASE);
pub const DIALOG_IN: MotionSpec = MotionSpec::new(180, EASE);
pub const RESIZE: MotionSpec = MotionSpec::new(200, EASE_OUT);
pub const COLLAPSE: MotionSpec = MotionSpec::new(180, EASE_OUT);
pub const HOVER_FADE: MotionSpec = MotionSpec::new(150, EASE_STANDARD);
/// Ambient loader period; cells are eased individually.
pub const PULSE: MotionSpec = MotionSpec::new(2400, EASE);
/// Activity indicator period; cells are eased individually.
pub const SPIN: MotionSpec = MotionSpec::new(750, EASE);

// ---------------------------------------------------------------------------
// Element helpers
// ---------------------------------------------------------------------------
//
// Vertical travel is a relative `top` inset: taffy applies relative insets
// after layout, so, like a CSS transform, siblings never move. Divs have no
// scale transform, so popover and dialog "pop" is approximated with fade and
// a short rise.

/// Content entrance: opacity 0 to 1 with a 4px rise over [`ENTER`].
pub fn fade_in<E>(id: impl Into<ElementId>, element: E) -> AnimationElement<E>
where
    E: Styled + IntoElement + 'static,
{
    element.with_animation(id, ENTER.animation(), |el, t| {
        el.relative().opacity(t).top(px(4. * (1. - t)))
    })
}

/// Opacity-only fade over [`FADE_QUICK`].
pub fn fade_quick<E>(id: impl Into<ElementId>, element: E) -> AnimationElement<E>
where
    E: Styled + IntoElement + 'static,
{
    element.with_animation(id, FADE_QUICK.animation(), |el, t| el.opacity(t))
}

/// Popover entrance travelling from the trigger's side: `from` is the signed
/// starting offset in pixels (negative for surfaces below their trigger,
/// positive for surfaces that open upward).
pub fn menu_in<E>(id: impl Into<ElementId>, from: f32, element: E) -> AnimationElement<E>
where
    E: Styled + IntoElement + 'static,
{
    element.with_animation(id, MENU_IN.animation(), move |el, t| {
        el.relative()
            .opacity(0.3 + 0.7 * t)
            .top(px(from * (1. - t)))
    })
}

/// Dialog entrance: fade with a 2px rise over [`DIALOG_IN`].
pub fn dialog_in<E>(id: impl Into<ElementId>, element: E) -> AnimationElement<E>
where
    E: Styled + IntoElement + 'static,
{
    element.with_animation(id, DIALOG_IN.animation(), |el, t| {
        el.relative().opacity(t).top(px(2. * (1. - t)))
    })
}

/// Linear interpolation.
pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

// ---------------------------------------------------------------------------
// Reduced-motion policy
// ---------------------------------------------------------------------------

/// The user's reduced-motion preference; serializable for settings files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MotionPreference {
    /// Follow the operating-system accessibility setting.
    #[default]
    System,
    /// Always reduce motion.
    Reduced,
    /// Always animate, regardless of the system setting.
    Full,
}

impl MotionPreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Reduced, Self::Full];

    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Reduced => "Reduced",
            Self::Full => "Full",
        }
    }
}

/// Inputs to the global reduce-motion flag. The system value is kept apart
/// from the preference so an OS change while the user has pinned a choice
/// still lands, and takes effect when they return to [`MotionPreference::System`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionPolicy {
    pub preference: MotionPreference,
    /// Also stop animating while the main window is in the background.
    pub pause_in_background: bool,
    /// Last value reported by the platform layer.
    pub system_reduces_motion: bool,
    /// Whether the main window is focused.
    pub window_active: bool,
}

impl Default for MotionPolicy {
    fn default() -> Self {
        Self {
            preference: MotionPreference::System,
            pause_in_background: false,
            system_reduces_motion: false,
            window_active: true,
        }
    }
}

impl Global for MotionPolicy {}

impl MotionPolicy {
    /// Whether motion should be reduced under this policy.
    pub fn reduces_motion(&self) -> bool {
        let reduced = match self.preference {
            MotionPreference::System => self.system_reduces_motion,
            MotionPreference::Reduced => true,
            MotionPreference::Full => false,
        };
        reduced || (self.pause_in_background && !self.window_active)
    }

    /// Whether only the operating-system setting (not the user or a
    /// background pause) is reducing motion. Activity indicators use this to
    /// keep a gentle, non-travelling brightness cue: progress feedback is
    /// essential, decoration is not.
    pub fn is_system_reduction_only(&self) -> bool {
        self.preference == MotionPreference::System
            && self.system_reduces_motion
            && !(self.pause_in_background && !self.window_active)
    }

    /// The installed policy, or the default when none is installed.
    pub fn global(cx: &App) -> MotionPolicy {
        cx.try_global::<MotionPolicy>().copied().unwrap_or_default()
    }

    /// Install this policy and apply it to the app-wide flag. Call before the
    /// first window opens so boot animations already honor it.
    pub fn install(self, cx: &mut App) {
        cx.set_global(self);
        apply(cx);
    }
}

fn update(cx: &mut App, f: impl FnOnce(&mut MotionPolicy)) {
    let mut policy = MotionPolicy::global(cx);
    f(&mut policy);
    policy.install(cx);
}

fn apply(cx: &mut App) {
    let reduced = MotionPolicy::global(cx).reduces_motion();
    // gpui refreshes windows only when the flag actually flips.
    cx.set_reduce_motion(reduced);
}

pub fn set_preference(preference: MotionPreference, cx: &mut App) {
    update(cx, |policy| policy.preference = preference);
}

pub fn set_pause_in_background(enabled: bool, cx: &mut App) {
    update(cx, |policy| policy.pause_in_background = enabled);
}

/// Report the operating-system reduced-motion setting. Platform layers call
/// this at startup and whenever the setting may have changed.
pub fn set_system_reduces_motion(reduces: bool, cx: &mut App) {
    update(cx, |policy| policy.system_reduces_motion = reduces);
}

/// Report main-window focus changes, for [`MotionPolicy::pause_in_background`].
pub fn window_activation_changed(active: bool, cx: &mut App) {
    update(cx, |policy| policy.window_active = active);
}

/// Whether motion is currently reduced (the app-wide gpui flag).
pub fn reduced(cx: &App) -> bool {
    cx.reduce_motion()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_hits_endpoints_and_matches_css_reference_values() {
        for curve in [EASE, EASE_OUT, EASE_IN_OUT, EASE_OUT_EXPO, EASE_STANDARD] {
            assert_eq!(curve.eval(0.), 0.);
            assert_eq!(curve.eval(1.), 1.);
            assert_eq!(curve.eval(-1.), 0.);
            assert_eq!(curve.eval(2.), 1.);
        }
        // Browser-computed reference values for CSS `ease`.
        assert!((EASE.eval(0.25) - 0.4085).abs() < 2e-3);
        assert!((EASE.eval(0.5) - 0.8024).abs() < 2e-3);
        // `ease-in-out` is symmetric around the midpoint.
        assert!((EASE_IN_OUT.eval(0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn bezier_output_is_monotonic_and_bounded() {
        for curve in [EASE, EASE_OUT_EXPO, EASE_OUT_QUINT] {
            let mut previous = 0.;
            for step in 0..=200 {
                let y = curve.eval(step as f32 / 200.);
                assert!((0. ..=1.).contains(&y));
                assert!(y + 1e-5 >= previous, "{curve:?} dipped at {step}");
                previous = y;
            }
        }
    }

    #[test]
    fn delay_holds_progress_at_zero_then_eases_to_one() {
        let spec = MotionSpec::new(100, EASE_OUT).with_delay(100);
        assert_eq!(spec.total(), Duration::from_millis(200));
        assert_eq!(spec.progress(0.25), 0.);
        assert_eq!(spec.progress(0.5), 0.);
        assert!(spec.progress(0.75) > 0. && spec.progress(0.75) < 1.);
        assert_eq!(spec.progress(1.), 1.);
        assert_eq!(MotionSpec::new(0, EASE).progress(0.), 1.);
    }

    #[test]
    fn policy_resolves_preference_system_and_background() {
        let policy = |preference, system, pause, active| MotionPolicy {
            preference,
            pause_in_background: pause,
            system_reduces_motion: system,
            window_active: active,
        };
        use MotionPreference::*;
        assert!(!policy(System, false, false, true).reduces_motion());
        assert!(policy(System, true, false, true).reduces_motion());
        assert!(policy(Reduced, false, false, true).reduces_motion());
        assert!(!policy(Full, true, false, true).reduces_motion());
        // Background pause overrides even an explicit Full preference.
        assert!(policy(Full, false, true, false).reduces_motion());
        assert!(!policy(Full, false, true, true).reduces_motion());

        assert!(policy(System, true, false, true).is_system_reduction_only());
        assert!(!policy(Reduced, true, false, true).is_system_reduction_only());
        assert!(!policy(System, true, true, false).is_system_reduction_only());
    }

    #[gpui::test]
    fn policy_drives_the_app_wide_animation_flag(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            MotionPolicy::default().install(cx);
            assert!(!reduced(cx));

            set_system_reduces_motion(true, cx);
            assert!(reduced(cx));

            set_preference(MotionPreference::Full, cx);
            assert!(!reduced(cx));

            set_pause_in_background(true, cx);
            window_activation_changed(false, cx);
            assert!(reduced(cx));
            window_activation_changed(true, cx);
            assert!(!reduced(cx));
        });
    }
}
