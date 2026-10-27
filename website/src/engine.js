// The site's renderer. Fields compute a tone per cell; the zkit Rust engine
// (compiled to WebAssembly) turns tones into dots with a threshold matrix.
// Every surface pauses offscreen, in hidden tabs and under reduced motion.
//
// Credits: dithering techniques adapted from dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT.
import init, {
  render_tones,
  render_chart,
  render_avatar,
  avatar_hue,
  render_field,
  render_globe,
  render_loader,
  loader_grid,
  convert_image,
  export_svg,
  catalog,
} from "./engine/dither.js";

import { reducesMotion, onMotionChange } from "./motion.js";

export { render_chart, render_avatar, avatar_hue, render_field, render_globe, render_loader, loader_grid, convert_image, export_svg, catalog };

let booted;
let initialized = false;
export const ready = () =>
  (booted ??= init().then(() => {
    initialized = true;
    schedule();
  }));


// Cell budget per surface, so large screens do not scale the work.
const MAX_CELLS = 70_000;
const FRAME_MS = 1000 / 24;

const css = (el, name) => getComputedStyle(el).getPropertyValue(name).trim();
export const rgbVar = (el, name) =>
  css(el, name)
    .split(/[\s,]+/)
    .filter(Boolean)
    .map(Number);
export const hex = (h) =>
  h
    .replace("#", "")
    .match(/../g)
    .map((v) => parseInt(v, 16));

export const COLORS = {
  blue: [63, 175, 243],
  purple: [184, 128, 255],
  green: [79, 213, 127],
  pink: [250, 120, 178],
  orange: [254, 162, 71],
  red: [255, 107, 107],
  grey: [140, 140, 140],
};

const MONO = new Uint8Array([0, 0, 0, 255, 255, 255]);

const surfaces = new Set();
const visible = new Set();
const observer = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      const surface = entry.target.__surface;
      if (!surface) continue;
      if (entry.isIntersecting) {
        visible.add(surface);
        surface.dirty = true;
      } else visible.delete(surface);
    }
    schedule();
  },
  { rootMargin: "120px" },
);
const resizer = new ResizeObserver((entries) => {
  for (const entry of entries) {
    const surface = entry.target.__surface;
    if (surface) surface.dirty = true;
  }
  schedule();
});

/**
 * Mount a dithered surface into `host`, which it fills.
 *
 * - `field(u, v, t, s)`: tone in 0..1 for the cell at u, v (0..1, y down).
 *   `s` carries w, h (cells), aspect, x, y (cell index) and cell (px).
 * - `palette`: colours from paper to ink, for multi-tone fields. The first
 *   colour is drawn transparent unless `opaque` is set.
 * - `inks` + `inkIndex(u, v, t, s)`: two-tone mode, choosing an ink per cell.
 * - `ink`: one ink, defaults to the theme's --ink-rgb.
 */
export function surface(host, options) {
  const canvas = document.createElement("canvas");
  canvas.className = "dither";
  canvas.setAttribute("aria-hidden", "true");
  host.classList.add("dither-layer");
  host.append(canvas);
  const s = {
    host,
    canvas,
    options: {
      cell: 3,
      matrix: "bayer4",
      contrast: 1,
      animated: false,
      still: 6,
      ...options,
    },
    time: options.still ?? 6,
    start: performance.now(),
    dirty: true,
    state: { w: 0, h: 0, aspect: 1, x: 0, y: 0, cell: 3 },
    tones: null,
    set(next) {
      Object.assign(this.options, next);
      this.dirty = true;
      schedule();
    },
    replay() {
      this.start = performance.now();
      this.dirty = true;
      schedule();
    },
    destroy() {
      surfaces.delete(this);
      visible.delete(this);
      observer.unobserve(host);
      resizer.unobserve(host);
      canvas.remove();
    },
  };
  host.__surface = s;
  surfaces.add(s);
  observer.observe(host);
  resizer.observe(host);
  return s;
}

function paint(s) {
  const o = s.options;
  const rect = s.host.getBoundingClientRect();
  if (rect.width < 2 || rect.height < 2) return;
  let cell = o.cell;
  while ((rect.width / cell) * (rect.height / cell) > (o.maxCells ?? MAX_CELLS)) cell += 1;
  const w = Math.ceil(rect.width / cell);
  const h = Math.ceil(rect.height / cell);
  const canvas = s.canvas;
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
    canvas.style.width = `${w * cell}px`;
    canvas.style.height = `${h * cell}px`;
  }
  const st = s.state;
  st.w = w;
  st.h = h;
  st.aspect = w / h;
  st.cell = cell;
  const t = s.time;
  if (o.raster) {
    // A Rust renderer that returns RGBA with transparent paper.
    const rgba = o.raster(w, h, t, s);
    canvas.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(rgba), w, h), 0, 0);
    return;
  }
  const palette = typeof o.palette === "function" ? o.palette(s) : o.palette;
  const multi = !!palette;
  const colors = multi ? new Uint8Array(palette.flat()) : MONO;
  let rgba;
  if (o.preset) {
    rgba = render_field(w, h, o.preset, t, o.matrix, colors, o.contrast);
  } else {
    rgba = renderTones(s, w, h, t, colors);
  }
  draw(s, rgba, w, h, t, multi ? palette : null);
}

function renderTones(s, w, h, t, colors) {
  const o = s.options;
  const st = s.state;
  const n = w * h;
  if (!s.tones || s.tones.length !== n) s.tones = new Float32Array(n);
  const tones = s.tones;
  const field = o.field;
  for (let y = 0, i = 0; y < h; y++) {
    const v = (y + 0.5) / h;
    st.y = y;
    for (let x = 0; x < w; x++, i++) {
      st.x = x;
      tones[i] = field((x + 0.5) / w, v, t, st);
    }
  }
  return render_tones(w, h, tones, o.matrix, colors, o.contrast);
}

function draw(s, rgba, w, h, t, palette) {
  const o = s.options;
  const st = s.state;
  const canvas = s.canvas;
  const multi = !!palette;
  const out = new ImageData(w, h);
  const data = out.data;
  const ink = o.ink ?? rgbVar(s.host, "--ink-rgb");
  const paper = multi ? palette[0] : null;
  const inks = typeof o.inks === "function" ? o.inks(s) : o.inks;
  const inkIndex = o.inkIndex;
  for (let y = 0, i = 0; y < h; y++) {
    st.y = y;
    for (let x = 0; x < w; x++, i += 4) {
      const r = rgba[i];
      if (multi) {
        const g = rgba[i + 1];
        const b = rgba[i + 2];
        const isPaper = r === paper[0] && g === paper[1] && b === paper[2];
        if (isPaper && !o.opaque) continue;
        data[i] = r;
        data[i + 1] = g;
        data[i + 2] = b;
        data[i + 3] = 255;
      } else {
        if (r === 0) continue;
        let c = ink;
        if (inkIndex) {
          st.x = x;
          c = inks[inkIndex((x + 0.5) / w, (y + 0.5) / h, t, st)] ?? ink;
        }
        data[i] = c[0];
        data[i + 1] = c[1];
        data[i + 2] = c[2];
        data[i + 3] = c[3] ?? 255;
      }
    }
  }
  canvas.getContext("2d").putImageData(out, 0, 0);
  o.onPaint?.(s);
}


let timer = 0;
let last = 0;
export function schedule() {
  if (timer) return;
  timer = requestAnimationFrame(frame);
}
function frame(now) {
  timer = 0;
  if (document.hidden || !initialized) return;
  const due = now - last >= FRAME_MS;
  let again = false;
  for (const s of visible) {
    const animate = s.options.animated && !reducesMotion();
    if (animate) {
      again = true;
      if (!due) continue;
      s.time = s.options.still + (now - s.start) / 1000;
      s.dirty = true;
    } else if (s.options.intro && !reducesMotion()) {
      const elapsed = (now - s.start) / 1000;
      if (elapsed < s.options.intro + 0.1) {
        again = true;
        if (!due) continue;
        s.time = elapsed;
        s.dirty = true;
      } else if (s.time !== s.options.intro) {
        s.time = s.options.intro;
        s.dirty = true;
      }
    } else if (s.time !== s.options.still) {
      s.time = s.options.still;
      s.dirty = true;
    }
    if (s.dirty) {
      s.dirty = false;
      paint(s);
    }
  }
  if (due) last = now;
  if (again) schedule();
}
document.addEventListener("visibilitychange", schedule);
onMotionChange(() => {
  for (const s of surfaces) s.dirty = true;
  schedule();
});
export function repaintAll() {
  for (const s of surfaces) s.dirty = true;
  schedule();
}

/* Field helpers */
export const clamp = (x, a = 0, b = 1) => (x < a ? a : x > b ? b : x);
export const smooth = (a, b, x) => {
  const t = clamp((x - a) / (b - a));
  return t * t * (3 - 2 * t);
};
export const mix = (a, b, t) => a + (b - a) * t;
export const fract = (x) => x - Math.floor(x);
export const hash = (x, y) => fract(Math.sin(x * 127.1 + y * 311.7) * 43758.5453);
export function noise(x, y) {
  const ix = Math.floor(x);
  const iy = Math.floor(y);
  const fx = x - ix;
  const fy = y - iy;
  const u = fx * fx * (3 - 2 * fx);
  const v = fy * fy * (3 - 2 * fy);
  const a = hash(ix, iy);
  const b = hash(ix + 1, iy);
  const c = hash(ix, iy + 1);
  const d = hash(ix + 1, iy + 1);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
export function fbm(x, y, octaves = 3) {
  let sum = 0;
  let amp = 0.5;
  let norm = 0;
  for (let i = 0; i < octaves; i++) {
    sum += amp * noise(x, y);
    norm += amp;
    x = x * 2.03 + 17.1;
    y = y * 2.03 + 9.2;
    amp *= 0.5;
  }
  return sum / norm;
}

/** Rasterise text into a coverage mask the size of a surface. */
export function textMask(lines, w, h, { weight = 600, scale = 0.42, family = "system-ui", lineHeight = 0.95, stroke = 0 } = {}) {
  const c = document.createElement("canvas");
  c.width = w;
  c.height = h;
  const ctx = c.getContext("2d", { willReadFrequently: true });
  let size = Math.floor(h * scale);
  ctx.font = `${weight} ${size}px ${family}`;
  const widest = Math.max(...lines.map((l) => ctx.measureText(l).width));
  if (widest > w * 0.92) size = Math.floor((size * w * 0.92) / widest);
  ctx.font = `${weight} ${size}px ${family}`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const total = size * lineHeight * lines.length;
  const fill = (style) => {
    ctx.fillStyle = style;
    lines.forEach((line, i) =>
      ctx.fillText(line, w / 2, h / 2 - total / 2 + size * lineHeight * (i + 0.5)),
    );
  };
  fill("#f00");
  if (stroke) {
    ctx.globalCompositeOperation = "source-atop";
    ctx.strokeStyle = "#0f0";
    ctx.lineWidth = stroke;
    lines.forEach((line, i) =>
      ctx.strokeText(line, w / 2, h / 2 - total / 2 + size * lineHeight * (i + 0.5)),
    );
  }
  const data = ctx.getImageData(0, 0, w, h).data;
  const fillMask = new Float32Array(w * h);
  const edgeMask = new Float32Array(w * h);
  for (let i = 0; i < w * h; i++) {
    fillMask[i] = data[i * 4 + 3] / 255;
    edgeMask[i] = data[i * 4 + 1] / 255;
  }
  return { fill: fillMask, edge: edgeMask, w, h };
}
