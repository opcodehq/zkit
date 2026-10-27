# Dither

Zkit's dither engine is shared between native GPUI surfaces and an interactive website. The engine produces deterministic RGBA rasters from normalized tone fields. The GPUI adapter paints color runs as batched GPU quads; the website compiles the same Rust crate to WebAssembly and presents its output on a canvas.

## Run

```sh
cargo run -p zkit-components --example dither
```

For the website, install the Rust toolchain specified in `rust-toolchain.toml`, Node.js 20.19+ or 22.12+, and the `wasm-bindgen-cli` version matching the `wasm-bindgen` package in `Cargo.lock` (currently 0.2.129).

```sh
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cd website
npm ci
npm run dev
```

The predev script builds the Rust engine automatically. `npm run build` creates a static deployment in `website/dist`; `npm run preview` serves it. Generated bindings and build output are ignored by Git. The site uses system fonts, and image conversion happens entirely inside the browser.

## GPUI surfaces

Initialize `zkit_components::init(cx)` once before opening a window. A surface defaults to the active theme's background and accent colors, and accepts GPUI styles and children.

```rust,ignore
use gpui::{prelude::*, px};
use zkit_components::{DitherSurface, Matrix, Palette, Preset};

DitherSurface::new(Preset::Aurora)
    .matrix(Matrix::Bayer8)
    .palette(Palette::duo([16, 21, 18], [208, 251, 116]))
    .cell_size(4.)
    .w_full().h(px(320.))
    .child(your_content)
```

Supply your own tone field with `.field(|x, y, seconds, aspect| ...)`. Coordinates are normalized to 0–1 and the result is clamped to that interval. Non-finite tones become paper. `aspect` is width / height, for preserving circular geometry. Foreground controls keep their GPUI focus, activation and accessibility behavior; the field itself is decorative.

```rust,ignore
DitherSurface::new(Preset::Gradient)
    .field(|x, y, t, _aspect| {
        0.5 + 0.5 * (x * 12. + y * 4. + t).sin()
    })
    .w_full().h(px(240.))
```

`.animated(false)` freezes the field at `.still_time(seconds)` (default 6). Animated surfaces share the existing leased clock at approximately 15 fps. Surfaces outside the rectangular content mask do not renew their lease. Reduced motion uses the still timestamp and schedules no animation. Hosts continue to report system preferences through the existing motion policy. `cell_size` is in logical pixels; a surface coarsens its cells to stay near 32,000 samples rather than scaling work with display density.

## Components

- `DitherButton::new(id, label)`: a foundation button with a dithered face that grows denser on hover. `.fill(ChartFill::{Gradient, Dotted, Hatched, Solid})`, and `.color(hsla)` (default accent). Supports `on_click` and `disabled`.
- `DitherChart::new(id, title)`: data charts (`ChartKind::Radar` too, plus `.stacked(true)` for areas and bars, and `.fill(ChartFill)` per series; defaults run gradient, hatched, dotted, solid) with a title, optional description, legend, y-axis with rounded ticks, gridlines and x-axis labels. Add `.categories(labels)` and one or more `.series(name, values)`; series take the theme's five chart colours (`Colors::chart`). `ChartKind::{Area, Line, Bar, Donut, Heatmap}`: area and line overlay series, bars are grouped, a donut shows the first series by category with a total and a legend table, and a heatmap uses one series per row with a Less→More scale. `.format(fn)` controls how values read (default compact: `1.2k`). Negative and non-finite values count as zero. The element exposes every value as a text summary.
- `DitherLoader::new(id, label)`: a busy progress indicator with `LoaderVariant::{Ring, Dots, Pulse, Bar}` and `LoaderSize::{Small, Medium, Large}` (24, 40, 64 px). Holds a still frame under reduced motion. `dither_loader(id, label)` is the default ring.
- `DitherGlobe::new(id)`: a rotating globe with a lit terminator, atmosphere, `.markers(..)` and `.arcs(..)`, `.speed(..)` and `.tilt(..)`. Land uses the accent; markers and arcs use the warning colour. Land comes from a 480 × 240 Natural Earth mask (public domain) in `zkit-dither`.
- `DitherImage::new(id, &raster, label)`: cached GPU texture with an accessible description. Construct it once, then clone it in views to reuse its atlas entry. Alpha is preserved; RGBA is converted to GPUI's BGRA storage format once.
- `DitherAvatar::new(id, name)` / `dither_avatar(id, name)`: a generative avatar from `zkit_dither::Avatar`: 32 name-derived bits folded left/right or top/bottom into an 8 × 8 pattern, one of 180 hues, drawn in dithered blocks. `.size(px)`, default 40.
- `dither_card()`, `dither_divider()`, `dither_skeleton()`, `dither_loader()`: composable surfaces. Give busy states a host progress/status label when needed.
- `dither_empty_state(title, description, action)` and `dither_not_found(action)`: layouts accepting real native actions.

```rust,ignore
use zkit_components::{DitherChart, ChartKind};

DitherChart::new("deploys", "Deploys")
    .kind(ChartKind::Bar)
    .categories(["Jan", "Feb", "Mar", "Apr", "May", "Jun"])
    .series("Production", [42., 58., 51., 66., 72., 80.])
    .series("Preview", [120., 140., 98., 160., 172., 190.])
    .plot_height(px(180.))
```

## Portable engine

`zkit-dither` has no GPUI dependency. `Dither::render` accepts a custom field; `render_preset` accepts a built-in field. Rasters expose their RGBA bytes, dimensions and horizontal-run SVG export. Maximum raster size is four million cells, with checked dimensions and buffers.

```rust,ignore
use zkit_dither::{Dither, Preset};

let raster = Dither::default().render_preset(320, 180, Preset::Waves, 6.)?;
std::fs::write("waves.svg", raster.svg())?;
// Feed raster.pixels() into an image encoder for PNG.
```

`Plot::render` draws chart marks (areas, lines, bars, donuts, radars with fills and stacking), `Avatar::from_name` builds avatars, and `Globe::render` and `LoaderVariant::render` produce transparent rasters for the globe and loaders; the GPUI components and the website both draw these.

`Matrix::{Bayer2, Bayer4, Bayer8, BlueNoise}` provides midpoint thresholds. Palettes contain two or more colors ordered from paper to ink; intermediate tones dither between adjacent palette entries. The 64×64 blue-noise tile uses 256 ranks, each repeated 16 times.

`dither_image(width, height, rgba, palette, method)` converts image buffers without mutating the input. Available techniques are ordered Bayer/blue noise, Floyd–Steinberg, Atkinson and Sierra Lite. Diffusion uses serpentine scanning and ignores fully transparent pixels when distributing error. Ordered conversion maps luminance between palette endpoint luminances; diffusion maps to the nearest palette color with a green-weighted distance. Ordered conversion is intended for luminance-ordered palettes; use diffusion for arbitrary color palettes.

The website (`website/`) documents all of zkit: components, motion and dither. Its Dither section covers `DitherSurface` with a field and matrix picker, a page per built-in field (ink colour, matrix, cell size, contrast, Rust snippet, PNG/SVG export), `DitherButton`, `DitherChart` (area, bar, line, donut and heatmap with real data, axes, legends and hover tooltips, mirroring `chart.rs`), avatars, the image lab, cards and states, larger examples (dithered type, globe, aurora, waves, photo) and the engine. Site-only fields are written in JavaScript and dithered through the engine's `render_tones` binding; preset previews call `render_field`. Surfaces follow the site's accent and appearance, pause offscreen and in hidden tabs, and follow the site-wide reduced-motion setting.

## Coverage and limits

This is an adaptation of the engine and its examples to Rust/GPUI, with the following coverage:

| Area | Coverage |
| --- | --- |
| Backgrounds | 24 fields: gradient, glow, mesh, aurora, noise, waves, plasma, beams, silk, smoke, contours, ripples, ridgelines, metaballs, tunnel, retro grid, eclipse, horizon, starfield, halftone, globe, dot map, orb, fade |
| Chart fields | 11 decorative sample fields: area, bar, line, sparkline, heatmap, donut, radar, gauge, progress, uptime, stat trend |
| Data-driven charts | Area, line, bar, donut and heatmap components |
| Interface | Button, avatar, card, divider, skeleton, four loader variants, globe, empty-state and not-found helpers |
| Images | Ordered/diffusion conversion, alpha preservation, cached native image and website image lab |
| Text / image interaction effects | Specialized text dissolve/scramble/number effects, image comparison, scroll reveals and logo marquees are not implemented |

Fields are adapted procedural implementations, not shader-identical GLSL translations. The `Globe`/`DitherGlobe` component uses a Natural Earth land mask (public domain); the older `Preset::Globe` and `Preset::DotMap` fields still use procedural land. Multi-hue per-cell color fields, the original CSS/SVG pattern APIs and host-specific React prop interfaces are not exposed. The native renderer evaluates fields on CPU and uses existing GPUI GPU batching; it does not inject custom shaders into the renderer. Rounded ancestor clipping follows GPUI's existing behavior.

## Verification

```sh
cargo test -p zkit-dither
cargo test -p zkit-components --test dither
cargo test -p zkit-components --lib dither_widgets
cargo check -p zkit-components --example dither
cargo check -p zkit-components --target wasm32-unknown-unknown
cd website && npm run build
```

Engine tests check exact matrix coverage, tiling, palette endpoints, invalid dimensions, finite field output, SVG structure, diffusion palette membership, alpha and input preservation. GPUI tests check descendant activation through a textured surface and data-dependent chart geometry. Browser checks cover filtering/search, playground controls, export events, image conversion, responsive overflow and accessibility.

## Attribution

zkit's dither work builds on two projects:

- [dithercn](https://dithercn.dev) by Leo ([opencoredev/dithercn](https://github.com/opencoredev/dithercn), MIT). The blue-noise threshold data, dithering techniques, globe (including the Natural Earth land mask), loaders and many example fields were adapted from it. Its copyright and permission notice are retained in `crates/dither/LICENSE-MIT`.
- [dither-kit](https://tripwire.sh/dither-kit) by ripgrim. The charts' fill styles, stacking, ribbon lines and radar, the dithered buttons and the generative avatars follow its design; zkit's implementations are written from scratch in Rust.

World land data is from Natural Earth (public domain). Website icons are from Lucide (ISC). Font notices are retained in `assets/fonts/zkit/licenses/`. The website's credits page (`docs.html#/credits`) lists the same; `NOTICE` and `crates/dither/NOTICE` record it for the repository.
