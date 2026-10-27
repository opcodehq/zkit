//! Progress and loading indicators.
//!
//! - [`ProgressBar`]: determinate or indeterminate linear progress, exposed
//!   as a progress indicator with its numeric value.
//! - [`Spinner`]: a compact activity grid for "working" states.
//! - [`PulseLoader`]: a calm row of breathing cells for ambient loading.
//!
//! Repeating motion is driven by the shared throttled clock in
//! [`crate::pulse`] rather than per-frame animations, and every indicator
//! has a still state under reduced motion.

use gpui::{
    App, Bounds, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    PathBuilder, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled,
    Window, canvas, div, point, prelude::FluentBuilder as _, px, relative,
};
use std::f32::consts::TAU;

use crate::motion::{EASE_IN_OUT, MotionSpec, PULSE};
use crate::pulse::{self, Rate};
use crate::theme::ActiveTheme as _;

/// One sweep of the indeterminate bar.
const SWEEP: MotionSpec = MotionSpec::new(1400, EASE_IN_OUT);
/// Width of the indeterminate segment, as a fraction of the track.
const SWEEP_WIDTH: f32 = 0.35;

/// Linear progress built on the foundation progress semantics.
#[derive(IntoElement)]
pub struct ProgressBar {
    id: ElementId,
    value: Option<f32>,
    label: Option<SharedString>,
    color: Option<Hsla>,
    height: Pixels,
}

impl ProgressBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            label: None,
            color: None,
            height: px(6.),
        }
    }

    /// Percentage complete, clamped to `0..=100`. Without a value the bar is
    /// indeterminate.
    pub fn value(mut self, percent: f32) -> Self {
        self.value = Some(percent.clamp(0., 100.));
        self
    }

    /// Accessible name, e.g. "Indexing workspace".
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }
}

/// Left edge and width (fractions of the track) of the indeterminate
/// segment at an eased phase. The segment enters from beyond the left edge
/// and exits past the right, so the sweep never pauses fully visible.
pub fn sweep_segment(eased: f32) -> (f32, f32) {
    let left = -SWEEP_WIDTH + (1. + SWEEP_WIDTH) * eased.clamp(0., 1.);
    (left, SWEEP_WIDTH)
}

impl RenderOnce for ProgressBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = window.current_view();
        let reduced = cx.reduce_motion();
        let raw = match self.value {
            None => pulse::phase(&SWEEP, view, Rate::Smooth, cx),
            Some(_) => 0.,
        };
        let theme = cx.theme();
        let fill = self.color.unwrap_or(theme.colors.accent);
        let radius = self.height / 2.;

        let indicator = match self.value {
            Some(percent) => div()
                .h_full()
                .w(relative(percent / 100.))
                .rounded(radius)
                .bg(fill),
            // Still state: a quiet full-width fill reads as "busy" without motion.
            None if reduced => div().size_full().rounded(radius).bg(fill.opacity(0.4)),
            None => {
                let (left, width) = sweep_segment(SWEEP.progress(raw));
                div()
                    .absolute()
                    .top_0()
                    .h_full()
                    .left(relative(left))
                    .w(relative(width))
                    .rounded(radius)
                    .bg(fill)
            }
        };

        gpui_base::Progress::new(self.id)
            .when_some(self.value, |this, value| this.value(value))
            .indeterminate(self.value.is_none())
            .when_some(self.label, |this, label| this.aria_label(label))
            .when(self.value.is_none(), |this| {
                this.a11y_synthetic_children(|tree| tree.parent_node().set_busy())
            })
            .relative()
            .w_full()
            .h(self.height)
            .rounded(radius)
            .overflow_hidden()
            .bg(theme.colors.border_strong)
            .child(indicator)
    }
}

/// Compact activity indicator: a 3x3 grid whose brightness travels upward.
/// Decorative by default; give it an id and label when it is the only cue.
#[derive(IntoElement)]
pub struct Spinner {
    size: Pixels,
    color: Option<Hsla>,
    accessible: Option<(ElementId, SharedString)>,
}

/// One full turn of the spinner's arc.
const ROTATE: MotionSpec = MotionSpec::new(900, crate::motion::EASE);
/// Share of the ring covered by the moving arc.
const ARC: f32 = 0.25;

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: px(16.),
            color: None,
            accessible: None,
        }
    }

    /// Diameter; the stroke scales with it.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Expose as a busy progress indicator named `label`.
    pub fn accessible(mut self, id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        self.accessible = Some((id.into(), label.into()));
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = window.current_view();
        // A ring that turns at a constant rate. Under a system-only motion
        // reduction the arc holds still and breathes; an explicit preference
        // keeps it fully still.
        let (turn, opacity) = if cx.reduce_motion() {
            (
                0.125,
                pulse::activity(view, Rate::Coarse, cx).opacity(0., 1.),
            )
        } else {
            (pulse::phase(&ROTATE, view, Rate::Smooth, cx), 1.)
        };
        let color = self.color.unwrap_or(cx.theme().colors.accent);
        let size = self.size;

        let ring = div().flex_none().size(size).child(
            canvas(
                |_, _, _| (),
                move |bounds: Bounds<Pixels>, _, window, _| {
                    let d = f32::from(bounds.size.width.min(bounds.size.height));
                    let stroke = (d / 8.).max(1.5);
                    let r = d / 2. - stroke / 2.;
                    let c = bounds.center();
                    let at = |a: f32| point(c.x + px(r * a.sin()), c.y - px(r * a.cos()));
                    let arc = |from: f32, sweep: f32, steps: usize| {
                        let mut path = PathBuilder::stroke(px(stroke));
                        path.move_to(at(from));
                        for i in 1..=steps {
                            path.line_to(at(from + sweep * i as f32 / steps as f32));
                        }
                        path.build().ok()
                    };
                    if let Some(track) = arc(0., TAU, 48) {
                        window.paint_path(track, color.opacity(0.2 * opacity));
                    }
                    if let Some(head) = arc(turn * TAU, ARC * TAU, 16) {
                        window.paint_path(head, color.opacity(opacity));
                    }
                },
            )
            .size_full(),
        );

        match self.accessible {
            Some((id, label)) => div()
                .id(id)
                .role(Role::ProgressIndicator)
                .aria_label(label)
                .a11y_synthetic_children(|tree| tree.parent_node().set_busy())
                .child(ring)
                .into_any_element(),
            None => ring.into_any_element(),
        }
    }
}

/// Ambient loader: a row of cells breathing in a left-to-right wave.
#[derive(IntoElement)]
pub struct PulseLoader {
    cells: usize,
    cell: Pixels,
    color: Option<Hsla>,
}

impl PulseLoader {
    pub fn new() -> Self {
        Self {
            cells: 5,
            cell: px(6.),
            color: None,
        }
    }

    pub fn cells(mut self, cells: usize) -> Self {
        self.cells = cells.max(1);
        self
    }

    pub fn cell_size(mut self, cell: Pixels) -> Self {
        self.cell = cell;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for PulseLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for PulseLoader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = window.current_view();
        let reduced = cx.reduce_motion();
        let raw = pulse::phase(&PULSE, view, Rate::Coarse, cx);
        let color = self.color.unwrap_or(cx.theme().colors.text_muted);
        let slot = self.cell;
        div()
            .flex()
            .items_center()
            .gap(slot / 2.)
            .children((0..self.cells).map(move |index| {
                // Still state: an even, mid-brightness row.
                let phase = pulse::staggered_phase(raw, index, pulse::PULSE_STAGGER);
                let (opacity, scale) = if reduced {
                    (0.5, 1.)
                } else {
                    (pulse::pulse_opacity(phase), pulse::pulse_scale(phase))
                };
                // Fixed slot: the cell breathes inside it without moving layout.
                div()
                    .size(slot)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .size(slot * scale)
                            .rounded(slot / 4.)
                            .bg(color)
                            .opacity(opacity),
                    )
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indeterminate_segment_sweeps_fully_across_the_track() {
        let (start, width) = sweep_segment(0.);
        assert!(start + width <= 0., "starts hidden");
        let (end, _) = sweep_segment(1.);
        assert!(end >= 1., "ends hidden");
        let (mid, _) = sweep_segment(0.5);
        assert!(mid > 0. && mid + width < 1.);
    }
}
