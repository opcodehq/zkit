//! Layout primitives: panels, toolbars and navigation sidebars.
//!
//! These own structure, density and semantics (headings, toolbar and
//! navigation roles, list positions); application state stays with the
//! caller.

use std::rc::Rc;

use gpui::{
    AnimationExt as _, AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, FontWeight,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement, Pixels, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_base::StyledExt as _;
use gpui_base::actions::{SelectDown, SelectFirst, SelectLast, SelectUp};
use smallvec::SmallVec;

use crate::motion::{self, RESIZE};
use crate::theme::ActiveTheme as _;

pub(crate) const LIST_CONTEXT: &str = "ZkitList";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectUp, Some(LIST_CONTEXT)),
        KeyBinding::new("down", SelectDown, Some(LIST_CONTEXT)),
        KeyBinding::new("home", SelectFirst, Some(LIST_CONTEXT)),
        KeyBinding::new("end", SelectLast, Some(LIST_CONTEXT)),
    ]);
}

/// Horizontal flex row, centered on the cross axis.
pub fn h_stack() -> Div {
    gpui_base::h_flex()
}

/// Vertical flex column.
pub fn v_stack() -> Div {
    gpui_base::v_flex()
}

/// Flexible space that pushes following siblings to the far edge.
pub fn spacer() -> Div {
    div().flex_1()
}

/// A 1px hairline divider.
#[derive(IntoElement)]
pub struct Divider {
    vertical: bool,
    strong: bool,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            strong: false,
        }
    }

    pub fn vertical() -> Self {
        Self {
            vertical: true,
            strong: false,
        }
    }

    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let c = &cx.theme().colors;
        let color = if self.strong {
            c.border_strong
        } else {
            c.border
        };
        div().flex_none().bg(color).map(|this| {
            if self.vertical {
                this.w(px(1.)).h_full()
            } else {
                this.h(px(1.)).w_full()
            }
        })
    }
}

/// A small count or status pill.
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    accent: bool,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            accent: false,
        }
    }

    /// Draw with the accent wash, for counts that need attention.
    pub fn accent(mut self) -> Self {
        self.accent = true;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let c = &theme.colors;
        div()
            .flex_none()
            .px(px(5.))
            .min_w(px(16.))
            .h(px(16.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.))
            .text_size(theme.metrics.text_xs)
            .font_weight(FontWeight::MEDIUM)
            .map(|this| {
                if self.accent {
                    this.bg(c.accent_wash).text_color(c.accent)
                } else {
                    this.bg(c.hover).text_color(c.text_muted)
                }
            })
            .child(self.label)
    }
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

/// A dense horizontal action bar with the toolbar role.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    label: SharedString,
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 4]>,
}

impl Toolbar {
    /// `label` names the toolbar for assistive technology ("Editor actions").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            style: StyleRefinement::default(),
            children: SmallVec::new(),
        }
    }
}

impl Styled for Toolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id(self.id)
            .role(Role::Toolbar)
            .aria_label(self.label)
            .flex_none()
            .flex()
            .items_center()
            .gap(px(4.))
            .h(theme.metrics.toolbar_height)
            .px(px(8.))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .children(self.children)
            .refine_style(&self.style)
    }
}

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// A titled region: compact header with trailing actions over a body.
///
/// The body scrolls vertically when its content overflows.
#[derive(IntoElement)]
pub struct Panel {
    id: ElementId,
    title: Option<SharedString>,
    actions: SmallVec<[AnyElement; 2]>,
    footer: Option<AnyElement>,
    bordered: bool,
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 2]>,
}

impl Panel {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            actions: SmallVec::new(),
            footer: None,
            bordered: true,
            style: StyleRefinement::default(),
            children: SmallVec::new(),
        }
    }

    /// Header title; also names the region for assistive technology.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// A trailing header action (usually a small ghost [`crate::Button`]).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Drop the outer border and radius, for panels docked edge to edge.
    pub fn borderless(mut self) -> Self {
        self.bordered = false;
        self
    }
}

impl Styled for Panel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Panel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Panel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let c = &theme.colors;
        let has_title = self.title.is_some();
        let has_header = has_title || !self.actions.is_empty();
        div()
            .id(self.id)
            .role(Role::Region)
            .when_some(self.title.clone(), |this, title| this.aria_label(title))
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_hidden()
            .bg(c.background)
            .when(self.bordered, |this| {
                this.border_1()
                    .border_color(c.border)
                    .rounded(theme.metrics.radius_lg)
            })
            .when(has_header, |this| {
                this.child(
                    h_stack()
                        .flex_none()
                        .h(theme.metrics.panel_header_height)
                        .pl(px(10.))
                        .pr(px(4.))
                        .gap(px(2.))
                        .border_b_1()
                        .border_color(c.border)
                        .children(self.title.map(|title| {
                            div()
                                .id("panel-title")
                                .role(Role::Heading)
                                .aria_level(2)
                                .aria_label(title.clone())
                                .flex_1()
                                .truncate()
                                .text_size(theme.metrics.text_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(c.text)
                                .child(title)
                        }))
                        .when(!has_title, |this| this.child(spacer()))
                        .children(self.actions),
                )
            })
            .child(
                div()
                    .id("panel-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(self.children),
            )
            .children(self.footer.map(|footer| {
                div()
                    .flex_none()
                    .border_t_1()
                    .border_color(c.border)
                    .px(px(10.))
                    .py(px(6.))
                    .child(footer)
            }))
            .refine_style(&self.style)
    }
}

// ---------------------------------------------------------------------------
// Sidebar
// ---------------------------------------------------------------------------

/// A navigation column with optional header and footer slots. Collapsing
/// animates the width over [`RESIZE`] (snapping under reduced motion).
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    label: SharedString,
    width: Option<Pixels>,
    collapsed_width: Pixels,
    collapsed: bool,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 4]>,
}

impl Sidebar {
    /// `label` names the navigation landmark ("Workspace").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            width: None,
            collapsed_width: px(0.),
            collapsed: false,
            header: None,
            footer: None,
            style: StyleRefinement::default(),
            children: SmallVec::new(),
        }
    }

    /// Expanded width; defaults to the theme's sidebar width.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Width while collapsed; 0 hides the sidebar entirely.
    pub fn collapsed_width(mut self, width: Pixels) -> Self {
        self.collapsed_width = width;
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl Styled for Sidebar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Sidebar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let c = &theme.colors;
        let expanded = self.width.unwrap_or(theme.metrics.sidebar_width);
        let (from, to) = if self.collapsed {
            (expanded, self.collapsed_width)
        } else {
            (self.collapsed_width, expanded)
        };
        let hidden = self.collapsed && self.collapsed_width <= px(0.);

        let column = div()
            .id(self.id.clone())
            .role(Role::Navigation)
            .aria_label(self.label)
            .flex_none()
            .flex()
            .flex_col()
            .h_full()
            .overflow_hidden()
            .bg(c.surface)
            .border_r_1()
            .border_color(c.border)
            .children(
                self.header
                    .map(|header| div().flex_none().p(px(8.)).child(header)),
            )
            .when(!hidden, |this| {
                this.child(
                    div()
                        .id("sidebar-content")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .px(px(6.))
                        .py(px(4.))
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .children(self.children),
                )
            })
            .children(self.footer.map(|footer| {
                div()
                    .flex_none()
                    .p(px(8.))
                    .border_t_1()
                    .border_color(c.border)
                    .child(footer)
            }))
            .refine_style(&self.style);

        // Keyed by the target state, so each toggle replays the resize once.
        let animation_id = ElementId::NamedChild(
            std::sync::Arc::new(self.id),
            if self.collapsed { "collapse" } else { "expand" }.into(),
        );
        column.with_animation(animation_id, RESIZE.animation(), move |this, t| {
            this.w(px(motion::lerp(from.into(), to.into(), t)))
        })
    }
}

/// A titled group of [`SidebarItem`]s with list semantics and arrow-key
/// navigation between items.
#[derive(IntoElement)]
pub struct SidebarSection {
    id: ElementId,
    title: Option<SharedString>,
    items: SmallVec<[SidebarItem; 8]>,
}

impl SidebarSection {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            items: SmallVec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn item(mut self, item: SidebarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SidebarItem>) -> Self {
        self.items.extend(items);
        self
    }
}

/// Move focus to an adjacent tab stop, but only while it stays inside
/// `container`; at the ends focus stays put.
pub(crate) fn step_within(
    container: &FocusHandle,
    forward: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let before = window.focused(cx);
    if forward {
        window.focus_next(cx)
    } else {
        window.focus_prev(cx)
    }
    if !container.contains_focused(window, cx) {
        if let Some(before) = before {
            window.focus(&before, cx);
        }
    }
}

/// Move focus to the first or last tab stop inside `container`.
pub(crate) fn jump_within(
    container: &FocusHandle,
    to_end: bool,
    window: &mut Window,
    cx: &mut App,
) {
    for _ in 0..512 {
        let before = window.focused(cx);
        step_within(container, to_end, window, cx);
        if window.focused(cx) == before {
            return;
        }
    }
}

impl RenderOnce for SidebarSection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let container = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let theme = cx.theme();
        let count = self.items.len();
        let (next, previous, first, last) = (
            container.clone(),
            container.clone(),
            container.clone(),
            container.clone(),
        );

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap(px(1.))
            .children(self.title.clone().map(|title| {
                div()
                    .px(px(8.))
                    .pt(px(4.))
                    .pb(px(2.))
                    .text_size(theme.metrics.text_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.colors.text_faint)
                    .child(title)
            }))
            .child(
                div()
                    .id("items")
                    .role(Role::List)
                    .when_some(self.title, |this, title| this.aria_label(title))
                    .track_focus(&container)
                    .key_context(LIST_CONTEXT)
                    .on_action(move |_: &SelectDown, window, cx| {
                        step_within(&next, true, window, cx)
                    })
                    .on_action(move |_: &SelectUp, window, cx| {
                        step_within(&previous, false, window, cx)
                    })
                    .on_action(move |_: &SelectFirst, window, cx| {
                        jump_within(&first, false, window, cx)
                    })
                    .on_action(move |_: &SelectLast, window, cx| {
                        jump_within(&last, true, window, cx)
                    })
                    .flex()
                    .flex_col()
                    .gap(px(1.))
                    .children(
                        self.items
                            .into_iter()
                            .enumerate()
                            .map(|(index, item)| item.position(index + 1, count)),
                    ),
            )
    }
}

type ItemHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One navigation row: optional icon, label and trailing badge.
#[derive(IntoElement)]
pub struct SidebarItem {
    id: ElementId,
    label: SharedString,
    icon: Option<AnyElement>,
    trailing: Option<AnyElement>,
    selected: bool,
    disabled: bool,
    position: Option<(usize, usize)>,
    on_click: Option<ItemHandler>,
}

impl SidebarItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            trailing: None,
            selected: false,
            disabled: false,
            position: None,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    fn position(mut self, index: usize, count: usize) -> Self {
        self.position = Some((index, count));
        self
    }
}

impl RenderOnce for SidebarItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let c = &theme.colors;
        let (hover, active, ring) = (c.hover, c.active, c.focus_ring);
        let disabled = self.disabled;
        gpui_base::Button::new(self.id)
            .role(Role::ListItem)
            .accessibility_label(self.label.clone())
            .aria_selected(self.selected)
            .when_some(self.position, |this, (index, count)| {
                this.aria_position_in_set(index).aria_size_of_set(count)
            })
            .disabled(disabled)
            .when(disabled, |this| {
                this.a11y_synthetic_children(|tree| tree.parent_node().set_disabled())
                    .opacity(0.45)
            })
            .selected(self.selected)
            .flex()
            .items_center()
            .gap(px(8.))
            .h(theme.metrics.control_height_md)
            .px(px(8.))
            .rounded(theme.metrics.radius_md)
            .border_1()
            .border_color(gpui::transparent_black())
            .text_size(theme.metrics.text_md)
            .map(|this| {
                if self.selected {
                    this.bg(c.selected)
                        .text_color(c.text)
                        .font_weight(FontWeight::MEDIUM)
                } else {
                    this.text_color(c.text_muted)
                }
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(hover))
                    .active(move |style| style.bg(active))
            })
            .focus_visible(move |style| style.border_color(ring))
            .when_some(self.on_click.filter(|_| !disabled), |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            })
            .children(self.icon.map(|icon| {
                div()
                    .flex_none()
                    .w(px(16.))
                    .flex()
                    .justify_center()
                    .text_color(if self.selected { c.text } else { c.text_faint })
                    .child(icon)
            }))
            .child(div().flex_1().min_w_0().truncate().child(self.label))
            .children(self.trailing)
    }
}
