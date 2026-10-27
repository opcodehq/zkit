#![cfg_attr(target_family = "wasm", no_main)]
//! Native material playground: `cargo run -p zkit-components --example glass`.
use gpui::{
    App, Bounds, Context, FontWeight, Window, WindowBounds, WindowOptions, div, linear_color_stop,
    linear_gradient, prelude::*, px, rgb, size,
};
use zkit_components::{
    Accent, ActiveTheme, Appearance, Button, GlassSurface, Root, Theme, h_stack, spacer, v_stack,
};

#[cfg(target_family = "wasm")]
thread_local! {
    static APPLICATION: std::cell::RefCell<Option<gpui::ApplicationHandle>> = const { std::cell::RefCell::new(None) };
}

struct GlassLab {
    solid: bool,
    light: bool,
    inspector: bool,
    selected: usize,
}
impl Render for GlassLab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = cx.theme().colors;
        let solid = self.solid;
        let muted = c.text_muted;
        let label = |text: &'static str| {
            div()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(muted)
                .child(text)
        };
        let surface = || GlassSurface::new().opaque(solid);
        Root::new().child(
            div().size_full().relative().overflow_hidden()
                .bg(linear_gradient(135., linear_color_stop(rgb(0x142b40), 0.), linear_color_stop(rgb(0x876b93), 1.)))
                // Abstract landscape is painted in GPUI beneath the system material.
                .child(div().absolute().left(px(-140.)).top(px(250.)).w(px(1000.)).h(px(700.)).rounded(px(350.))
                    .bg(linear_gradient(25., linear_color_stop(rgb(0x317f81), 0.), linear_color_stop(rgb(0x163c52), 1.))))
                .child(div().absolute().right(px(-180.)).top(px(330.)).w(px(1000.)).h(px(650.)).rounded(px(320.))
                    .bg(linear_gradient(160., linear_color_stop(rgb(0xcf927e), 0.), linear_color_stop(rgb(0x725678), 1.))))
                .child(div().absolute().left(px(46.)).top(px(38.)).right(px(46.)).child(
                    surface().h(px(64.)).child(h_stack().size_full().px(px(20.)).gap(px(12.))
                        .child(div().text_size(px(22.)).child("◈"))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Zkit Studio"))
                        .child(div().text_size(px(12.)).text_color(muted).child("/  Material studies"))
                        .child(spacer())
                        .child(Button::new("solid", if solid { "Use glass" } else { "Reduce transparency" }).ghost()
                            .on_click(cx.listener(|this, _, _, cx| { this.solid = !this.solid; cx.notify(); })))
                        .child(Button::new("theme", if self.light { "Dark" } else { "Light" }).ghost()
                            .on_click(cx.listener(|this, _, _, cx| { this.light = !this.light; Theme::new(if this.light {Appearance::Light} else {Appearance::Dark}, Accent::Indigo).install(cx); cx.notify(); })))
                        .child(Button::new("inspector", "Inspector").toggled(self.inspector)
                            .on_click(cx.listener(|this, _, _, cx| {this.inspector = !this.inspector; cx.notify();}))))))
                .child(div().absolute().left(px(46.)).top(px(124.)).w(px(205.)).bottom(px(42.)).child(
                    surface().sidebar().size_full().child(v_stack().p(px(18.)).gap(px(12.)).size_full()
                        .child(label("WORKSPACE"))
                        .children([("◫", "Overview"), ("◇", "Materials"), ("▤", "Components"), ("↗", "Activity")].into_iter().enumerate().map(|(i,(icon,title))|
                            Button::new(("nav",i),title).leading(icon).ghost().toggled(self.selected==i)
                                .on_click(cx.listener(move |this,_,_,cx| {this.selected=i;cx.notify();}))))
                        .child(spacer())
                        .child(label("ONE VIEW. EVERY PLATFORM."))
                        .child(div().text_size(px(12.)).text_color(muted).child("Rust views, system materials, familiar interactions.")))))
                .child(div().absolute().left(px(290.)).top(px(164.)).right(px(64.)).child(
                    v_stack().gap(px(16.)).text_color(rgb(0xffffff))
                        .child(div().text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).child("NATIVE MATERIAL EXPLORER"))
                        .child(div().text_size(px(48.)).font_weight(FontWeight::SEMIBOLD).child(["A little more clarity.", "Let the content breathe.", "Familiar by design.", "Everything in motion."][self.selected]))
                        .child(div().max_w(px(460.)).text_size(px(16.)).child("System surfaces meet GPU-rendered content. A quieter workspace, with depth where it matters."))))
                .child(div().absolute().left(px(290.)).bottom(px(80.)).w(px(350.)).h(px(224.)).child(
                    surface().size_full().child(v_stack().p(px(24.)).gap(px(14.))
                        .child(label("MATERIAL / 01"))
                        .child(div().text_size(px(26.)).font_weight(FontWeight::SEMIBOLD).child("A lighter touch"))
                        .child(div().text_size(px(14.)).text_color(muted).child("Native glass behind real GPUI controls. Focus and interaction stay in Rust."))
                        .child(h_stack().gap(px(8.)).child(Button::new("explore", "Explore materials").primary()
                            .on_click(cx.listener(|this,_,_,cx| {this.inspector=true;this.selected=1;cx.notify();})))
                            .child(Button::new("reset", "Reset").ghost().on_click(cx.listener(|this,_,_,cx| {this.solid=false;this.selected=0;cx.notify();})))))))
                .when(self.inspector, |view| view.child(
                    div().absolute().right(px(46.)).bottom(px(80.)).w(px(280.)).h(px(224.)).child(
                        surface().size_full().child(v_stack().p(px(24.)).gap(px(14.))
                            .child(h_stack().child(label("SURFACE INSPECTOR")).child(spacer())
                                .child(Button::icon("close", "×", "Close inspector").small().ghost()
                                    .on_click(cx.listener(|this,_,_,cx| {this.inspector=false;cx.notify();}))))
                            .child(div().text_size(px(20.)).font_weight(FontWeight::SEMIBOLD).child(if solid {"Solid surface"} else {"Adaptive glass"}))
                            .child(div().text_size(px(13.)).text_color(muted).child("Rounded geometry\nSystem appearance\nGPUI buttons and keyboard focus"))
                            .child(div().text_size(px(11.)).text_color(muted).child(if cfg!(target_os="macos") {"AppKit material bridge"} else {"Portable GPU blur fallback"}))))))
                .child(div().absolute().left(px(290.)).bottom(px(42.)).text_size(px(11.)).text_color(rgb(0xffffff)).child("ZKIT  /  GLASS STUDY 001"))
        )
    }
}
fn run() {
    let app = gpui_platform::application().run_embedded(|cx: &mut App| {
        zkit_components::init(cx);
        #[cfg(not(target_family = "wasm"))]
        let light = std::env::args().any(|arg| arg == "--light");
        #[cfg(not(target_family = "wasm"))]
        let solid = std::env::args().any(|arg| arg == "--solid");
        #[cfg(target_family = "wasm")]
        let (light, solid) = (false, false);
        Theme::new(
            if light {
                Appearance::Light
            } else {
                Appearance::Dark
            },
            Accent::Indigo,
        )
        .install(cx);
        let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Zkit — Native Materials".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                #[cfg(target_os = "macos")]
                window
                    .enable_scene_overlay()
                    .expect("native material overlay");
                let _ = window;
                cx.new(|_| GlassLab {
                    solid,
                    light,
                    inspector: true,
                    selected: 0,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
    #[cfg(target_family = "wasm")]
    APPLICATION.with(|owner| *owner.borrow_mut() = Some(app));
    #[cfg(not(target_family = "wasm"))]
    let _ = app;
}
#[cfg(not(target_family = "wasm"))]
fn main() {
    env_logger::init();
    run();
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run();
}
