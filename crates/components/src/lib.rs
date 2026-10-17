//! Zkit components: styled, accessible building blocks for developer tools.
//!
//! This crate is the presentation layer over the foundation crate
//! [`gpui_base`] (re-exported as [`base`]), which owns interaction,
//! focus, accessibility semantics and positioning. Everything here is
//! platform-neutral Rust view code shared by native and browser builds;
//! platform facts (such as the operating-system reduced-motion setting) are
//! reported in by the host application.
//!
//! ```ignore
//! gpui_platform::application().run(|cx| {
//!     zkit_components::init(cx);
//!     Theme::dark(Accent::Indigo).install(cx);
//!     // open windows; render `Root::new().child(...)` at the top of each view
//! });
//! ```

pub mod button;
pub mod focus;
pub mod layout;
pub mod motion;
pub mod progress;
pub mod pulse;
pub mod surface;
pub mod theme;

/// The foundation layer, for primitives this crate does not restyle.
pub use gpui_base as base;

pub use button::{Button, ButtonColors, ButtonSize, ButtonVariant, Kbd, format_keystroke};
pub use focus::{Disclosure, FocusNext, FocusPrevious, Root, focus_next, focus_previous};
pub use gpui_base::{Align, Placement};
pub use layout::{
    Badge, Divider, Panel, Sidebar, SidebarItem, SidebarSection, Toolbar, h_stack, spacer, v_stack,
};
pub use motion::{MotionPolicy, MotionPreference};
pub use progress::{ProgressBar, PulseLoader, Spinner};
pub use surface::{
    Dialog, DismissReason, Menu, MenuItem, Popover, PopoverHandle, PopoverKind, card,
};
pub use theme::{Accent, ActiveTheme, Appearance, Colors, Metrics, Theme};

use gpui::App;

/// Install foundation infrastructure, key bindings and default theme and
/// motion policy. Call once before opening windows. An already-installed
/// theme or motion policy is kept.
pub fn init(cx: &mut App) {
    gpui_base::init(cx);
    focus::init(cx);
    layout::init(cx);
    surface::init(cx);
    if !cx.has_global::<Theme>() {
        Theme::default().install(cx);
    }
    if !cx.has_global::<MotionPolicy>() {
        MotionPolicy::default().install(cx);
    }
}
