//! Shared, throttled clock for repeating loaders, plus their pure phase math.
//!
//! A `with_animation(...repeat())` element requests a redraw every display
//! frame for as long as it is mounted, which pins a window at its full
//! refresh rate for a single spinner. Loaders instead read a phase from this
//! clock, which notifies subscribed views at about 30fps (or 15fps for coarse
//! indicators). Each read renews a short lease for the calling view; once no
//! loader is painted, leases expire and the clock parks, so a window without
//! a mounted loader schedules nothing.
//!
//! All loaders share one epoch, so multiple instances stay phase-locked.
//! Every phase function is pure in `0..1`, so the math is testable without a
//! window.

use std::collections::HashMap;
use std::time::Duration;

use gpui::{App, EntityId, Global};
use web_time::Instant;

use crate::motion::{MotionPolicy, MotionSpec, PULSE, SPIN};

/// Tick interval (about 30fps).
const TICK: Duration = Duration::from_millis(33);
/// How long a view stays subscribed after its last loader paint.
const LEASE: Duration = Duration::from_millis(300);

/// Notification rate for a loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rate {
    /// About 30fps, for smooth fades.
    #[default]
    Smooth,
    /// About 15fps, for chunky cell loaders where extra frames are invisible.
    Coarse,
}

impl Rate {
    fn stride(self) -> u64 {
        match self {
            Rate::Smooth => 1,
            Rate::Coarse => 2,
        }
    }
}

struct PulseClock {
    epoch: Instant,
    leases: HashMap<EntityId, Lease>,
    tick: u64,
    running: bool,
}

struct Lease {
    until: Instant,
    stride: u64,
}

impl Lease {
    fn renew(&mut self, now: Instant, stride: u64) {
        self.until = now + LEASE;
        self.stride = self.stride.min(stride);
    }

    fn take_tick(&mut self, tick: u64) -> bool {
        if !tick.is_multiple_of(self.stride) {
            return false;
        }
        // Each paint re-establishes the fastest mounted loader, so a retired
        // smooth loader does not permanently pull a coarse one up to 30fps.
        self.stride = u64::MAX;
        true
    }
}

impl Global for PulseClock {}

impl Default for PulseClock {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            leases: HashMap::new(),
            tick: 0,
            running: false,
        }
    }
}

/// Current phase `0..1` of a repeating spec for `view`, renewing its lease.
/// Returns a static 0 and schedules nothing when motion is reduced.
pub fn phase(spec: &MotionSpec, view: EntityId, rate: Rate, cx: &mut App) -> f32 {
    if cx.reduce_motion() {
        return 0.;
    }
    schedule(view, rate.stride(), cx);
    clock_phase(spec, cx)
}

/// Elapsed seconds for ambient surfaces, sharing the leased animation clock.
/// Reduced motion returns a still timestamp and schedules no work.
pub fn seconds(view: EntityId, rate: Rate, cx: &mut App) -> f32 {
    if cx.reduce_motion() {
        return 0.;
    }
    schedule(view, rate.stride(), cx);
    cx.default_global::<PulseClock>()
        .epoch
        .elapsed()
        .as_secs_f32()
}

fn clock_phase(spec: &MotionSpec, cx: &mut App) -> f32 {
    let clock = cx.default_global::<PulseClock>();
    (clock.epoch.elapsed().as_secs_f32() / spec.total().as_secs_f32()).fract()
}

/// Brightness driver for activity indicators.
///
/// Under a system-only motion reduction, activity keeps a slow, uniform
/// brightness pulse (no travelling chase), because "work is happening" is
/// essential feedback. An explicit user preference or background pause keeps
/// it fully still.
#[derive(Debug, Clone, Copy)]
pub struct Activity {
    phase: f32,
    subtle: bool,
}

impl Activity {
    /// Cell opacity at a per-cell phase offset, resting at `dim`.
    pub fn opacity(&self, cell_phase: f32, dim: f32) -> f32 {
        if self.subtle {
            0.6 + 0.2 * pulse_wave(self.phase)
        } else {
            spin_opacity(self.phase + cell_phase, dim)
        }
    }

    /// Whether the indicator is animating at all.
    pub fn is_animating(&self) -> bool {
        self.phase != 0. || !self.subtle
    }
}

pub fn activity(view: EntityId, rate: Rate, cx: &mut App) -> Activity {
    let subtle = cx.reduce_motion();
    let animate = !subtle || MotionPolicy::global(cx).is_system_reduction_only();
    let phase = if animate {
        let stride = if subtle {
            Rate::Coarse.stride()
        } else {
            rate.stride()
        };
        schedule(view, stride, cx);
        clock_phase(if subtle { &PULSE } else { &SPIN }, cx)
    } else {
        0.
    };
    Activity { phase, subtle }
}

fn schedule(view: EntityId, stride: u64, cx: &mut App) {
    let clock = cx.default_global::<PulseClock>();
    let now = Instant::now();
    clock
        .leases
        .entry(view)
        .or_insert(Lease {
            until: now + LEASE,
            stride,
        })
        .renew(now, stride);
    if clock.running {
        return;
    }
    clock.running = true;
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            let parked = cx.update(|cx| {
                let clock = cx.default_global::<PulseClock>();
                let now = Instant::now();
                clock.leases.retain(|_, lease| lease.until > now);
                if clock.leases.is_empty() {
                    clock.running = false;
                    return true;
                }
                clock.tick = clock.tick.wrapping_add(1);
                let tick = clock.tick;
                let views: Vec<EntityId> = clock
                    .leases
                    .iter_mut()
                    .filter_map(|(view, lease)| lease.take_tick(tick).then_some(*view))
                    .collect();
                for view in views {
                    cx.notify(view);
                }
                false
            });
            if parked {
                break;
            }
        }
    })
    .detach();
}

// ---------------------------------------------------------------------------
// Pure phase math
// ---------------------------------------------------------------------------

/// Ambient loader cells rest at this opacity between pulses.
pub const PULSE_MIN_OPACITY: f32 = 0.12;
/// ...and at this scale.
pub const PULSE_MIN_SCALE: f32 = 0.9;
/// Per-cell stagger as a fraction of the pulse period (0.15s of 2.4s).
pub const PULSE_STAGGER: f32 = 0.15 / 2.4;
/// Opacity an activity cell rests at between pulses.
pub const SPIN_DIM: f32 = 0.15;

/// A cell's phase, given the loader's raw phase and the cell's index.
pub fn staggered_phase(raw: f32, index: usize, stagger: f32) -> f32 {
    (raw - index as f32 * stagger).rem_euclid(1.)
}

/// Cosine pulse: 0 at phase 0, 1 at phase 0.5, back to 0 at phase 1.
pub fn pulse_wave(phase: f32) -> f32 {
    0.5 - 0.5 * (phase * std::f32::consts::TAU).cos()
}

pub fn pulse_opacity(phase: f32) -> f32 {
    PULSE_MIN_OPACITY + (1. - PULSE_MIN_OPACITY) * pulse_wave(phase)
}

pub fn pulse_scale(phase: f32) -> f32 {
    PULSE_MIN_SCALE + (1. - PULSE_MIN_SCALE) * pulse_wave(phase)
}

/// Activity-cell opacity at a local phase: full at the cycle start, easing
/// down to `dim` by 45%, resting until 92%, then rising back to full. A
/// per-cell phase offset sweeps this pulse across a grid.
pub fn spin_opacity(t: f32, dim: f32) -> f32 {
    let t = t.rem_euclid(1.);
    if t < 0.45 {
        crate::motion::lerp(1., dim, t / 0.45)
    } else if t < 0.92 {
        dim
    } else {
        crate::motion::lerp(dim, 1., (t - 0.92) / 0.08)
    }
}

/// Phase offset of a `(row, col)` cell in a `side x side` activity grid. The
/// pulse enters at the bottom edge and converges on the top-centre cell, so
/// the wave reads as travelling upward.
pub fn spin_cell_phase(row: usize, col: usize, side: usize) -> f32 {
    let last = side.saturating_sub(1) as f32;
    let centre = last / 2.;
    let max = last + centre;
    let d = last - row as f32 + (col as f32 - centre).abs();
    if max == 0. { 0. } else { d / (max + 1.) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::AppContext as _;

    #[test]
    fn pulse_wave_peaks_mid_cycle_and_rests_at_the_edges() {
        assert!(pulse_wave(0.).abs() < 1e-6);
        assert!((pulse_wave(0.5) - 1.).abs() < 1e-6);
        assert!(pulse_wave(1.).abs() < 1e-6);
        assert!((pulse_opacity(0.) - PULSE_MIN_OPACITY).abs() < 1e-6);
        assert!((pulse_scale(0.5) - 1.).abs() < 1e-6);
    }

    #[test]
    fn stagger_wraps_into_the_unit_interval() {
        assert!((staggered_phase(0.0, 1, 0.25) - 0.75).abs() < 1e-6);
        assert!((staggered_phase(0.1, 0, 0.25) - 0.1).abs() < 1e-6);
        for index in 0..32 {
            let phase = staggered_phase(0.3, index, PULSE_STAGGER);
            assert!((0. ..1.).contains(&phase));
        }
    }

    #[test]
    fn spin_opacity_is_continuous_across_its_keyframes() {
        let dim = SPIN_DIM;
        assert_eq!(spin_opacity(0., dim), 1.);
        assert!((spin_opacity(0.45, dim) - dim).abs() < 1e-5);
        assert_eq!(spin_opacity(0.7, dim), dim);
        assert!((spin_opacity(0.9999, dim) - 1.).abs() < 1e-2);
        let mut previous = spin_opacity(0., dim);
        for step in 1..=1000 {
            let next = spin_opacity(step as f32 / 1000., dim);
            assert!((next - previous).abs() < 0.02, "jump at {step}");
            previous = next;
        }
    }

    #[test]
    fn spin_wave_travels_upward_toward_the_top_centre() {
        let side = 3;
        let top_centre = spin_cell_phase(0, 1, side);
        let bottom_centre = spin_cell_phase(2, 1, side);
        assert!(bottom_centre < top_centre);
        assert_eq!(spin_cell_phase(2, 0, side), spin_cell_phase(2, 2, side));
        assert_eq!(spin_cell_phase(0, 0, 1), 0.);
    }

    #[test]
    fn lease_keeps_the_fastest_painted_rate_for_one_tick() {
        let now = Instant::now();
        let mut lease = Lease {
            until: now,
            stride: Rate::Coarse.stride(),
        };
        lease.renew(now, Rate::Smooth.stride());
        assert!(lease.take_tick(1));
        // Without a fresh paint, the next tick falls back to "not due".
        assert!(!lease.take_tick(2));
    }

    #[gpui::test]
    fn reduced_motion_freezes_phase_without_scheduling(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let view = cx.new(|_| ()).entity_id();
            cx.set_reduce_motion(true);
            assert_eq!(phase(&PULSE, view, Rate::Smooth, cx), 0.);
            assert!(!cx.default_global::<PulseClock>().running);

            // A user-chosen reduction keeps activity fully still.
            crate::motion::set_preference(crate::motion::MotionPreference::Reduced, cx);
            assert!(!activity(view, Rate::Smooth, cx).is_animating());
            assert!(!cx.default_global::<PulseClock>().running);

            // A system-only reduction keeps a gentle activity cue running.
            crate::motion::set_preference(crate::motion::MotionPreference::System, cx);
            crate::motion::set_system_reduces_motion(true, cx);
            assert!(cx.reduce_motion());
            assert!(activity(view, Rate::Smooth, cx).is_animating());
            assert!(cx.default_global::<PulseClock>().running);
        });
    }
}
