#![cfg_attr(target_family = "wasm", no_main)]

//! Interactive component gallery.
//!
//! Run natively with `cargo run -p zkit-components --example gallery`.
//! Every control is live: Tab / Shift-Tab move focus, Enter or Space
//! activate, arrow keys move within the sidebar and menus, Escape closes
//! popovers and the dialog.

use gpui::{
    App, Bounds, Context, FocusHandle, FontWeight, KeyBinding, SharedString, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, size,
};
use zkit_components::{
    Accent, ActiveTheme as _, Appearance, Badge, Button, Dialog, Disclosure, Divider, Menu,
    MenuItem, MotionPreference, Panel, Popover, ProgressBar, PulseLoader, Root, Sidebar,
    SidebarItem, SidebarSection, Spinner, Theme, Toolbar, h_stack, motion, spacer, v_stack,
};

#[cfg(target_family = "wasm")]
thread_local! {
    static APPLICATION: std::cell::RefCell<Option<gpui::ApplicationHandle>> = const { std::cell::RefCell::new(None) };
}

actions!(gallery, [Quit, ToggleSidebar]);

const NAV: [(&str, &str); 4] = [
    ("Explorer", "▤"),
    ("Search", "⌕"),
    ("Source Control", "⑂"),
    ("Run and Debug", "▷"),
];

struct Gallery {
    focus: FocusHandle,
    selected: usize,
    sidebar_collapsed: bool,
    progress: f32,
    saving: bool,
    word_wrap: bool,
    discard: Disclosure,
    status: SharedString,
    entrance: usize,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            focus,
            selected: 0,
            sidebar_collapsed: false,
            progress: 40.,
            saving: false,
            word_wrap: true,
            discard: Disclosure::new(cx),
            status: "Ready".into(),
            entrance: 0,
        }
    }

    fn set_status(&mut self, status: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.status = status.into();
        cx.notify();
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let dark = theme.appearance.is_dark();
        Toolbar::new("toolbar", "Gallery actions")
            .child(
                Button::icon("toggle-sidebar", "◧", "Toggle sidebar")
                    .toggled(!self.sidebar_collapsed)
                    .shortcut("secondary-b")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_collapsed = !this.sidebar_collapsed;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .px(px(4.))
                    .text_size(theme.metrics.text_md)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Zkit Components"),
            )
            .child(spacer())
            .child(
                Popover::new(
                    "accent-menu",
                    Button::new("accent", theme.accent.label())
                        .ghost()
                        .trailing("▾"),
                )
                .menu(move |popover, _, _| {
                    Accent::ALL
                        .iter()
                        .fold(Menu::new().heading("Accent"), |menu, &accent| {
                            menu.item(
                                MenuItem::new(SharedString::from(accent.label()), accent.label())
                                    .checked(accent == theme.accent)
                                    .dismisses(popover)
                                    .on_select(move |_, _, cx| {
                                        Theme::new(cx.theme().appearance, accent).install(cx)
                                    }),
                            )
                        })
                }),
            )
            .child(
                Popover::new(
                    "motion-menu",
                    Button::new("motion", "Motion").ghost().trailing("▾"),
                )
                .menu(|popover, _, cx| {
                    let current = zkit_components::MotionPolicy::global(cx).preference;
                    MotionPreference::ALL.iter().fold(
                        Menu::new().heading("Reduce motion"),
                        |menu, &preference| {
                            menu.item(
                                MenuItem::new(
                                    SharedString::from(preference.label()),
                                    preference.label(),
                                )
                                .checked(preference == current)
                                .dismisses(popover)
                                .on_select(move |_, _, cx| motion::set_preference(preference, cx)),
                            )
                        },
                    )
                }),
            )
            .child(
                Button::new("appearance", if dark { "Light" } else { "Dark" })
                    .leading(if dark { "☼" } else { "☾" })
                    .on_click(|_, _, cx| {
                        let theme = cx.theme();
                        let next = if theme.appearance.is_dark() {
                            Appearance::Light
                        } else {
                            Appearance::Dark
                        };
                        Theme::new(next, theme.accent).install(cx);
                    }),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let items = NAV.iter().enumerate().map(|(index, (label, icon))| {
            SidebarItem::new(("nav", index), *label)
                .icon(*icon)
                .selected(self.selected == index)
                .when(index == 2, |item| item.trailing(Badge::new("3").accent()))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected = index;
                    this.set_status(format!("Opened {}", NAV[index].0), cx);
                }))
        });
        Sidebar::new("sidebar", "Workspace")
            .collapsed(self.sidebar_collapsed)
            .child(SidebarSection::new("views").title("Views").items(items))
            .child(
                SidebarSection::new("recent").title("Recent").items([
                    SidebarItem::new("recent-0", "main.rs").icon("·"),
                    SidebarItem::new("recent-1", "Cargo.toml").icon("·"),
                    SidebarItem::new("recent-2", "archived.log")
                        .icon("·")
                        .disabled(true),
                ]),
            )
            .footer(
                h_stack()
                    .gap(px(6.))
                    .child(Spinner::new().accessible("indexing", "Indexing workspace"))
                    .child(
                        div()
                            .text_size(cx.theme().metrics.text_sm)
                            .text_color(cx.theme().colors.text_muted)
                            .child("Indexing…"),
                    ),
            )
    }

    fn buttons_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let row = || h_stack().gap(px(6.)).flex_wrap();
        Panel::new("buttons")
            .title("Buttons")
            .action(Button::icon("buttons-more", "⋯", "More button options").small())
            .child(
                v_stack()
                    .p(px(12.))
                    .gap(px(10.))
                    .child(
                        row()
                            .child(
                                Button::new("primary", "Commit")
                                    .primary()
                                    .shortcut("secondary-enter")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_status("Committed", cx)
                                    })),
                            )
                            .child(
                                Button::new("accent-btn", "Create branch")
                                    .accent()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_status("Branch created", cx)
                                    })),
                            )
                            .child(Button::new("secondary", "Fetch").on_click(
                                cx.listener(|this, _, _, cx| this.set_status("Fetched", cx)),
                            ))
                            .child(Button::new("ghost", "Stash").ghost().on_click(
                                cx.listener(|this, _, _, cx| this.set_status("Stashed", cx)),
                            ))
                            .child(Button::new("danger", "Discard").danger().on_click(
                                cx.listener(|this, _, window, cx| this.discard.open(window, cx)),
                            )),
                    )
                    .child(
                        row()
                            .child(Button::new("small", "Small").small())
                            .child(Button::new("medium", "Medium"))
                            .child(Button::new("large", "Large").large())
                            .child(Button::icon("icon-add", "+", "Add file"))
                            .child(Button::icon("icon-refresh", "↻", "Refresh").secondary()),
                    )
                    .child(
                        row()
                            .child(Button::new("disabled", "Disabled").disabled(true))
                            .child(
                                Button::new("save", if self.saving { "Saving" } else { "Save" })
                                    .primary()
                                    .loading(self.saving)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.saving = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("stop", "Stop")
                                    .ghost()
                                    .disabled(!self.saving)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.saving = false;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("wrap", "Word wrap")
                                    .ghost()
                                    .toggled(self.word_wrap)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.word_wrap = !this.word_wrap;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
    }

    fn progress_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let caption = |text: &'static str| {
            div()
                .text_size(theme.metrics.text_sm)
                .text_color(theme.colors.text_muted)
                .child(text)
        };
        Panel::new("progress").title("Progress").child(
            v_stack()
                .p(px(12.))
                .gap(px(10.))
                .child(caption("Determinate"))
                .child(
                    h_stack()
                        .gap(px(8.))
                        .child(
                            div().flex_1().child(
                                ProgressBar::new("build")
                                    .value(self.progress)
                                    .label("Build progress"),
                            ),
                        )
                        .child(
                            div()
                                .w(px(36.))
                                .text_size(theme.metrics.text_sm)
                                .text_color(theme.colors.text_muted)
                                .child(format!("{:.0}%", self.progress)),
                        )
                        .child(
                            Button::icon("less", "−", "Decrease progress")
                                .small()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.progress = (this.progress - 10.).max(0.);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::icon("more", "+", "Increase progress")
                                .small()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.progress = (this.progress + 10.).min(100.);
                                    cx.notify();
                                })),
                        ),
                )
                .child(caption("Indeterminate"))
                .child(ProgressBar::new("sync").label("Syncing"))
                .child(caption("Activity"))
                .child(
                    h_stack()
                        .gap(px(16.))
                        .child(Spinner::new())
                        .child(Spinner::new().size(px(18.)))
                        .child(PulseLoader::new()),
                ),
        )
    }

    fn surfaces_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let entrance = self.entrance;
        Panel::new("surfaces").title("Surfaces and motion").child(
            v_stack()
                .p(px(12.))
                .gap(px(10.))
                .child(
                    h_stack()
                        .gap(px(6.))
                        .child(
                            Popover::new(
                                "details",
                                Button::new("details-trigger", "Details").trailing("▾"),
                            )
                            .width(px(260.))
                            .label("Branch details")
                            .content(move |popover, _, _| {
                                v_stack()
                                    .p(px(8.))
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("feature/reusable-components"),
                                    )
                                    .child(
                                        div()
                                            .text_size(theme.metrics.text_sm)
                                            .text_color(theme.colors.text_muted)
                                            .child("3 commits ahead, 1 behind main."),
                                    )
                                    .child(Divider::horizontal())
                                    .child(
                                        h_stack().gap(px(6.)).justify_end().child(
                                            Button::new("details-close", "Done")
                                                .small()
                                                .primary()
                                                .on_click({
                                                    let popover = popover.clone();
                                                    move |_, window, cx| popover.dismiss(window, cx)
                                                }),
                                        ),
                                    )
                            }),
                        )
                        .child(
                            Popover::new(
                                "file-menu",
                                Button::new("file-trigger", "File").trailing("▾"),
                            )
                            .menu(|popover, _, _| {
                                Menu::new()
                                    .item(
                                        MenuItem::new("new", "New file")
                                            .shortcut("secondary-n")
                                            .dismisses(popover),
                                    )
                                    .item(
                                        MenuItem::new("open", "Open…")
                                            .shortcut("secondary-o")
                                            .dismisses(popover),
                                    )
                                    .item(MenuItem::new("revert", "Revert").disabled(true))
                                    .separator()
                                    .item(
                                        MenuItem::new("delete", "Delete")
                                            .danger()
                                            .dismisses(popover),
                                    )
                            }),
                        )
                        .child(Button::new("dialog-trigger", "Open dialog").on_click(
                            cx.listener(|this, _, window, cx| this.discard.open(window, cx)),
                        ))
                        .child(Button::new("replay", "Replay entrance").ghost().on_click(
                            cx.listener(|this, _, _, cx| {
                                this.entrance += 1;
                                cx.notify();
                            }),
                        )),
                )
                .child(motion::fade_in(
                    ("entrance", entrance),
                    div()
                        .p(px(10.))
                        .rounded(cx.theme().metrics.radius_md)
                        .bg(cx.theme().colors.surface)
                        .border_1()
                        .border_color(cx.theme().colors.border)
                        .text_size(cx.theme().metrics.text_sm)
                        .text_color(cx.theme().colors.text_muted)
                        .child(if motion::reduced(cx) {
                            "Motion is reduced: entrances snap to their final state."
                        } else {
                            "Entrances fade and rise over 320ms; menus and dialogs pop in."
                        }),
                )),
        )
    }
}

impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        Root::new()
            .child(
                v_stack()
                    .id("gallery")
                    .track_focus(&self.focus)
                    .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                        this.sidebar_collapsed = !this.sidebar_collapsed;
                        cx.notify();
                    }))
                    .size_full()
                    .child(self.toolbar(cx))
                    .child(
                        h_stack()
                            .flex_1()
                            .min_h_0()
                            .items_stretch()
                            .child(self.sidebar(cx))
                            .child(
                                div()
                                    .id("content")
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_y_scroll()
                                    .p(px(12.))
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(self.buttons_panel(cx))
                                    .child(
                                        h_stack()
                                            .gap(px(12.))
                                            .items_start()
                                            .child(div().flex_1().child(self.progress_panel(cx)))
                                            .child(div().flex_1().child(self.surfaces_panel(cx))),
                                    ),
                            ),
                    )
                    .child(
                        h_stack()
                            .flex_none()
                            .h(px(22.))
                            .px(px(10.))
                            .bg(theme.colors.surface)
                            .border_t_1()
                            .border_color(theme.colors.border)
                            .text_size(theme.metrics.text_xs)
                            .text_color(theme.colors.text_muted)
                            .child(self.status.clone())
                            .child(spacer())
                            .child(format!(
                                "{} · {}",
                                NAV[self.selected].0,
                                theme.accent.label()
                            )),
                    ),
            )
            .child(
                Dialog::new("discard", &self.discard)
                    .title("Discard changes?")
                    .description(
                        "Uncommitted edits in 4 files will be lost. This cannot be undone.",
                    )
                    .action(
                        Button::new("discard-cancel", "Cancel").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.discard.close(window, cx);
                                this.set_status("Kept changes", cx);
                            },
                        )),
                    )
                    .action(Button::new("discard-confirm", "Discard").danger().on_click(
                        cx.listener(|this, _, window, cx| {
                            this.discard.close(window, cx);
                            this.set_status("Discarded changes", cx);
                        }),
                    ))
                    // Escape, the close button and the backdrop all keep changes.
                    .on_dismiss({
                        let view = cx.entity().downgrade();
                        move |reason, window, cx| {
                            view.update(cx, |this, cx| {
                                this.discard.close(window, cx);
                                this.set_status(format!("Dialog dismissed ({reason:?})"), cx);
                            })
                            .ok();
                        }
                    }),
            )
    }
}

fn run_example() {
    let _application = gpui_platform::application().run_embedded(|cx: &mut App| {
        zkit_components::init(cx);
        Theme::dark(Accent::Indigo).install(cx);
        cx.bind_keys([
            KeyBinding::new("secondary-q", Quit, None),
            KeyBinding::new("secondary-b", ToggleSidebar, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(1080.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Zkit Components".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Gallery::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
    #[cfg(target_family = "wasm")]
    APPLICATION.with(|owner| *owner.borrow_mut() = Some(_application));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    env_logger::init();
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
