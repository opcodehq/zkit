//! Compact buttons with semantic variants, states and keyboard operation.
//!
//! Behavior (pointer and Enter/Space activation, focus, accessibility role)
//! comes from the foundation [`gpui_base::Button`]; this module owns the
//! presentation. Buttons are tab stops, show a ring only for keyboard focus,
//! and expose disabled, toggled and busy states to assistive technology.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Toggled, Window, div, prelude::FluentBuilder as _, px,
};

use crate::progress::Spinner;
use crate::theme::{ActiveTheme as _, Colors, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// High-contrast solid plate: the one primary action in a region.
    Primary,
    /// Accent fill, for confirm actions that should carry the brand hue.
    Accent,
    /// Raised plate with a hairline border. The default.
    #[default]
    Secondary,
    /// Transparent until hovered: toolbars, rows, icon actions.
    Ghost,
    /// Destructive confirm.
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ButtonSize {
    fn height(self, theme: &Theme) -> Pixels {
        let m = &theme.metrics;
        match self {
            ButtonSize::Small => m.control_height_sm,
            ButtonSize::Medium => m.control_height_md,
            ButtonSize::Large => m.control_height_lg,
        }
    }

    fn padding_x(self) -> Pixels {
        match self {
            ButtonSize::Small => px(9.),
            ButtonSize::Medium => px(11.),
            ButtonSize::Large => px(13.),
        }
    }

    fn text_size(self, theme: &Theme) -> Pixels {
        match self {
            ButtonSize::Small => theme.metrics.text_sm,
            ButtonSize::Medium | ButtonSize::Large => theme.metrics.text_md,
        }
    }
}

/// Resolved paint for one variant in one state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonColors {
    pub background: Hsla,
    pub hover: Hsla,
    pub active: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
}

impl ButtonColors {
    /// Paint for `variant`; `selected` covers toggled-on and open-trigger states.
    pub fn resolve(colors: &Colors, variant: ButtonVariant, selected: bool) -> Self {
        let clear = gpui::transparent_black();
        match variant {
            ButtonVariant::Primary => Self {
                background: colors.solid,
                hover: colors.solid_hover,
                active: colors.solid_hover,
                foreground: colors.on_solid,
                border: colors.solid,
            },
            ButtonVariant::Accent => Self {
                background: colors.accent_strong,
                hover: colors.accent_strong.opacity(0.9),
                active: colors.accent_strong.opacity(0.82),
                foreground: colors.on_accent,
                border: colors.accent_strong,
            },
            ButtonVariant::Danger => Self {
                background: colors.danger_strong,
                hover: colors.danger_strong.opacity(0.9),
                active: colors.danger_strong.opacity(0.82),
                foreground: colors.on_danger,
                border: colors.danger_strong,
            },
            ButtonVariant::Secondary => Self {
                background: if selected {
                    colors.surface_raised_hover
                } else {
                    colors.surface_raised
                },
                hover: colors.surface_raised_hover,
                active: colors.surface_raised_hover,
                foreground: colors.text,
                border: colors.border_strong,
            },
            ButtonVariant::Ghost => Self {
                background: if selected { colors.selected } else { clear },
                hover: colors.hover,
                active: colors.active,
                foreground: colors.text,
                border: clear,
            },
        }
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A compact, accessible button.
///
/// ```ignore
/// Button::new("save", "Save").primary().shortcut("cmd-s").on_click(...)
/// Button::icon("close", "×", "Close panel").ghost().small()
/// Button::new("wrap", "Wrap").toggled(self.wrap).on_click(...)
/// ```
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    accessible_label: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    shortcut: Option<SharedString>,
    variant: ButtonVariant,
    size: ButtonSize,
    icon_only: bool,
    full_width: bool,
    disabled: bool,
    selected: bool,
    toggled: Option<bool>,
    expanded: Option<bool>,
    loading: bool,
    tab_index: isize,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// A text button; the label is also its accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::build(id.into(), Some(label.into()), None)
    }

    /// A square icon-only button. `accessible_label` is required because
    /// there is no visible text to name it.
    pub fn icon(
        id: impl Into<ElementId>,
        icon: impl IntoElement,
        accessible_label: impl Into<SharedString>,
    ) -> Self {
        let mut button = Self::build(id.into(), None, Some(accessible_label.into()));
        button.leading = Some(icon.into_any_element());
        button.icon_only = true;
        button.variant = ButtonVariant::Ghost;
        button
    }

    fn build(
        id: ElementId,
        label: Option<SharedString>,
        accessible_label: Option<SharedString>,
    ) -> Self {
        Self {
            id,
            label,
            accessible_label,
            leading: None,
            trailing: None,
            shortcut: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            icon_only: false,
            full_width: false,
            disabled: false,
            selected: false,
            toggled: None,
            expanded: None,
            loading: false,
            tab_index: 0,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }
    pub fn accent(self) -> Self {
        self.variant(ButtonVariant::Accent)
    }
    pub fn secondary(self) -> Self {
        self.variant(ButtonVariant::Secondary)
    }
    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }
    pub fn danger(self) -> Self {
        self.variant(ButtonVariant::Danger)
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    pub fn small(self) -> Self {
        self.size(ButtonSize::Small)
    }
    pub fn large(self) -> Self {
        self.size(ButtonSize::Large)
    }

    /// Element before the label (an icon glyph).
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Element after the label (a count, chevron).
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Show a keyboard hint and announce it to assistive technology. Uses
    /// gpui keystroke notation (`"cmd-s"`, `"ctrl-shift-p"`). This does not
    /// bind the key; register the binding in the application keymap.
    pub fn shortcut(mut self, keystroke: impl Into<SharedString>) -> Self {
        self.shortcut = Some(keystroke.into());
        self
    }

    /// Override the accessible name (defaults to the visible label).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Persistent selected presentation, e.g. while the button's menu is open.
    /// Purely visual; use [`toggled`](Self::toggled) for on/off state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Make this a toggle button: announces pressed state and draws selected.
    pub fn toggled(mut self, on: bool) -> Self {
        self.toggled = Some(on);
        self
    }

    /// Announce that this button controls a surface that is open or closed.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Show a spinner and ignore activation while work is in flight.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Activation handler for pointer clicks and Enter/Space on keyboard focus.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Run `extra` after any handler the caller installed.
    pub(crate) fn also_on_click(
        mut self,
        extra: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        let previous = self.on_click.take();
        self.on_click = Some(Rc::new(move |event, window, cx| {
            if let Some(previous) = &previous {
                previous(event, window, cx);
            }
            extra(event, window, cx);
        }));
        self
    }
}

impl gpui_base::Selectable for Button {
    fn selected(self, selected: bool) -> Self {
        Button::selected(self, selected)
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl gpui_base::Disableable for Button {
    fn disabled(self, disabled: bool) -> Self {
        Button::disabled(self, disabled)
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let selected = self.selected || self.toggled == Some(true);
        let paint = ButtonColors::resolve(&theme.colors, self.variant, selected);
        let height = self.size.height(theme);
        let inert = self.disabled || self.loading;
        let ring = theme.colors.focus_ring;
        let foreground = paint.foreground;
        let resting_shadow = match self.variant {
            ButtonVariant::Primary | ButtonVariant::Accent | ButtonVariant::Danger => {
                theme.filled_shadow(paint.background)
            }
            ButtonVariant::Secondary => theme.control_shadow(),
            ButtonVariant::Ghost => Vec::new(),
        };
        let focus_shadow = vec![gpui::BoxShadow {
            color: ring.opacity(0.45),
            offset: gpui::point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(2.),
            inset: false,
        }];
        let accessible_label = self.accessible_label.or_else(|| self.label.clone());
        let (disabled, loading) = (self.disabled, self.loading);

        let leading = if self.loading {
            Some(
                Spinner::new()
                    .size(px(14.))
                    .color(foreground)
                    .into_any_element(),
            )
        } else {
            self.leading
        };

        gpui_base::Button::new(self.id)
            .disabled(inert)
            .selected(selected)
            .tab_index(self.tab_index)
            .when_some(accessible_label, |this, label| {
                this.accessibility_label(label)
            })
            .when_some(self.toggled, |this, on| {
                this.aria_toggled(if on { Toggled::True } else { Toggled::False })
            })
            .when_some(self.expanded, |this, expanded| this.aria_expanded(expanded))
            .when_some(self.shortcut.clone(), |this, keys| {
                this.aria_keyshortcuts(keys)
            })
            .when(disabled || loading, |this| {
                this.a11y_synthetic_children(move |tree| {
                    let node = tree.parent_node();
                    if disabled {
                        node.set_disabled();
                    }
                    if loading {
                        node.set_busy();
                    }
                })
            })
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(6.))
            .h(height)
            .when(self.icon_only, |this| this.w(height))
            .when(!self.icon_only, |this| this.px(self.size.padding_x()))
            .when(self.full_width, |this| this.w_full())
            .rounded(theme.metrics.radius_lg)
            .border_1()
            .border_color(paint.border)
            .bg(paint.background)
            .when(!disabled, |this| this.shadow(resting_shadow))
            .text_color(foreground)
            .text_size(self.size.text_size(theme))
            .font_weight(FontWeight::MEDIUM)
            .line_height(px(20.))
            .whitespace_nowrap()
            .overflow_hidden()
            .when(!inert, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(paint.hover))
                    .active(move |style| style.bg(paint.active).shadow(Vec::new()))
            })
            .when(disabled, |this| this.opacity(0.64))
            .focus_visible(move |style| style.border_color(ring).shadow(focus_shadow))
            .when_some(
                (!inert).then_some(self.on_click).flatten(),
                |this, handler| this.on_click(move |event, window, cx| handler(event, window, cx)),
            )
            .children(leading)
            .children(self.label)
            .children(self.trailing)
            .when_some(self.shortcut, |this, keys| {
                this.child(Kbd::new(keys).muted_on(foreground))
            })
    }
}

/// A keyboard shortcut hint rendered as compact key caps.
///
/// Takes gpui keystroke notation (`"cmd-shift-p"`) and renders platform
/// symbols on macOS and words elsewhere.
#[derive(IntoElement)]
pub struct Kbd {
    keystroke: SharedString,
    color: Option<Hsla>,
}

impl Kbd {
    pub fn new(keystroke: impl Into<SharedString>) -> Self {
        Self {
            keystroke: keystroke.into(),
            color: None,
        }
    }

    /// Tint the hint relative to a host foreground (inside a filled button).
    pub fn muted_on(mut self, foreground: Hsla) -> Self {
        self.color = Some(foreground.opacity(0.6));
        self
    }
}

/// Display text for gpui keystroke notation.
pub fn format_keystroke(keystroke: &str, mac: bool) -> String {
    keystroke
        .split(' ')
        .map(|chord| {
            let parts: Vec<&str> = chord.split('-').collect();
            let (key, modifiers) = match parts.split_last() {
                // A trailing empty part means the key itself is "-".
                Some((&"", rest)) => ("-", &rest[..rest.len().saturating_sub(1)]),
                Some((key, rest)) => (*key, rest),
                None => ("", &[][..]),
            };
            let mut out = String::new();
            for modifier in modifiers {
                let symbol = match (*modifier, mac) {
                    ("cmd" | "secondary", true) => "⌘",
                    ("ctrl", true) => "⌃",
                    ("alt", true) => "⌥",
                    ("shift", true) => "⇧",
                    ("fn", true) => "fn",
                    ("cmd" | "secondary", false) => "Ctrl+",
                    ("ctrl", false) => "Ctrl+",
                    ("alt", false) => "Alt+",
                    ("shift", false) => "Shift+",
                    ("fn", false) => "Fn+",
                    (other, _) => other,
                };
                out.push_str(symbol);
            }
            let key = match key {
                "enter" => if mac { "↩" } else { "Enter" }.to_string(),
                "escape" => "Esc".to_string(),
                "backspace" => if mac { "⌫" } else { "Backspace" }.to_string(),
                "tab" => if mac { "⇥" } else { "Tab" }.to_string(),
                "space" => "Space".to_string(),
                "up" => "↑".to_string(),
                "down" => "↓".to_string(),
                "left" => "←".to_string(),
                "right" => "→".to_string(),
                other if other.chars().count() == 1 => other.to_uppercase(),
                other => {
                    let mut chars = other.chars();
                    chars
                        .next()
                        .map(|first| first.to_uppercase().chain(chars).collect())
                        .unwrap_or_default()
                }
            };
            out.push_str(&key);
            out
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl RenderOnce for Kbd {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mac = cfg!(target_os = "macos");
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .h(px(20.))
            .min_w(px(20.))
            .px(px(4.))
            .rounded(px(4.))
            .when(self.color.is_none(), |this| this.bg(theme.colors.hover))
            .text_size(theme.metrics.text_xs)
            .font_weight(FontWeight::MEDIUM)
            .line_height(px(16.))
            .text_color(self.color.unwrap_or(theme.colors.text_muted))
            .child(format_keystroke(&self.keystroke, mac))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Accent, contrast_ratio, over};

    #[test]
    fn every_variant_keeps_label_contrast_at_rest_and_on_hover() {
        for theme in [Theme::dark(Accent::Indigo), Theme::light(Accent::Indigo)] {
            let c = &theme.colors;
            for variant in [
                ButtonVariant::Primary,
                ButtonVariant::Accent,
                ButtonVariant::Secondary,
                ButtonVariant::Ghost,
                ButtonVariant::Danger,
            ] {
                for selected in [false, true] {
                    let paint = ButtonColors::resolve(c, variant, selected);
                    for (state, fill) in [("rest", paint.background), ("hover", paint.hover)] {
                        // Buttons sit on content or chrome; check the harder one.
                        for plane in [c.background, c.surface] {
                            let plate = over(fill, plane);
                            let ratio = contrast_ratio(over(paint.foreground, plate), plate);
                            assert!(
                                ratio >= 4.5,
                                "{:?} {variant:?} selected={selected} {state}: {ratio}",
                                theme.appearance
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn selection_changes_quiet_variants_but_not_filled_ones() {
        let c = &Theme::dark(Accent::Indigo).colors;
        let ghost = |s| ButtonColors::resolve(c, ButtonVariant::Ghost, s);
        assert_ne!(ghost(false).background, ghost(true).background);
        let primary = |s| ButtonColors::resolve(c, ButtonVariant::Primary, s);
        assert_eq!(primary(false), primary(true));
    }

    #[test]
    fn keystrokes_format_per_platform() {
        assert_eq!(format_keystroke("cmd-shift-p", true), "⌘⇧P");
        assert_eq!(format_keystroke("cmd-shift-p", false), "Ctrl+Shift+P");
        assert_eq!(format_keystroke("escape", true), "Esc");
        assert_eq!(format_keystroke("ctrl-k ctrl-s", false), "Ctrl+K Ctrl+S");
        assert_eq!(format_keystroke("cmd--", true), "⌘-");
        assert_eq!(format_keystroke("f12", false), "F12");
    }
}
