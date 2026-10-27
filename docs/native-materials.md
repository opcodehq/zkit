# Native materials in GPUI

`GlassSurface` bridges AppKit material views into GPUI's existing scene overlay.
Children remain normal GPUI elements: the bridge supplies the material beneath
text and controls, rather than replacing the interaction or layout system.

Run the playground on macOS:

```sh
cargo run -p zkit-components --example glass
cargo run -p zkit-components --example glass -- --light
cargo run -p zkit-components --example glass -- --solid
```

## Using a surface

Enable the overlay once when opening a macOS window, before creating its view:

```rust,ignore
#[cfg(target_os = "macos")]
window.enable_scene_overlay()?;
```

Then use the component with the normal GPUI styling and child APIs:

```rust,ignore
GlassSurface::new()
    .radius(px(20.))
    .w(px(320.))
    .h(px(180.))
    .child(Button::new("continue", "Continue"))
```

Use `.sidebar()` for the system sidebar material and `.opaque(true)` for a
solid appearance. The host should pass its reduced-transparency preference to
`opaque`; automatic observation of that preference is not implemented here.
The native material follows the installed Zkit theme's light/dark appearance.
Use `.radius(...)`, rather than styling individual corners, to keep the native
and GPUI rounded geometry aligned.

## What runs where

| Capability | Implementation |
| --- | --- |
| Liquid Glass surfaces on supported macOS | Public `NSGlassEffectView`, resolved at runtime |
| Glass on older macOS | Public `NSVisualEffectView` popover material |
| macOS sidebar material | Public `NSVisualEffectView` sidebar material |
| Linux, Windows, browser material fallback | Existing GPUI GPU backdrop blur plus translucent tint |
| Solid surfaces | GPUI fill; native material views removed on the next presented frame |
| Controls, keyboard focus, accessibility | Existing GPUI components |

The new native material path does not require private Apple material APIs.
The older, separate overlay-blur path in this repository still uses its existing
compositor integration. Apple documents the system effects at
[NSGlassEffectView](https://developer.apple.com/documentation/appkit/nsglasseffectview)
and [NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview).

## Composition boundaries

Material requests travel with the scene's paint operations, including cached
scene replay. The macOS compositor reuses native views, updates their geometry
and appearance, and removes them when their requests disappear. Material views
return no hit-test target, leaving input routing with GPUI. GPU blur is omitted
for native material requests on the macOS overlay, so it does not paint over the
system effect. Renderers without the native bridge keep the GPU blur.

All native materials share a plane above the base scene and below GPUI overlay
content. Use sibling, non-overlapping material regions. Nested/interleaved glass,
per-surface opacity animation, transformed surfaces, and arbitrary rounded
ancestor clipping are not supported by this first bridge. Rectangular clipping
uses the intersection as the native frame, so a partially clipped surface's
rounded edge may differ from the original geometry. This needs a dedicated
native clipping container before use in complex scrolling layouts.

The playground demonstrates a glass toolbar, floating cards, a vibrancy sidebar,
and standard native window chrome. It includes GPUI navigation, theme, inspector,
and solid-appearance controls. It does not embed native buttons or text fields.

## Further integration opportunities

| Next candidate | Work needed |
| --- | --- |
| Glass menus and popovers | Connect material surfaces to anchored popup placement; resolve stacking with existing overlays |
| System menus and dialogs | Reuse GPUI's existing platform menu and prompt interfaces |
| Native buttons, switches, segmented controls | Stable native child identity, value callbacks, focus and accessibility ownership |
| SwiftUI view hosting | An Objective-C/Swift hosting boundary, sizing contract, lifecycle and input coordination |
| Native text fields | IME, selection, clipboard, undo and focus coordination |
| Merging glass groups | Container integration and a defined mapping from GPUI layout to native effect groups |

## Validation

The native playground was compiled and launched on macOS 26.6.2 with the macOS
26.5 SDK. App-owned window captures show actual system material views alongside
GPUI-rendered content. Linux compilation and scene replay/batching tests also
pass. The browser example type-checks with `RUSTC_BOOTSTRAP=1`; an existing
`wasm_thread` dependency requires an unstable WebAssembly atomic-wait feature.
The browser and Windows appearances have not been visually verified.
