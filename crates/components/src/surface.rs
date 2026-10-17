//! Floating surfaces: popovers, menus and modal dialogs.
//!
//! Behavior comes from the foundation layer: [`gpui_base::PopoverState`]
//! owns open state, Escape dismissal and focus capture/restore;
//! [`gpui_base::Positioner`] places popovers beside their trigger, flipping
//! and clamping to the window; [`gpui_base::FocusTrapElement`] keeps Tab
//! inside a dialog. This module adds presentation, entrance motion, keyboard
//! activation of triggers and menu semantics.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, ElementId, Entity, EntityId, FontWeight,
    InteractiveElement as _, IntoElement, KeyBinding, MouseButton, ParentElement, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Toggled, Window, deferred, div, prelude::FluentBuilder as _, px,
};
use gpui_base::actions::{Cancel, SelectDown, SelectFirst, SelectLast, SelectUp};
use gpui_base::{
    Align, ElementExt as _, FocusTrapElement as _, Placement, PopoverState, Positioner,
    StyledExt as _,
};
use smallvec::SmallVec;

use crate::button::{Button, Kbd};
use crate::focus::Disclosure;
use crate::layout::{jump_within, step_within};
use crate::motion;
use crate::theme::{ActiveTheme as _, Theme};

const DIALOG_CONTEXT: &str = "ZkitDialog";
/// Matches the foundation popover's context, whose Escape binding we reuse.
const POPOVER_CONTEXT: &str = "Popover";
/// Menus also take list navigation bindings.
const MENU_CONTEXT: &str = "Popover ZkitList";
const DIALOG_PRIORITY: usize = 10;

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", Cancel, Some(DIALOG_CONTEXT))]);
}

/// A floating card: overlay plate, hairline border, shadow and inset.
pub fn card(theme: &Theme) -> Div {
    let c = &theme.colors;
    let m = &theme.metrics;
    div()
        .flex()
        .flex_col()
        .p(m.card_inset)
        .gap(px(1.))
        .bg(c.surface_overlay)
        .border_1()
        .border_color(c.border_strong)
        .rounded(m.radius_card)
        .shadow(theme.overlay_shadow())
        .overflow_hidden()
        .text_size(m.text_md)
        .text_color(c.text)
}

// ---------------------------------------------------------------------------
// Popover
// ---------------------------------------------------------------------------

/// Handle passed to popover content so it can close its surface.
#[derive(Clone)]
pub struct PopoverHandle {
    state: Entity<PopoverState>,
    parent: EntityId,
}

impl PopoverHandle {
    pub fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| state.dismiss(window, cx));
        cx.notify(self.parent);
    }
}

type ContentBuilder = Box<dyn FnOnce(&PopoverHandle, &mut Window, &mut App) -> AnyElement>;

/// What the popover surface is, for assistive technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverKind {
    /// Arbitrary content (forms, details): a non-modal dialog.
    #[default]
    Dialog,
    /// A list of commands: arrow keys move between [`MenuItem`]s.
    Menu,
}

/// A trigger button with an anchored floating surface.
///
/// Opens on pointer press or Enter/Space on the focused trigger; focus moves
/// into the surface and returns to the trigger when it closes. Escape or a
/// press outside dismisses.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    trigger: Button,
    content: Option<ContentBuilder>,
    kind: PopoverKind,
    placement: Placement,
    align: Align,
    width: Option<Pixels>,
    label: Option<SharedString>,
}

impl Popover {
    pub fn new(id: impl Into<ElementId>, trigger: Button) -> Self {
        Self {
            id: id.into(),
            trigger,
            content: None,
            kind: PopoverKind::default(),
            placement: Placement::Bottom,
            align: Align::Start,
            width: None,
            label: None,
        }
    }

    pub fn content<E: IntoElement>(
        mut self,
        build: impl FnOnce(&PopoverHandle, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |handle, window, cx| {
            build(handle, window, cx).into_any_element()
        }));
        self
    }

    /// Render a [`Menu`]: sets menu semantics and arrow-key navigation.
    pub fn menu(
        mut self,
        build: impl FnOnce(&PopoverHandle, &mut Window, &mut App) -> Menu + 'static,
    ) -> Self {
        self.kind = PopoverKind::Menu;
        self.content(build)
    }

    /// Preferred side; flips when the window lacks room.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Accessible name of the surface (defaults to none; menus are named by
    /// their trigger).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Default)]
struct TriggerBounds(Option<Bounds<Pixels>>);

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state(self.id.clone(), cx, |_, cx| PopoverState::new(false, cx));
        let bounds = window.use_keyed_state((self.id.clone(), "trigger"), cx, |_, _| {
            TriggerBounds::default()
        });
        let parent = window.current_view();
        let handle = PopoverHandle {
            state: state.clone(),
            parent,
        };
        let open = state.read(cx).is_open();
        let focus = gpui::Focusable::focus_handle(state.read(cx), cx);

        let toggle = {
            move |window: &mut Window, cx: &mut App| {
                state.update(cx, |state, cx| {
                    // Resync to the rendered state first: an outside-press
                    // dismissal on this same press must not reopen it.
                    state.set_open(open, cx);
                    state.toggle_open(window, cx);
                });
                cx.notify(parent);
            }
        };
        let pointer_toggle = toggle.clone();
        let trigger =
            self.trigger
                .selected(open)
                .expanded(open)
                .also_on_click(move |event, window, cx| {
                    // Pointer presses toggle on mouse-down below; this covers keys.
                    if event.is_keyboard() {
                        toggle(window, cx);
                    }
                });

        let root = div()
            .id(self.id)
            .flex_none()
            .child(trigger)
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                pointer_toggle(window, cx)
            })
            .on_prepaint({
                let bounds = bounds.clone();
                move |rect, window, cx| {
                    let first = bounds.update(cx, |b, _| b.0.replace(rect).is_none());
                    if first {
                        window.request_animation_frame();
                    }
                }
            });

        let trigger_bounds = bounds.read(cx).0;
        let (true, Some(trigger_bounds), Some(content)) = (open, trigger_bounds, self.content)
        else {
            return root;
        };

        let theme = cx.theme();
        let travel = if self.placement == Placement::Top {
            4.
        } else {
            -4.
        };
        let kind = self.kind;
        let surface = card(theme)
            .id("popover-surface")
            .role(match kind {
                PopoverKind::Dialog => Role::Dialog,
                PopoverKind::Menu => Role::Menu,
            })
            .when_some(self.label, |this, label| this.aria_label(label))
            .occlude()
            .tab_group()
            .track_focus(&focus)
            // One context string: a second `key_context` call would replace it.
            .key_context(match kind {
                PopoverKind::Dialog => POPOVER_CONTEXT,
                PopoverKind::Menu => MENU_CONTEXT,
            })
            .when(kind == PopoverKind::Menu, |this| {
                let (next, previous, first, last) =
                    (focus.clone(), focus.clone(), focus.clone(), focus.clone());
                this.on_action(move |_: &SelectDown, window, cx| {
                    step_within(&next, true, window, cx)
                })
                .on_action(move |_: &SelectUp, window, cx| {
                    step_within(&previous, false, window, cx)
                })
                .on_action(move |_: &SelectFirst, window, cx| {
                    jump_within(&first, false, window, cx)
                })
                .on_action(move |_: &SelectLast, window, cx| jump_within(&last, true, window, cx))
            })
            .on_action({
                let handle = handle.clone();
                move |_: &Cancel, window, cx| handle.dismiss(window, cx)
            })
            .on_mouse_down_out({
                let handle = handle.clone();
                move |_, window, cx| handle.dismiss(window, cx)
            })
            .when_some(self.width, |this, width| this.w(width))
            .when(self.width.is_none(), |this| this.min_w(px(160.)))
            .child(content(&handle, window, cx));

        root.child(
            deferred(
                Positioner::side(trigger_bounds)
                    .placement(self.placement)
                    .align(self.align)
                    .offset(px(4.))
                    .margin(px(8.))
                    .child(motion::menu_in("popover-in", travel, surface)),
            )
            .with_priority(gpui_base::POPUP_PRIORITY),
        )
    }
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

/// Rows for a menu popover: items, headings and separators.
#[derive(IntoElement, Default)]
pub struct Menu {
    rows: SmallVec<[MenuRow; 8]>,
}

enum MenuRow {
    Item(MenuItem),
    Heading(SharedString),
    Separator,
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.rows.push(MenuRow::Item(item));
        self
    }

    pub fn heading(mut self, label: impl Into<SharedString>) -> Self {
        self.rows.push(MenuRow::Heading(label.into()));
        self
    }

    pub fn separator(mut self) -> Self {
        self.rows.push(MenuRow::Separator);
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let count = self
            .rows
            .iter()
            .filter(|row| matches!(row, MenuRow::Item(_)))
            .count();
        let mut index = 0;
        let rows: Vec<AnyElement> = self
            .rows
            .into_iter()
            .map(|row| match row {
                MenuRow::Item(item) => {
                    index += 1;
                    item.position(index, count).into_any_element()
                }
                MenuRow::Heading(label) => div()
                    .px(px(8.))
                    .pt(px(6.))
                    .pb(px(2.))
                    .text_size(theme.metrics.text_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.colors.text_faint)
                    .child(label)
                    .into_any_element(),
                // Full-bleed: cancel the card inset so the rule spans the card.
                MenuRow::Separator => div()
                    .mx(-theme.metrics.card_inset)
                    .my(px(3.))
                    .h(px(1.))
                    .bg(theme.colors.border)
                    .into_any_element(),
            })
            .collect();
        div().flex().flex_col().gap(px(1.)).children(rows)
    }
}

type SelectHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One menu command with optional icon, shortcut hint and check state.
#[derive(IntoElement)]
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    icon: Option<AnyElement>,
    shortcut: Option<SharedString>,
    checked: Option<bool>,
    danger: bool,
    disabled: bool,
    position: Option<(usize, usize)>,
    dismiss: Option<PopoverHandle>,
    on_select: Option<SelectHandler>,
}

impl MenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            checked: None,
            danger: false,
            disabled: false,
            position: None,
            dismiss: None,
            on_select: None,
        }
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Keyboard hint in gpui keystroke notation (display only).
    pub fn shortcut(mut self, keystroke: impl Into<SharedString>) -> Self {
        self.shortcut = Some(keystroke.into());
        self
    }

    /// Make this a checkable item showing a check mark when on.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Close the given popover after selection.
    pub fn dismisses(mut self, handle: &PopoverHandle) -> Self {
        self.dismiss = Some(handle.clone());
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    fn position(mut self, index: usize, count: usize) -> Self {
        self.position = Some((index, count));
        self
    }
}

impl RenderOnce for MenuItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let c = &theme.colors;
        let (hover, active) = (c.hover, c.active);
        let disabled = self.disabled;
        let text = if self.danger { c.danger } else { c.text };
        let dismiss = self.dismiss;
        let on_select = self.on_select;

        gpui_base::Button::new(self.id)
            .role(if self.checked.is_some() {
                Role::MenuItemCheckBox
            } else {
                Role::MenuItem
            })
            .accessibility_label(self.label.clone())
            .when_some(self.checked, |this, on| {
                this.aria_toggled(if on { Toggled::True } else { Toggled::False })
            })
            .when_some(self.shortcut.clone(), |this, keys| {
                this.aria_keyshortcuts(keys)
            })
            .when_some(self.position, |this, (index, count)| {
                this.aria_position_in_set(index).aria_size_of_set(count)
            })
            .disabled(disabled)
            .when(disabled, |this| {
                this.a11y_synthetic_children(|tree| tree.parent_node().set_disabled())
                    .opacity(0.45)
            })
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(26.))
            .px(px(8.))
            .rounded(theme.metrics.card_row_radius())
            .text_color(text)
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(hover))
                    .active(move |style| style.bg(active))
            })
            // Menus move focus row by row, so focus is the highlight itself.
            .focus(move |style| style.bg(hover))
            .when(!disabled, |this| {
                this.on_click(move |event, window, cx| {
                    if let Some(handler) = &on_select {
                        handler(event, window, cx);
                    }
                    if let Some(dismiss) = &dismiss {
                        dismiss.dismiss(window, cx);
                    }
                })
            })
            .when(self.checked.is_some() || self.icon.is_some(), |this| {
                let mark = match self.checked {
                    Some(true) => Some(div().child("✓").into_any_element()),
                    Some(false) => None,
                    None => self.icon,
                };
                this.child(
                    div()
                        .flex_none()
                        .w(px(14.))
                        .flex()
                        .justify_center()
                        .text_color(if self.danger { c.danger } else { c.text_muted })
                        .children(mark),
                )
            })
            .child(div().flex_1().min_w_0().truncate().child(self.label))
            .children(self.shortcut.map(Kbd::new))
    }
}

// ---------------------------------------------------------------------------
// Dialog
// ---------------------------------------------------------------------------

/// Why a dialog asked to close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismissReason {
    Escape,
    Backdrop,
    CloseButton,
}

type DismissHandler = Rc<dyn Fn(DismissReason, &mut Window, &mut App)>;

/// A modal dialog over a scrim, driven by a view-owned [`Disclosure`].
///
/// Focus moves into the dialog on [`Disclosure::open`], Tab cycles inside
/// it (with [`crate::Root`] installed), and Escape, the close button or a
/// backdrop press call [`on_dismiss`](Self::on_dismiss); the view decides
/// whether to [`Disclosure::close`]. Enter is deliberately not bound, so a
/// focused button inside the dialog activates normally.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    open: bool,
    focus: gpui::FocusHandle,
    title: Option<SharedString>,
    description: Option<SharedString>,
    width: Pixels,
    close_button: bool,
    close_on_backdrop: bool,
    footer: SmallVec<[AnyElement; 3]>,
    on_dismiss: Option<DismissHandler>,
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 2]>,
}

impl Dialog {
    pub fn new(id: impl Into<ElementId>, disclosure: &Disclosure) -> Self {
        Self {
            id: id.into(),
            open: disclosure.is_open(),
            focus: disclosure.focus_handle().clone(),
            title: None,
            description: None,
            width: px(420.),
            close_button: true,
            close_on_backdrop: true,
            footer: SmallVec::new(),
            on_dismiss: None,
            style: StyleRefinement::default(),
            children: SmallVec::new(),
        }
    }

    /// Title; also the dialog's accessible name.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Supporting text; also the dialog's accessible description.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// Footer action, laid out right-aligned in insertion order.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.footer.push(action.into_any_element());
        self
    }

    pub fn close_button(mut self, show: bool) -> Self {
        self.close_button = show;
        self
    }

    pub fn close_on_backdrop(mut self, closable: bool) -> Self {
        self.close_on_backdrop = closable;
        self
    }

    pub fn on_dismiss(
        mut self,
        handler: impl Fn(DismissReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

impl Styled for Dialog {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }
        let theme = cx.theme();
        let c = &theme.colors;
        let viewport = window.viewport_size();
        let dismiss = move |handler: &Option<DismissHandler>, reason| {
            let handler = handler.clone();
            move |window: &mut Window, cx: &mut App| {
                if let Some(handler) = &handler {
                    handler(reason, window, cx);
                }
            }
        };
        let on_escape = dismiss(&self.on_dismiss, DismissReason::Escape);
        let on_backdrop = dismiss(&self.on_dismiss, DismissReason::Backdrop);
        let on_close = dismiss(&self.on_dismiss, DismissReason::CloseButton);
        let close_on_backdrop = self.close_on_backdrop;

        let header = (self.title.is_some() || self.close_button).then(|| {
            div()
                .flex()
                .items_start()
                .gap(px(8.))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .children(self.title.clone().map(|title| {
                            div()
                                .text_size(theme.metrics.text_lg)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(c.text)
                                .child(title)
                        }))
                        .children(self.description.clone().map(|description| {
                            div()
                                .text_size(theme.metrics.text_md)
                                .text_color(c.text_muted)
                                .child(description)
                        })),
                )
                .when(self.close_button, |this| {
                    this.child(
                        Button::icon("dialog-close", "✕", "Close dialog")
                            .small()
                            .on_click(move |_, window, cx| on_close(window, cx)),
                    )
                })
        });

        let panel = div()
            .id("dialog")
            .role(Role::Dialog)
            .when_some(self.title, |this, title| this.aria_label(title))
            .when_some(self.description, |this, description| {
                this.aria_description(description)
            })
            .a11y_synthetic_children(|tree| tree.parent_node().set_modal())
            .key_context(DIALOG_CONTEXT)
            .on_action(move |_: &Cancel, window, cx| on_escape(window, cx))
            // Presses inside the panel must not reach the backdrop.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .w(self.width)
            .max_w(viewport.width - px(32.))
            .max_h(viewport.height - px(64.))
            .flex()
            .flex_col()
            .gap(px(14.))
            .p(px(16.))
            .bg(c.surface_dialog)
            .border_1()
            .border_color(c.border_strong)
            .rounded(theme.metrics.radius_card)
            .shadow(theme.overlay_shadow())
            .text_size(theme.metrics.text_md)
            .text_color(c.text)
            .children(header)
            .when(!self.children.is_empty(), |this| {
                this.child(div().flex_1().min_h_0().children(self.children))
            })
            .when(!self.footer.is_empty(), |this| {
                this.child(div().flex().justify_end().gap(px(6.)).children(self.footer))
            })
            .refine_style(&self.style)
            .focus_trap((self.id.clone(), "trap"), &self.focus);

        deferred(
            gpui::anchored()
                .position(gpui::point(px(0.), px(0.)))
                .child(
                    div()
                        .id(self.id)
                        .occlude()
                        .w(viewport.width)
                        .h(viewport.height)
                        .bg(c.scrim)
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            cx.stop_propagation();
                            if close_on_backdrop {
                                on_backdrop(window, cx);
                            }
                        })
                        .child(motion::dialog_in("dialog-in", panel)),
                ),
        )
        .with_priority(DIALOG_PRIORITY)
        .into_any_element()
    }
}
