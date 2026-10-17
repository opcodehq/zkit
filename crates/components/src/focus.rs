//! Keyboard focus: trap-aware Tab traversal and disclosure focus restoration.
//!
//! [`Root`] is the element applications place at the top of each window. It
//! paints the theme's base surface and handles Tab / Shift-Tab, keeping
//! focus inside the active focus trap (an open dialog) when there is one.
//!
//! [`Disclosure`] is the view-owned open state of a dialog or sheet: opening
//! records the previously focused element and moves focus into the surface;
//! closing restores it.

use gpui::{
    AnyElement, App, FocusHandle, InteractiveElement as _, IntoElement, KeyBinding, ParentElement,
    RenderOnce, StyleRefinement, Styled, Window, actions, div,
};
use gpui_base::StyledExt as _;
use smallvec::SmallVec;

use crate::theme::ActiveTheme as _;

actions!(zkit, [FocusNext, FocusPrevious]);

pub(crate) const ROOT_CONTEXT: &str = "ZkitRoot";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, Some(ROOT_CONTEXT)),
        KeyBinding::new("shift-tab", FocusPrevious, Some(ROOT_CONTEXT)),
    ]);
}

/// Move focus to the next tab stop, cycling inside the active focus trap.
pub fn focus_next(window: &mut Window, cx: &mut App) {
    traverse(window, cx, true);
}

/// Move focus to the previous tab stop, cycling inside the active focus trap.
pub fn focus_previous(window: &mut Window, cx: &mut App) {
    traverse(window, cx, false);
}

fn traverse(window: &mut Window, cx: &mut App, forward: bool) {
    let step = |window: &mut Window, cx: &mut App| {
        if forward {
            window.focus_next(cx)
        } else {
            window.focus_prev(cx)
        }
    };
    let Some(trap) = gpui_base::active_focus_trap(window, cx) else {
        step(window, cx);
        return;
    };
    // Step until focus lands back inside the trap. Tab order is a cycle, so
    // stop if traversal returns to where it started (a trap with no stops).
    let start = window.focused(cx);
    for _ in 0..256 {
        step(window, cx);
        if trap.contains_focused(window, cx) {
            return;
        }
        if window.focused(cx) == start {
            break;
        }
    }
    trap.focus(window, cx);
}

/// Window root: base surface, default text style and trap-aware Tab handling.
#[derive(IntoElement)]
pub struct Root {
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 2]>,
}

impl Root {
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            children: SmallVec::new(),
        }
    }
}

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

impl Styled for Root {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Root {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Root {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .key_context(ROOT_CONTEXT)
            .on_action(|_: &FocusNext, window, cx| focus_next(window, cx))
            .on_action(|_: &FocusPrevious, window, cx| focus_previous(window, cx))
            .size_full()
            .relative()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .font_family(theme.font_sans.clone())
            .text_size(theme.metrics.text_md)
            .children(self.children)
            .refine_style(&self.style)
    }
}

/// View-owned open state for a modal surface, with focus restoration.
///
/// ```ignore
/// struct MyView { confirm: Disclosure }
/// // open from a button:  self.confirm.open(window, cx)
/// // render:              Dialog::new("confirm", &self.confirm).on_dismiss(...)
/// ```
pub struct Disclosure {
    open: bool,
    focus_handle: FocusHandle,
    restore_focus: Option<FocusHandle>,
}

impl Disclosure {
    pub fn new(cx: &mut App) -> Self {
        Self {
            open: false,
            focus_handle: cx.focus_handle(),
            restore_focus: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Focus handle tracked by the surface's container.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Open the surface, remembering the current focus and focusing the surface.
    pub fn open(&mut self, window: &mut Window, cx: &mut App) {
        if self.open {
            return;
        }
        self.open = true;
        self.restore_focus = if self.focus_handle.contains_focused(window, cx) {
            None
        } else {
            window.focused(cx)
        };
        window.focus(&self.focus_handle, cx);
    }

    /// Close the surface and return focus to where it was before opening.
    pub fn close(&mut self, window: &mut Window, cx: &mut App) {
        if !self.open {
            return;
        }
        self.open = false;
        if let Some(previous) = self.restore_focus.take() {
            window.focus(&previous, cx);
        }
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut App) {
        if self.open {
            self.close(window, cx)
        } else {
            self.open(window, cx)
        }
    }
}
