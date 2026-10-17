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
