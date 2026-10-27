# Zkit components

`crates/components` (`zkit-components`) is the styled presentation layer of
Zkit. It sits on the foundation crate `crates/component-base` (`gpui-base`),
which owns interaction, focus, accessibility semantics and positioning, and
re-exports it as `zkit_components::base` for primitives this layer does not
restyle (inputs, selects, tables, trees, sliders, toasts, and so on).

The crate depends only on `gpui`, `gpui-base`, `serde`, `smallvec` and
`web-time`, and has no platform crates, so the same view code compiles for
native targets and `wasm32-unknown-unknown`.

## Setup

```rust
gpui_platform::application().run(|cx| {
    zkit_components::init(cx);              // foundation + key bindings + defaults
    Theme::dark(Accent::Indigo).install(cx); // or Theme::light(..)
    MotionPolicy { preference: saved_preference, ..Default::default() }.install(cx);
    // Platform layer: report the OS setting when known / when it changes.
    zkit_components::motion::set_system_reduces_motion(os_reduces_motion, cx);
    // open windows...
});
```

Render `Root::new().child(...)` at the top of each window. `Root` paints the
base surface, sets the default text style, and handles Tab / Shift-Tab so
focus stays inside an open dialog.

## Modules

| Module     | Contents |
|------------|----------|
| `theme`    | `Theme` (appearance, accent, `Colors`, `Metrics`, fonts), `ActiveTheme::theme()`, OKLCH helpers, `contrast_ratio`. `Theme::install` also projects tokens into the foundation theme. |
| `motion`   | `CubicBezier`, `MotionSpec` catalog (`ENTER`, `MENU_IN`, `DIALOG_IN`, `RESIZE`, ...), `fade_in` / `fade_quick` / `menu_in` / `dialog_in`, `MotionPolicy` and `MotionPreference`. |
| `pulse`    | Shared lease-based loader clock (about 30 or 15 fps; parks when no loader is painted) and pure phase math. |
| `focus`    | `Root`, trap-aware `focus_next` / `focus_previous`, `Disclosure` (open state with focus restore). |
| `button`   | `Button` (Primary, Accent, Secondary, Ghost, Danger; Small, Medium, Large; disabled, selected, toggled, expanded, loading; icon-only; shortcut hint), `Kbd`, `format_keystroke`. |
| `layout`   | `h_stack`, `v_stack`, `spacer`, `Divider`, `Badge`, `Toolbar`, `Panel` (a frame: title band, inset body plate, footer; `plain_body()` for a `Card` inside), `Card` (title, description, action, body, footer), `Sidebar`, `SidebarSection`, `SidebarItem`. |
| `surface`  | `card`, `Popover` (dialog or menu kind), `Menu`, `MenuItem`, `Dialog` (header, body, and actions in a muted footer band), `DismissReason`. |
| `progress` | `ProgressBar` (determinate or indeterminate), `Spinner`, `PulseLoader`. |

## Look

Controls are 32 px tall (28 small, 36 large) with 10 px corners and 14 px
text. Dialogs, cards and frames use 18 px corners. Bordered plates rest on a
hairline and a soft shadow (`Theme::control_shadow`); floating surfaces use
`Theme::overlay_shadow`; filled buttons add a one-pixel top highlight
(`Theme::filled_shadow`). Neutrals are achromatic; the accent colours focus
rings, accent buttons and dither ink.

## Behavior contract

- **Buttons** are tab stops. Enter or Space activates the focused button (the
  foundation's keyboard click), and the focus ring appears only for keyboard
  focus. Disabled and loading buttons ignore activation. Assistive technology
  is told about disabled, toggled (pressed), expanded, busy and keyboard
  shortcuts.
- **Popover** opens on pointer press, or on Enter/Space on the focused
  trigger. Focus moves into the surface; Escape or a press outside closes it,
  and focus returns to the trigger. Placement uses the foundation
  `Positioner`, which flips and clamps to the window.
  `Popover::menu` adds menu semantics and Up/Down/Home/End navigation;
  `MenuItem::dismisses(handle)` closes the menu after selection.
- **Dialog** is driven by a view-owned `Disclosure`. `Disclosure::open`
  records the current focus and focuses the dialog; Tab cycles inside it via
  the foundation focus trap; Escape, the close button and backdrop presses
  call `on_dismiss(reason)`. The view decides whether to call
  `Disclosure::close`, which restores focus. Enter is not bound at the
  dialog level, so a focused button inside it activates normally.
- **SidebarSection** is a list. Up/Down/Home/End move between items without
  leaving the section; Enter or Space activates the focused item.
- **Motion**: every `with_animation` helper honors `App::reduce_motion`
  (oneshots snap to their end state). Loaders read the pulse clock, which
  returns a still phase when motion is reduced. Activity indicators keep a
  gentle, non-travelling brightness cue when only the OS asks for reduced
  motion, and stay fully still for an explicit user preference or a
  background pause.

## Gallery

```sh
cargo run -p zkit-components --example gallery
```

The gallery shows a toolbar (sidebar toggle, accent and motion menus,
appearance switch), a collapsible sidebar with sections and a badge, button
variants, sizes and states, progress indicators, popovers, a menu, the modal
dialog and a replayable entrance. The example also has a `wasm32` entry point
following the GPUI example pattern.

## Tests

`cargo test -p zkit-components` runs:

- Unit tests: WCAG contrast for text, filled-button labels and focus rings
  across appearances and accents; bezier accuracy against CSS reference
  values; reduced-motion policy resolution; pulse math; keystroke formatting.
- `tests/behavior.rs`: menu keyboard open, arrow selection and Escape focus
  restore; dialog Tab trap, Enter activating a focused button, and Escape
  dismissal; sidebar arrow navigation; inert loading button.

## Known gaps

- Divs have no scale transform, so popover and dialog entrances use a fade
  with a short rise.
- Closing is immediate: there is no exit animation.
- The foundation `Popover` / `Dialog` elements are not used directly. The
  popover uses foundation `PopoverState` and `Positioner` so the trigger can
  open from the keyboard. The dialog uses the foundation focus trap and
  `Cancel` action because the foundation dialog binds Enter to confirm, which
  would pre-empt a focused button.
- Reading the operating-system reduced-motion setting is left to the host
  platform layer (`set_system_reduces_motion`).
