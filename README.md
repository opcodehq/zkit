# Zkit

Zuse's GPU interface toolkit for native macOS/Linux applications and browser WebAssembly clients.

The toolkit maintains GPUI platform, rendering, text and input foundations alongside reusable components. Browser and native targets share Rust view code; platform-specific capabilities stay behind explicit interfaces.

## Workspace

- `crates/gpui`: UI primitives, layout, interaction and accessibility.
- `crates/gpui_platform`: platform selection and application lifecycle.
- `crates/gpui_web`: browser window, input, clipboard and WASM integration.
- `crates/gpui_wgpu`: portable GPU and text rendering.
- Native platform crates implement macOS, Linux and Windows backends.

See `NOTICE`, `LICENSE-APACHE` and `LICENSE-GPL` for retained upstream licensing.

## Dither

The portable `zkit-dither` engine powers GPUI texture surfaces and a WebAssembly playground. It includes 35 background/chart fields, Bayer and blue-noise matrices, palettes, image diffusion and SVG export. Native components include textured controls, data-driven charts and cached images.

Run `cargo run -p zkit-components --example dither` for the native gallery, or see [website/README.md](website/README.md) for the website, which documents the components, motion and dither. The [dither guide](docs/dither.md) documents the API, supported examples and remaining interaction effects.
