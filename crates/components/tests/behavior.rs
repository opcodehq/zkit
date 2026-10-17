//! Keyboard and focus behavior of the composed components.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Context, FocusHandle, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers,
    ParentElement as _, Render, Styled as _, TestAppContext, VisualTestContext, Window, div,
    prelude::*, px,
};
use zkit_components::{
    Button, Dialog, Disclosure, Menu, MenuItem, Popover, Root, SidebarItem, SidebarSection,
};

type Log = Rc<RefCell<Vec<String>>>;

fn labeled(selector: &'static str, element: impl IntoElement) -> impl IntoElement {
    div()
        .debug_selector(move || selector.to_string())
        .child(element)
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} not rendered"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();
}

/// Press and release each key. Enter/Space activation fires on key-up.
fn press(cx: &mut VisualTestContext, keys: &str) {
    for key in keys.split(' ') {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        redraw(cx);
    }
}

fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear());
    cx.run_until_parked();
}

fn open_window<V: Render + 'static>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V + 'static,
) -> &mut VisualTestContext {
    cx.update(zkit_components::init);
    let (_, cx) = cx.add_window_view(build);
    redraw(cx);
    cx
}

// ---------------------------------------------------------------------------

struct MenuHarness {
    log: Log,
}

impl Render for MenuHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        Root::new().child(div().p(px(20.)).child(labeled(
            "trigger",
            Popover::new("menu", Button::new("open-menu", "File")).menu(move |popover, _, _| {
                let (first, second) = (log.clone(), log);
                Menu::new()
                    .item(
                        MenuItem::new("first", "First")
                            .dismisses(popover)
                            .on_select(move |_, _, _| first.borrow_mut().push("first".into())),
                    )
                    .item(
                        MenuItem::new("second", "Second")
                            .dismisses(popover)
                            .on_select(move |_, _, _| second.borrow_mut().push("second".into())),
                    )
            }),
        )))
    }
}

#[gpui::test]
fn menu_opens_from_the_keyboard_and_arrows_select_items(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_window(cx, {
        let log = log.clone();
        move |_, _| MenuHarness { log }
    });

    // Pointer open, then Escape closes and returns focus to the trigger.
    click(cx, "trigger");
    redraw(cx);
    let focused_in_menu = cx.update(|window, cx| window.focused(cx));
    assert!(
        focused_in_menu.is_some(),
        "opening moves focus into the menu"
    );
    press(cx, "escape");
    let trigger_focus = cx.update(|window, cx| window.focused(cx));
    assert_ne!(
        trigger_focus, focused_in_menu,
        "focus returned to the trigger"
    );

    // Keyboard: Enter on the focused trigger opens; Down twice reaches the
    // second item; Enter activates it and closes the menu.
    press(cx, "enter");
    press(cx, "down down enter");
    assert_eq!(&*log.borrow(), &["second".to_string()]);
    assert_eq!(cx.update(|window, cx| window.focused(cx)), trigger_focus);

    // Down past the last item stays on it rather than leaving the menu.
    press(cx, "enter");
    press(cx, "down down down enter");
    assert_eq!(
        &*log.borrow(),
        &["second".to_string(), "second".to_string()]
    );
}

// ---------------------------------------------------------------------------

struct DialogHarness {
    dialog: Disclosure,
    log: Log,
}

impl Render for DialogHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        let dismiss_log = self.log.clone();
        Root::new()
            .child(
                div()
                    .p(px(20.))
                    .flex()
                    .gap(px(8.))
                    .child(labeled(
                        "open",
                        Button::new("open", "Open").on_click(cx.listener(|this, _, window, cx| {
                            this.dialog.open(window, cx);
                            cx.notify();
                        })),
                    ))
                    .child(Button::new("outside", "Outside").on_click({
                        let log = log.clone();
                        move |_, _, _| log.borrow_mut().push("outside".into())
                    })),
            )
            .child(
                Dialog::new("dialog", &self.dialog)
                    .title("Confirm")
                    .close_button(false)
                    .action(Button::new("cancel", "Cancel").on_click({
                        let log = log.clone();
                        move |_, _, _| log.borrow_mut().push("cancel".into())
                    }))
                    .action(
                        Button::new("ok", "OK")
                            .on_click(move |_, _, _| log.borrow_mut().push("ok".into())),
                    )
                    .on_dismiss(move |reason, _, _| {
                        dismiss_log.borrow_mut().push(format!("dismiss:{reason:?}"))
                    }),
            )
    }
}

#[gpui::test]
fn dialog_traps_tab_activates_buttons_on_enter_and_reports_escape(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_window(cx, {
        let log = log.clone();
        move |_, cx| DialogHarness {
            dialog: Disclosure::new(cx),
            log,
        }
    });

    click(cx, "open");
    redraw(cx);

    // Tab cycles Cancel -> OK -> Cancel ... and never reaches the page.
    for _ in 0..5 {
        press(cx, "tab");
    }
    // Five tabs from the dialog container land on Cancel (odd count).
    press(cx, "enter");
    assert_eq!(&*log.borrow(), &["cancel".to_string()]);

    press(cx, "tab enter");
    assert_eq!(&*log.borrow(), &["cancel".to_string(), "ok".to_string()]);

    press(cx, "escape");
    assert_eq!(
        log.borrow().last().map(String::as_str),
        Some("dismiss:Escape")
    );
    assert!(!log.borrow().iter().any(|entry| entry == "outside"));
}

// ---------------------------------------------------------------------------

struct SidebarHarness {
    focus: FocusHandle,
    log: Log,
}

impl Render for SidebarHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let items = ["One", "Two", "Three"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                let log = self.log.clone();
                SidebarItem::new(("item", index), label)
                    .on_click(move |_, _, _| log.borrow_mut().push(label.to_string()))
            });
        Root::new().child(
            div()
                .track_focus(&self.focus)
                .w(px(200.))
                .child(SidebarSection::new("section").title("Items").items(items))
                .child(Button::new("after", "After")),
        )
    }
}

#[gpui::test]
fn sidebar_arrows_move_between_items_without_leaving_the_list(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_window(cx, {
        let log = log.clone();
        move |window, cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            SidebarHarness { focus, log }
        }
    });

    // Tab into the first item, then use arrows.
    press(cx, "tab");
    press(cx, "enter");
    press(cx, "down enter");
    press(cx, "down down down enter");
    press(cx, "home enter");
    assert_eq!(
        &*log.borrow(),
        &["One", "Two", "Three", "One"].map(String::from)
    );
}

// ---------------------------------------------------------------------------

struct LoadingHarness {
    log: Log,
    loading: bool,
}

impl Render for LoadingHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        Root::new().child(labeled(
            "save",
            Button::new("save", "Save")
                .loading(self.loading)
                .on_click(move |_, _, _| log.borrow_mut().push("save".into())),
        ))
    }
}

#[gpui::test]
fn loading_button_ignores_activation_until_work_finishes(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_window(cx, {
        let log = log.clone();
        move |_, _| LoadingHarness { log, loading: true }
    });

    click(cx, "save");
    press(cx, "tab enter");
    assert!(log.borrow().is_empty());

    cx.update(|window, cx| {
        let view = window.root::<LoadingHarness>().flatten().unwrap();
        view.update(cx, |this, cx| {
            this.loading = false;
            cx.notify();
        });
    });
    redraw(cx);
    click(cx, "save");
    assert_eq!(&*log.borrow(), &["save".to_string()]);
}
