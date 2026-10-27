# Zkit Dither website

The zkit website: a home page (`index.html`) and docs (`docs.html`) for the components, the motion system and the dither engine. Component previews are web renderings built from the same theme tokens, metrics, motion curves, loader math and keyboard behaviour as `zkit-components` (`src/theme.js`, `src/motion.js`, `src/components.js`). Every dither, the image lab and SVG exports run the Rust engine compiled to WebAssembly. The site is plain HTML, CSS and JavaScript with no framework, and uses system fonts like the native toolkit.

From this directory:

```sh
npm ci
npm run dev
```

Requires the repository's Rust toolchain and `wasm-bindgen-cli` matching `Cargo.lock` (currently 0.2.129). Install it with `cargo install wasm-bindgen-cli --version 0.2.129 --locked`. The engine builds automatically before development and production builds.

```sh
npm run build
npm run preview
```

Deploy `dist/` to any static host. The development and preview server accept workspace preview hosts under `conductor.show`. No external APIs, tracking, uploaded images or runtime font services are used. Sample media credits are in `public/media/CREDITS.md`.

See [the dither guide](../docs/dither.md) for the native API, engine details, implementation coverage and limitations.

## Deploy

The site is served at https://zkit.opcodehq.com by Cloudflare Workers as static
assets (`wrangler.jsonc`).
`npm run deploy` builds the engine and the site, then uploads `dist/`. Log in
once with `npx wrangler login`, or set `CLOUDFLARE_API_TOKEN` (and
`CLOUDFLARE_ACCOUNT_ID`) in CI.

## Credits

The dither engine adapts [dithercn](https://dithercn.dev) (MIT; notice in `crates/dither/LICENSE-MIT`). Charts, dithered buttons and avatars follow the design of [dither-kit](https://tripwire.sh/dither-kit) by ripgrim. Icons are [Lucide](https://lucide.dev) (ISC). See `NOTICE`, `crates/dither/NOTICE` and the site's Credits page.
