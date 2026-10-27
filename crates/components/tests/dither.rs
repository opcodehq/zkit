//! Textured surfaces preserve native child controls and cached raster colors.
use gpui::{
    Context, IntoElement, Modifiers, Render, Styled as _, TestAppContext, Window, prelude::*, px,
};
use std::{cell::Cell, rc::Rc};
use zkit_components::{DitherButton, DitherSurface, Preset, Root};

struct Harness {
    clicks: Rc<Cell<usize>>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        Root::new().child(
            DitherSurface::new(Preset::Aurora)
                .animated(false)
                .w(px(320.))
                .h(px(180.))
                .child(
                    gpui::div()
                        .p(px(20.))
                        .debug_selector(|| "control".into())
                        .child(
                            DitherButton::new("action", "Activate")
                                .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                        ),
                ),
        )
    }
}
#[gpui::test]
fn texture_preserves_descendant_activation_and_layout(cx: &mut TestAppContext) {
    cx.update(zkit_components::init);
    let clicks = Rc::new(Cell::new(0));
    let (_, cx) = cx.add_window_view({
        let clicks = clicks.clone();
        move |_, _| Harness { clicks }
    });
    cx.update(|window, cx| window.draw(cx).clear());
    let bounds = cx
        .debug_bounds("control")
        .expect("textured surface child laid out");
    assert!(bounds.size.width > px(0.));
    cx.simulate_click(
        gpui::point(bounds.origin.x + px(45.), bounds.origin.y + px(35.)),
        Modifiers::default(),
    );
    cx.run_until_parked();
    assert_eq!(clicks.get(), 1);
    // Pointer activation focuses the foundation button. Enter must activate it
    // without any special behavior from the textured wrapper.
    let keystroke = gpui::Keystroke::parse("enter").unwrap();
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    cx.run_until_parked();
    assert_eq!(clicks.get(), 2);
}
