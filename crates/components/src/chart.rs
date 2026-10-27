//! Data charts with a dithered finish: area, line, bar, donut, radar and
//! heatmap, with axes, labels and a legend.
//!
//! Series take the theme's chart colours in order and a fill style each
//! (gradient, hatched, dotted, solid by default), so overlapping series stay
//! distinct. Areas and bars can stack. Values are plotted against rounded
//! ticks; non-finite and negative values count as zero. The marks come from
//! `zkit_dither::Plot`, the same code the website runs in the browser, and
//! the element carries a text summary of every value.
//!
//! Credits: chart styles follow dither-kit by ripgrim (https://tripwire.sh/dither-kit); dithering adapts dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT.

use std::{f32::consts::TAU, rc::Rc};

use gpui::{
    App, Bounds, ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Window, canvas, div, prelude::FluentBuilder as _, px, relative,
};
use gpui_base::StyledExt as _;
use zkit_dither::{Plot, PlotKind};

pub use zkit_dither::ChartFill;

use crate::dither::{paint_raster, theme_bytes};
use crate::layout::{h_stack, v_stack};
use crate::theme::ActiveTheme as _;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartKind {
    #[default]
    Area,
    Line,
    Bar,
    Donut,
    Radar,
    Heatmap,
}

/// One named series of values, aligned with the chart's categories.
#[derive(Clone, Debug)]
pub struct ChartSeries {
    pub name: SharedString,
    pub values: Vec<f32>,
    pub fill: ChartFill,
}

type Format = Rc<dyn Fn(f32) -> String>;

/// A labelled chart. Area, line and bar plot every series against shared
/// categories; a donut plots the first series by category; a radar draws one
/// polygon per series with categories as spokes; a heatmap uses one series
/// per row and categories as columns.
///
/// ```ignore
/// DitherChart::new("visitors", "Visitors")
///     .kind(ChartKind::Area)
///     .stacked(true)
///     .categories(["Jan", "Feb", "Mar", "Apr", "May", "Jun"])
///     .series("Desktop", [186., 305., 237., 273., 209., 314.])
///     .series("Mobile", [80., 200., 120., 190., 130., 140.])
/// ```
#[derive(IntoElement)]
pub struct DitherChart {
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    kind: ChartKind,
    stacked: bool,
    categories: Vec<SharedString>,
    series: Vec<ChartSeries>,
    format: Format,
    legend: bool,
    plot_height: Pixels,
    style: StyleRefinement,
}

impl DitherChart {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            kind: ChartKind::Area,
            stacked: false,
            categories: Vec::new(),
            series: Vec::new(),
            format: Rc::new(compact),
            legend: true,
            plot_height: px(200.),
            style: StyleRefinement::default(),
        }
    }

    /// Supporting text under the title.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn kind(mut self, kind: ChartKind) -> Self {
        self.kind = kind;
        self
    }

    /// Stack areas and bars instead of overlaying or grouping them.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    /// Category labels: the x axis, donut segments, radar spokes or heatmap columns.
    pub fn categories<S: Into<SharedString>>(
        mut self,
        labels: impl IntoIterator<Item = S>,
    ) -> Self {
        self.categories = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Add a named series. It takes the next theme chart colour and fill.
    pub fn series(
        mut self,
        name: impl Into<SharedString>,
        values: impl IntoIterator<Item = f32>,
    ) -> Self {
        let fill = ChartFill::nth(self.series.len());
        self.series.push(ChartSeries {
            name: name.into(),
            values: values
                .into_iter()
                .map(|v| if v.is_finite() { v.max(0.) } else { 0. })
                .collect(),
            fill,
        });
        self
    }

    /// Fill style of the most recently added series.
    pub fn fill(mut self, fill: ChartFill) -> Self {
        if let Some(series) = self.series.last_mut() {
            series.fill = fill;
        }
        self
    }

    /// How values read on the axis, legend and summary. Default: compact
    /// numbers (`1.2k`, `3.4M`).
    pub fn format(mut self, format: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Rc::new(format);
        self
    }

    pub fn legend(mut self, show: bool) -> Self {
        self.legend = show;
        self
    }

    /// Height of the plotting area. Default 200 px.
    pub fn plot_height(mut self, height: Pixels) -> Self {
        self.plot_height = height;
        self
    }

    fn summary(&self) -> String {
        let format = &self.format;
        let series = self
            .series
            .iter()
            .map(|s| {
                let points = s
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match self.categories.get(i) {
                        Some(label) => format!("{label} {}", format(*v)),
                        None => format(*v),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}: {points}", s.name)
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("{}. {series}", self.title)
    }

    /// The axis maximum: the largest value, or the largest stack.
    fn data_max(&self) -> f32 {
        let count = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0);
        if self.stacked && matches!(self.kind, ChartKind::Area | ChartKind::Bar) {
            (0..count)
                .map(|i| {
                    self.series
                        .iter()
                        .map(|s| s.values.get(i).copied().unwrap_or(0.))
                        .sum::<f32>()
                })
                .fold(0., f32::max)
        } else {
            self.series
                .iter()
                .flat_map(|s| s.values.iter().copied())
                .fold(0., f32::max)
        }
    }
}

impl Styled for DitherChart {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// `1234 → 1.2k`, `2_500_000 → 2.5M`; small values keep up to one decimal.
pub fn compact(v: f32) -> String {
    let trim = |x: f32, suffix: &str| {
        let s = format!("{x:.1}");
        format!("{}{suffix}", s.strip_suffix(".0").unwrap_or(&s))
    };
    match v.abs() {
        a if a >= 1e9 => trim(v / 1e9, "B"),
        a if a >= 1e6 => trim(v / 1e6, "M"),
        a if a >= 1e3 => trim(v / 1e3, "k"),
        _ => trim(v, ""),
    }
}

/// Rounded axis ticks from zero covering `max`: steps of 1, 2, 2.5 or 5
/// times a power of ten, aiming for about `count` intervals.
pub fn nice_ticks(max: f32, count: usize) -> Vec<f32> {
    let max = if max.is_finite() && max > 0. { max } else { 1. };
    let rough = max / count.max(1) as f32;
    let magnitude = 10f32.powf(rough.log10().floor());
    let step = [1., 2., 2.5, 5., 10.]
        .into_iter()
        .map(|m| m * magnitude)
        .find(|step| *step >= rough)
        .unwrap_or(10. * magnitude);
    let steps = (max / step - 1e-4).ceil().max(1.) as usize;
    (0..=steps).map(|i| i as f32 * step).collect()
}

/// A canvas that draws the plot's marks at 2 px cells.
fn marks(
    kind: PlotKind,
    series: Vec<Vec<f32>>,
    fills: Vec<ChartFill>,
    stacked: bool,
    top: f32,
    colors: Vec<[u8; 3]>,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds: Bounds<Pixels>, _, window, _| {
            let cell = 2.;
            let cols = (f32::from(bounds.size.width) / cell) as usize;
            let rows = (f32::from(bounds.size.height) / cell) as usize;
            if cols == 0 || rows == 0 {
                return;
            }
            let plot = Plot {
                kind,
                series: &series,
                fills: &fills,
                stacked,
                top,
                progress: 1.,
            };
            if let Ok(raster) = plot.render(cols, rows, &colors) {
                paint_raster(window, bounds.origin, &raster, cell);
            }
        },
    )
    .absolute()
    .inset_0()
}

impl RenderOnce for DitherChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let c = theme.colors;
        let summary = self.summary();
        let n = self.series.len().max(self.categories.len()).max(1);
        let colors: Vec<Hsla> = (0..n).map(|i| c.chart[i % c.chart.len()]).collect();
        let bytes: Vec<[u8; 3]> = colors.iter().map(|c| theme_bytes(*c)).collect();
        let data: Vec<Vec<f32>> = self.series.iter().map(|s| s.values.clone()).collect();
        let fills: Vec<ChartFill> = self.series.iter().map(|s| s.fill).collect();
        let format = self.format.clone();
        let text_xs = theme.metrics.text_xs;
        let mono = theme.font_mono.clone();
        // Axis and legend labels are small monospace text.
        let label = move |text: String, color: Hsla| {
            div()
                .font_family(mono.clone())
                .text_size(text_xs)
                .text_color(color)
                .whitespace_nowrap()
                .child(text)
        };
        let swatch = |color: Hsla| div().flex_none().size(px(8.)).rounded(px(2.)).bg(color);

        let header = v_stack()
            .gap(px(2.))
            .child(
                div()
                    .text_size(theme.metrics.text_md)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(c.text)
                    .child(self.title.clone()),
            )
            .children(
                self.description
                    .clone()
                    .map(|d| label(d.to_string(), c.text_muted)),
            );

        let legend_items: Vec<(SharedString, Hsla)> = match self.kind {
            ChartKind::Donut | ChartKind::Heatmap => Vec::new(),
            _ => self
                .series
                .iter()
                .zip(&colors)
                .map(|(s, c)| (s.name.clone(), *c))
                .collect(),
        };
        let legend = (self.legend && !legend_items.is_empty()).then(|| {
            h_stack()
                .gap(px(14.))
                .flex_wrap()
                .children(legend_items.into_iter().map(|(name, color)| {
                    h_stack()
                        .gap(px(6.))
                        .child(swatch(color))
                        .child(label(name.to_string(), c.text_muted))
                }))
        });

        let body =
            match self.kind {
                ChartKind::Area | ChartKind::Line | ChartKind::Bar => {
                    let ticks = nice_ticks(self.data_max(), 4);
                    let top = ticks.last().copied().unwrap_or(1.);
                    let kind = self.kind;
                    // Wide enough for the longest tick label.
                    let longest = ticks
                        .iter()
                        .map(|t| format(*t).chars().count())
                        .max()
                        .unwrap_or(1);
                    let axis = px(12. + 7. * longest as f32);
                    let plot_kind = match kind {
                        ChartKind::Line => PlotKind::Line,
                        ChartKind::Bar => PlotKind::Bar,
                        _ => PlotKind::Area,
                    };
                    let plot = div().relative().flex_1().h(self.plot_height).child(marks(
                        plot_kind,
                        data,
                        fills,
                        self.stacked,
                        top,
                        bytes,
                    ));
                    let y_axis = div()
                        .relative()
                        .flex_none()
                        .w(axis)
                        .h(self.plot_height)
                        .children(ticks.iter().map(|t| {
                            div()
                                .absolute()
                                .right(px(8.))
                                .top(relative(1. - t / top))
                                .mt(px(-8.))
                                .child(label(format(*t), c.text_faint))
                        }));
                    // Show at most about eight category labels.
                    let stride = self.categories.len().div_ceil(8).max(1);
                    let x_axis = h_stack()
                        .pl(axis)
                        .when(kind != ChartKind::Bar, |this| this.justify_between())
                        .children(self.categories.iter().enumerate().map(|(i, text)| {
                            let shown = i % stride == 0;
                            div()
                                .when(kind == ChartKind::Bar, |this| {
                                    this.flex_1().flex().justify_center()
                                })
                                .child(label(
                                    text.to_string(),
                                    if shown {
                                        c.text_faint
                                    } else {
                                        gpui::transparent_black()
                                    },
                                ))
                        }));
                    v_stack()
                        .gap(px(8.))
                        .child(h_stack().items_start().child(y_axis).child(plot))
                        .child(x_axis)
                        .into_any_element()
                }
                ChartKind::Radar => {
                    let top = nice_ticks(self.data_max(), 4).last().copied().unwrap_or(1.);
                    let spokes = self.categories.len().max(3);
                    let size = self.plot_height.max(px(200.));
                    v_stack()
                        .items_center()
                        .child(
                            div()
                                .relative()
                                .size(size)
                                // Rings and spokes.
                                .child(
                                    canvas(
                                        |_, _, _| (),
                                        move |bounds: Bounds<Pixels>, _, window, _| {
                                            let center = bounds.center();
                                            let r = f32::from(bounds.size.width) * 0.45;
                                            let grid = c.border;
                                            for ring in 1..=4 {
                                                let rr = r * ring as f32 / 4.;
                                                let mut path = gpui::PathBuilder::stroke(px(1.));
                                                for i in 0..=spokes {
                                                    let a = i as f32 / spokes as f32 * TAU;
                                                    let p = gpui::point(
                                                        center.x + px(rr * a.sin()),
                                                        center.y - px(rr * a.cos()),
                                                    );
                                                    if i == 0 {
                                                        path.move_to(p)
                                                    } else {
                                                        path.line_to(p)
                                                    }
                                                }
                                                if let Ok(path) = path.build() {
                                                    window.paint_path(path, grid);
                                                }
                                            }
                                            for i in 0..spokes {
                                                let a = i as f32 / spokes as f32 * TAU;
                                                let mut path = gpui::PathBuilder::stroke(px(1.));
                                                path.move_to(center);
                                                path.line_to(gpui::point(
                                                    center.x + px(r * a.sin()),
                                                    center.y - px(r * a.cos()),
                                                ));
                                                if let Ok(path) = path.build() {
                                                    window.paint_path(path, grid);
                                                }
                                            }
                                        },
                                    )
                                    .absolute()
                                    .inset_0(),
                                )
                                .child(marks(PlotKind::Radar, data, fills, false, top, bytes))
                                // Spoke labels just outside the rim.
                                .children(self.categories.iter().enumerate().map(|(i, text)| {
                                    let a = i as f32 / spokes as f32 * TAU;
                                    let (x, y) = (0.5 + 0.5 * a.sin(), 0.5 - 0.5 * a.cos());
                                    div()
                                        .absolute()
                                        .left(relative(x))
                                        .top(relative(y))
                                        .ml(px(-40.))
                                        .mt(px(-8.))
                                        .w(px(80.))
                                        .flex()
                                        .justify_center()
                                        .child(label(text.to_string(), c.text_muted))
                                })),
                        )
                        .into_any_element()
                }
                ChartKind::Donut => {
                    let values: Vec<f32> = self
                        .series
                        .first()
                        .map(|s| s.values.clone())
                        .unwrap_or_default();
                    let total: f32 = values.iter().sum();
                    let ring = div()
                        .relative()
                        .flex_none()
                        .size(px(176.))
                        .child(marks(PlotKind::Donut, data, fills, false, 1., bytes))
                        .child(
                            v_stack()
                                .absolute()
                                .inset_0()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_size(px(20.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(c.text)
                                        .child(format(total)),
                                )
                                .child(label("Total".into(), c.text_muted)),
                        );
                    let rows = self.categories.iter().zip(&values).zip(colors.iter()).map(
                        |((name, value), color)| {
                            let share = if total > 0. { value / total * 100. } else { 0. };
                            h_stack()
                                .gap(px(8.))
                                .text_size(theme.metrics.text_sm)
                                .child(swatch(*color))
                                .child(div().flex_1().text_color(c.text).child(name.clone()))
                                .child(div().text_color(c.text).child(format(*value)))
                                .child(
                                    div()
                                        .w(px(44.))
                                        .flex()
                                        .justify_end()
                                        .text_color(c.text_faint)
                                        .child(format!("{share:.0}%")),
                                )
                        },
                    );
                    h_stack()
                        .gap(px(28.))
                        .items_center()
                        .child(ring)
                        .child(v_stack().flex_1().gap(px(8.)).children(rows))
                        .into_any_element()
                }
                ChartKind::Heatmap => {
                    let top = self.data_max().max(1e-6);
                    let ink = bytes[0];
                    let rows = self.series.len().max(1);
                    let cols = self.categories.len().max(1);
                    let stride = cols.div_ceil(8).max(1);
                    let row_height = px(20.);
                    // A dithered swatch at each step, matching the cells.
                    let step = move |level: f32| {
                        canvas(
                            |_, _, _| (),
                            move |bounds: Bounds<Pixels>, _, window, _| {
                                let series = vec![vec![level * top]];
                                let plot = Plot {
                                    kind: PlotKind::Heatmap,
                                    series: &series,
                                    fills: &[],
                                    stacked: false,
                                    top,
                                    progress: 1.,
                                };
                                let n = (f32::from(bounds.size.width) / 2.) as usize;
                                if let Ok(raster) = plot.render(n.max(1), n.max(1), &[ink]) {
                                    paint_raster(window, bounds.origin, &raster, 2.);
                                }
                            },
                        )
                        .size(px(12.))
                    };
                    let scale = h_stack()
                        .gap(px(4.))
                        .justify_end()
                        .child(label("Less".into(), c.text_muted))
                        .children([0., 0.25, 0.5, 0.75, 1.].into_iter().map(step))
                        .child(label("More".into(), c.text_muted));
                    v_stack()
                        .gap(px(6.))
                        .child(
                            h_stack()
                                .items_start()
                                .child(v_stack().w(px(40.)).children(self.series.iter().map(|s| {
                                    div()
                                        .h(row_height)
                                        .flex()
                                        .items_center()
                                        .child(label(s.name.to_string(), c.text_faint))
                                })))
                                .child(
                                    div().relative().flex_1().h(row_height * rows as f32).child(
                                        marks(PlotKind::Heatmap, data, fills, false, top, bytes),
                                    ),
                                ),
                        )
                        .child(h_stack().pl(px(40.)).children(
                            self.categories.iter().enumerate().map(|(i, text)| {
                                div().flex_1().child(label(
                                    text.to_string(),
                                    if i % stride == 0 {
                                        c.text_faint
                                    } else {
                                        gpui::transparent_black()
                                    },
                                ))
                            }),
                        ))
                        .child(div().pt(px(6.)).child(scale))
                        .into_any_element()
                }
            };

        v_stack()
            .id(self.id)
            .role(Role::Image)
            .aria_label(summary)
            .gap(px(16.))
            .child(
                h_stack()
                    .items_start()
                    .justify_between()
                    .gap(px(16.))
                    .child(header)
                    .children(legend),
            )
            .child(body)
            .refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_are_round_and_cover_the_maximum() {
        assert_eq!(nice_ticks(314., 4), vec![0., 100., 200., 300., 400.]);
        assert_eq!(nice_ticks(9.2, 4), vec![0., 2.5, 5., 7.5, 10.]);
        assert_eq!(nice_ticks(0., 4), vec![0., 0.25, 0.5, 0.75, 1.]);
        assert_eq!(nice_ticks(400., 4), vec![0., 100., 200., 300., 400.]);
    }

    #[test]
    fn compact_numbers_read_naturally() {
        assert_eq!(compact(950.), "950");
        assert_eq!(compact(1200.), "1.2k");
        assert_eq!(compact(2_500_000.), "2.5M");
        assert_eq!(compact(3000.), "3k");
    }

    #[test]
    fn stacked_charts_scale_to_the_tallest_stack() {
        let chart = DitherChart::new("c", "C")
            .kind(ChartKind::Bar)
            .stacked(true)
            .series("a", [10., 20.])
            .series("b", [5., 15.]);
        assert_eq!(chart.data_max(), 35.);
        assert_eq!(chart.stacked(false).data_max(), 20.);
    }
}
