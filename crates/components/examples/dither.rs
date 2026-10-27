//! Native dither playground: cargo run -p zkit-components --example dither
use gpui::{
    App, Bounds, Context, FontWeight, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use zkit_components::{
    Accent, Appearance, Button, DitherButton, DitherSurface, Matrix, Palette, Preset, Root, Theme,
    h_stack, spacer, v_stack,
};

struct DitherLab {
    preset: Preset,
    matrix: usize,
    palette: usize,
    cell: f32,
    playing: bool,
    still_time: f32,
}
impl Render for DitherLab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palettes = [
            Palette::duo([16, 21, 18], [208, 251, 116]),
            Palette::duo([18, 16, 23], [186, 161, 248]),
            Palette::duo([24, 18, 14], [250, 166, 95]),
            Palette::duo([242, 241, 233], [37, 55, 40]),
        ];
        let matrices = [
            Matrix::Bayer2,
            Matrix::Bayer4,
            Matrix::Bayer8,
            Matrix::BlueNoise,
        ];
        let sidebar = v_stack()
            .id("presets")
            .w(px(210.))
            .flex_none()
            .h_full()
            .overflow_y_scroll()
            .p(px(16.))
            .gap(px(6.))
            .children(Preset::ALL.iter().enumerate().map(|(i, &preset)| {
                Button::new(("preset", i), preset.label())
                    .ghost()
                    .toggled(self.preset == preset)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.preset = preset;
                        cx.notify();
                    }))
            }));
        let heading = h_stack()
            .child(div().text_size(px(30.)).child(self.preset.label()))
            .child(spacer())
            .child(
                Button::new("play", if self.playing { "Pause" } else { "Play" }).on_click(
                    cx.listener(|this, _, window, cx| {
                        this.still_time = zkit_components::pulse::seconds(
                            window.current_view(),
                            zkit_components::pulse::Rate::Coarse,
                            cx,
                        );
                        this.playing = !this.playing;
                        cx.notify();
                    }),
                ),
            );
        let controls = h_stack()
            .gap(px(12.))
            .child(
                Button::new("matrix", format!("Matrix: {:?}", matrices[self.matrix])).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.matrix = (this.matrix + 1) % 4;
                        cx.notify();
                    }),
                ),
            )
            .child(
                Button::new("palette", "Change palette").on_click(cx.listener(|this, _, _, cx| {
                    this.palette = (this.palette + 1) % 4;
                    cx.notify();
                })),
            )
            .child(
                Button::new("scale", format!("Cell: {}px", self.cell)).on_click(cx.listener(
                    |this, _, _, cx| {
                        this.cell = if this.cell >= 8. { 2. } else { this.cell + 2. };
                        cx.notify();
                    },
                )),
            );
        let content = v_stack()
            .flex_1()
            .min_w_0()
            .h_full()
            .p(px(28.))
            .gap(px(20.))
            .child(heading)
            .child(
                DitherSurface::new(self.preset)
                    .matrix(matrices[self.matrix])
                    .palette(palettes[self.palette].clone())
                    .cell_size(self.cell)
                    .animated(self.playing)
                    .still_time(self.still_time)
                    .w_full()
                    .flex_1()
                    .rounded(px(8.)),
            )
            .child(controls)
            .child(
                h_stack()
                    .gap(px(12.))
                    .child(zkit_components::dither_avatar("demo-avatar", "Zkit"))
                    .child(DitherButton::new("dither-toggle", "Switch field").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.preset = if this.preset == Preset::Aurora {
                                Preset::Waves
                            } else {
                                Preset::Aurora
                            };
                            cx.notify();
                        }),
                    )),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(0x9ca994))
                    .child("35 fields · 4 matrices · shared clock · reduced motion aware"),
            );
        let header = h_stack()
            .flex_none()
            .p(px(20.))
            .gap(px(12.))
            .border_b_1()
            .border_color(rgb(0x30382e))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::BOLD)
                    .child("zkit / dither"),
            )
            .child(spacer())
            .child(
                div()
                    .text_size(px(12.))
                    .child("RUST VIEWS. EVERY PIXEL CONSIDERED."),
            );
        Root::new().child(
            v_stack()
                .size_full()
                .bg(rgb(0x101512))
                .text_color(rgb(0xe9eee5))
                .child(header)
                .child(h_stack().flex_1().min_h_0().child(sidebar).child(content)),
        )
    }
}
fn main() {
    env_logger::init();
    gpui_platform::application().run(|cx: &mut App| {
        zkit_components::init(cx);
        Theme::new(Appearance::Dark, Accent::Green).install(cx);
        let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| DitherLab {
                    preset: Preset::Aurora,
                    matrix: 2,
                    palette: 0,
                    cell: 4.,
                    playing: true,
                    still_time: 6.,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
