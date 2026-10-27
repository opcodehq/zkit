// Live pieces for the landing page. Each is a field written in JavaScript
// and dithered by the zkit Rust engine.
//
// Credits: several demos adapt examples from dithercn (https://dithercn.dev, https://github.com/opencoredev/dithercn), MIT; notice in crates/dither/LICENSE-MIT; button and avatar styles follow dither-kit by ripgrim (https://tripwire.sh/dither-kit).
import { surface, clamp, smooth, mix, fract, hash, noise, fbm, textMask, convert_image, render_globe, render_loader, loader_grid, render_avatar, avatar_hue, COLORS, rgbVar, schedule } from "./engine.js";
import { oklch, theme } from "./theme.js";
import { Matrix4 } from "./matrix.js";
import { landCoverage } from "./world-land.js";

const TAU = Math.PI * 2;
const exp = Math.exp;
const ease = (x) => 1 - (1 - clamp(x)) ** 3;
const sn = (x, y) => noise(x, y) * 2 - 1;

/* Hero: a slow aurora over a planet's horizon, with a few fixed stars. */
const PLANET = { cx: 0.5, cy: 1.3, rx: 0.76, ry: 0.42 };
export function hero(host) {
  return surface(host, {
    cell: 3,
    maxCells: 190_000,
    matrix: "bayer4",
    animated: true,
    still: 14,
    field(u, v, t, s) {
      const e = Math.hypot((u - PLANET.cx) / PLANET.rx, (v - PLANET.cy) / PLANET.ry) - 1;
      const above = Math.max(e, 0);
      const below = Math.max(-e, 0);
      let tone = 0;
      if (e > 0 && above < 0.75) {
        const wx = u * s.aspect;
        const wy = 1 - v;
        const warpX = fbm(wx * 1.3 + t * 0.021, wy * 1.3);
        const warpY = fbm(wx * 1.3 + 5.2, wy * 1.3 + 1.3 - t * 0.017);
        const n = fbm(wx * 1.1 + 1.7 * warpX + t * 0.013, wy * 2.6 + 1.7 * warpY);
        const curtains = fbm(wx * 5 + 2 * warpX - t * 0.03, above * 1.5 - t * 0.01);
        const lit = 0.35 + 0.9 * n * curtains + 0.25 * n;
        tone = exp(-above * 10) * lit + 0.22 * exp(-above * 5.5) * curtains * curtains;
      } else if (e <= 0 && below < 0.2) {
        const n = noise(u * s.aspect * 6 + t * 0.05, v * 4);
        tone = exp(-below * 34) * (0.85 + 0.4 * n);
      }
      if (above > 0.3 && Math.abs(u - 0.5) > 0.22) {
        const star = hash(hash(s.x, s.y) * 97, hash(s.y + 31.7, s.x) * 97);
        if (star > 0.9988) tone = 1;
      }
      return tone;
    },
  });
}

/* Aurora: curtains in four colours over a dark sky. */
export function aurora(host) {
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    animated: true,
    still: 8,
    opaque: true,
    palette: [
      [2, 10, 21],
      [20, 35, 70],
      [74, 66, 141],
      [84, 221, 125],
    ],
    field(u, v, t, s) {
      const x = u * s.aspect;
      const base = 0.66 + 0.1 * Math.sin(u * 3.4 + 0.4 + t * 0.06) + 0.05 * Math.sin(u * 7.3 - t * 0.04) - 0.06 * (u - 0.5) + 0.08 * (fbm(x * 1.1 + t * 0.03, 3.1) - 0.5);
      const d = base - v;
      const sky = 0.2 + 0.1 * (1 - v);
      if (d < 0) return sky * exp(d * 7) * 0.9;
      const fold = fbm(x * 1.6 + t * 0.04, 7.3);
      const rays = fbm(x * 10 + fold * 3 - t * 0.05, d * 1.1 - t * 0.04);
      const curtain = exp(-d * 2.8) * (0.25 + 1.25 * rays * rays);
      const edge = exp(-d * 30) * 0.3;
      return Math.max(sky, clamp(curtain * 0.95 + edge));
    },
  });
}

/* Gauge: a half-ring of ticks that fills to a value. */
export function gauge(host, value = 0.68) {
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    intro: 1.4,
    still: 1.4,
    inks: [null, [52, 54, 64]],
    field(u, v, t, s) {
      const X = u * s.w;
      const Y = v * s.h;
      const cx = s.w / 2;
      const cy = s.h * 0.86;
      const outer = s.w * 0.47;
      const inner = outer * 0.74;
      const r = Math.hypot(X - cx, Y - cy);
      if (r < inner || r > outer || Y > cy + 1) return 0;
      const frac = 1 - Math.atan2(cy - Y, X - cx) / Math.PI;
      if (fract(frac * 36) > 0.72) return 0;
      const filled = frac <= value * ease(t / 1.2);
      const radial = (r - inner) / (outer - inner);
      return filled ? 0.45 + 0.4 * radial + 0.25 * frac : 0.75;
    },
    inkIndex(u, v, t, s) {
      const X = u * s.w;
      const Y = v * s.h;
      const frac = 1 - Math.atan2(s.h * 0.86 - Y, X - s.w / 2) / Math.PI;
      return frac <= value * ease(t / 1.2) ? 0 : 1;
    },
  });
}

/* Stat card: a trend line with a fading fill and a faint dot grid. */
const TREND = [0.32, 0.35, 0.33, 0.38, 0.36, 0.41, 0.4, 0.45, 0.43, 0.47, 0.5, 0.48, 0.53, 0.56, 0.54, 0.6, 0.62, 0.6, 0.66, 0.7, 0.68, 0.75, 0.8, 0.86];
const sampleTrend = (u, series = TREND) => {
  const x = clamp(u) * (series.length - 1);
  const i = Math.min(series.length - 2, Math.floor(x));
  const f = x - i;
  return mix(series[i], series[i + 1], f * f * (3 - 2 * f));
};
export function statCard(host) {
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    intro: 1.2,
    still: 1.2,
    field(u, v, t, s) {
      const grid = 0.3 * smooth(0.45, 1, u) * smooth(0.55, 0, v);
      if (u > ease(t / 1.1)) return grid;
      const top = 0.98 - 0.32 * sampleTrend(u);
      const dy = (v - top) * s.h;
      if (Math.abs(dy) < 0.8) return 1;
      if (dy > 0) return Math.max(grid, 0.5 * exp(-dy / 7));
      return grid;
    },
  });
}

/* Bar chart: weekly bars, densest at the top, the latest one solid. */
export const BARS = [38, 44, 41, 52, 47, 58, 55, 63, 49, 61, 68, 74];
export function barChart(host, max = 80) {
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    intro: 1,
    still: 1,
    field(u, v, t) {
      const slot = u * BARS.length;
      const i = Math.floor(slot);
      const local = slot - i;
      if (local < 0.15 || local > 0.85) return 0;
      const top = 1 - (BARS[i] / max) * ease(t / 0.9 - i * 0.03);
      if (v < top) return 0;
      if (i === BARS.length - 1) return 1;
      return 0.95 - 0.7 * ((v - top) / (1 - top));
    },
  });
}

/* Dither text: solid outlines over a half-tone fill that shimmers. */
export function ditherText(host, lines) {
  let mask = null;
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    animated: true,
    still: 2,
    field(u, v, t, s) {
      if (!mask || mask.w !== s.w || mask.h !== s.h)
        mask = textMask(lines, s.w, s.h, { weight: 700, scale: 0.34, stroke: 2.4, lineHeight: 0.92 });
      const i = s.y * s.w + s.x;
      if (mask.fill[i] < 0.5) return 0;
      if (mask.edge[i] > 0.5) return 1;
      return 0.4 + 0.3 * fbm(u * 5 - t * 0.25, v * 3 + t * 0.1);
    },
  });
}

/* Globe: land from a mask, a lit terminator and two dotted orbits. */
export function globe(host) {
  const tilt = 0.32;
  const ct = Math.cos(tilt);
  const st = Math.sin(tilt);
  const L = [-0.55, -0.45, 0.7];
  const orbit = (X, Y, cx, cy, a, b, rot) => {
    const c = Math.cos(rot);
    const n = Math.sin(rot);
    const x = (X - cx) * c + (Y - cy) * n;
    const y = -(X - cx) * n + (Y - cy) * c;
    const q = Math.sqrt((x / a) ** 2 + (y / b) ** 2);
    return { near: Math.abs(q - 1) * b < 0.7, back: y < 0, phi: Math.atan2(y / b, x / a) };
  };
  const sample = (u, v, t, s) => {
    const X = u * s.w;
    const Y = v * s.h;
    const R = Math.min(s.w, s.h) * 0.47;
    const cx = s.w * 0.5;
    const cy = s.h * 0.56;
    const dx = (X - cx) / R;
    const dy = (Y - cy) / R;
    const r2 = dx * dx + dy * dy;
    for (const [a, b, rot, speed] of [
      [1.32, 0.42, -0.42, 0.4],
      [1.18, 0.3, 0.55, -0.3],
    ]) {
      const o = orbit(X, Y, cx, cy, a * R, b * R, rot);
      if (o.near && !(o.back && r2 < 1) && fract(o.phi * 9 + t * speed * 0.3) < 0.45) return [1, 1];
    }
    if (r2 >= 1) {
      const r = Math.sqrt(r2);
      return [r < 1.1 ? 0.28 * exp(-(r - 1) * 26) : 0, 0];
    }
    const dz = Math.sqrt(1 - r2);
    const y2 = dy * ct - dz * st;
    const z2 = dy * st + dz * ct;
    const lat = (-Math.asin(clamp(y2, -1, 1)) * 180) / Math.PI;
    let lon = (Math.atan2(dx, z2) * 180) / Math.PI + t * 6 - 20;
    const light = clamp(-dx * L[0] - dy * L[1] + dz * L[2], 0, 1);
    const land = landCoverage(lat, lon) >= 0.5;
    return [land ? 0.3 + 0.7 * light : 0.05 + 0.1 * light, 0];
  };
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    animated: true,
    still: 4,
    inks: [null, COLORS.orange],
    field: (u, v, t, s) => sample(u, v, t, s)[0],
    inkIndex: (u, v, t, s) => sample(u, v, t, s)[1],
  });
}

/* Donut: segments that sweep in, each in its own colour. */
export const DONUT = [
  ["North America", 48210, COLORS.blue],
  ["Europe", 31870, COLORS.orange],
  ["Asia Pacific", 17430, COLORS.green],
  ["South America", 6340, COLORS.pink],
  ["Other", 2150, COLORS.grey],
];
export function donut(host) {
  const total = DONUT.reduce((a, d) => a + d[1], 0);
  const stops = [];
  let acc = 0;
  for (const d of DONUT) {
    stops.push([acc / total, (acc += d[1]) / total]);
  }
  const segment = (u, v) => {
    const frac = fract(Math.atan2(u - 0.5, -(v - 0.5)) / TAU + 1);
    return [stops.findIndex(([a, b]) => frac >= a && frac < b), frac];
  };
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    intro: 1.3,
    still: 1.3,
    inks: DONUT.map((d) => d[2]),
    field(u, v, t) {
      const r = Math.hypot(u - 0.5, v - 0.5) * 2;
      if (r > 0.98 || r < 0.66) return 0;
      const [i, frac] = segment(u, v);
      if (i < 0 || frac > ease(t / 1.2)) return 0;
      const [a, b] = stops[i];
      if (frac - a < 0.006 || b - frac < 0.006) return 0;
      const radial = (r - 0.66) / 0.32;
      return 0.5 + 0.5 * Math.abs(radial - 0.5) * 2 + 0.12 * (noise(u * 30, v * 30) - 0.5);
    },
    inkIndex: (u, v) => Math.max(0, segment(u, v)[0]),
  });
}

/* Avatar: a mirrored 5 × 5 mark generated from a name. */
const nameHash = (name) => {
  let h = 2166136261;
  for (const ch of name) h = Math.imul(h ^ ch.charCodeAt(0), 16777619);
  return h >>> 0;
};
export const AVATAR_COLORS = [COLORS.pink, COLORS.purple, COLORS.blue, COLORS.green, COLORS.orange, COLORS.red];
export function avatar(host, name, { cell = 3 } = {}) {
  const h = nameHash(name);
  const bits = [];
  for (let i = 0; i < 15; i++) bits.push((h >>> i) & 1);
  const color = AVATAR_COLORS[(h >>> 17) % AVATAR_COLORS.length];
  return surface(host, {
    cell,
    matrix: "bayer4",
    ink: color,
    field(u, v) {
      const r = Math.hypot(u - 0.5, v - 0.5);
      if (r > 0.5) return 0;
      const bg = 0.1 + 0.18 * (1 - r * 2);
      const gx = Math.floor((u - 0.2) / 0.12);
      const gy = Math.floor((v - 0.2) / 0.12);
      if (gx < 0 || gx > 4 || gy < 0 || gy > 4) return bg;
      const col = gx < 3 ? gx : 4 - gx;
      return bits[gy * 3 + col] ? 0.95 - 0.25 * v : bg;
    },
  });
}

/* Dither image: a photo through a four-step blue palette. */
export async function ditherImage(host, src) {
  const img = new Image();
  img.src = src;
  await img.decode();
  const canvas = document.createElement("canvas");
  canvas.className = "dither";
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", "Buzz Aldrin on the Moon, dithered in four shades of blue");
  host.append(canvas);
  const draw = () => {
    const rect = host.getBoundingClientRect();
    const cell = 2;
    const w = Math.max(1, Math.floor(rect.width / cell));
    const h = Math.max(1, Math.floor(rect.height / cell));
    const scratch = document.createElement("canvas");
    scratch.width = w;
    scratch.height = h;
    const ctx = scratch.getContext("2d");
    const scale = Math.max(w / img.width, h / img.height);
    const dw = img.width * scale;
    const dh = img.height * scale;
    ctx.filter = "contrast(1.15)";
    ctx.drawImage(img, (w - dw) / 2, (h - dh) * 0.12, dw, dh);
    const source = ctx.getImageData(0, 0, w, h);
    const palette = new Uint8Array([10, 20, 32, 28, 72, 118, 46, 134, 204, 120, 205, 255]);
    const pixels = convert_image(w, h, source.data, palette, "blue-noise");
    canvas.width = w;
    canvas.height = h;
    canvas.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(pixels), w, h), 0, 0);
  };
  draw();
  new ResizeObserver(draw).observe(host);
}

/* Waves: thin bright swells with a soft shade beneath each. */
export function waves(host) {
  return surface(host, {
    cell: 3,
    matrix: "bayer4",
    animated: true,
    still: 3,
    field(u, v, t, s) {
      if (v < 0.12) return 0;
      if (v < 0.2) return 0.22 + 0.1 * noise(u * 20 + t * 0.2, v * 10);
      let tone = 0.08 * fbm(u * 6 + t * 0.05, v * 3);
      for (let k = 0; k < 7; k++) {
        const y = 0.24 + k * 0.11 + 0.025 * Math.sin(u * 4.5 + k * 1.3 + t * 0.35) + 0.015 * Math.sin(u * 11 - t * 0.25 + k);
        const dy = (v - y) * s.h;
        if (Math.abs(dy) < 0.75) return 1 - k * 0.04;
        if (dy > 0) tone = Math.max(tone, 0.24 * exp(-dy / 5));
      }
      return tone;
    },
  });
}

/* Heatmap: 26 weeks of activity, one square per day. */
export const HEAT = (() => {
  const cells = [];
  for (let c = 0; c < 26; c++)
    for (let r = 0; r < 7; r++) {
      const busy = r > 0 && r < 6 ? 0.25 : -0.2;
      const x = hash(c * 3.1, r * 7.7) + busy + c * 0.012;
      cells.push(x < 0.35 ? 0 : x < 0.55 ? 1 : x < 0.75 ? 2 : x < 0.95 ? 3 : 4);
    }
  return cells;
})();
export function heatmap(host) {
  const at = (u, v) => HEAT[Math.min(25, Math.floor(u * 26)) * 7 + Math.min(6, Math.floor(v * 7))];
  return surface(host, {
    cell: 2,
    matrix: "bayer4",
    inks: [COLORS.green, [44, 44, 48]],
    field(u, v) {
      if (fract(u * 26) > 0.78 || fract(v * 7) > 0.78) return 0;
      const level = at(u, v);
      return level === 0 ? 1 : [0, 0.3, 0.55, 0.78, 1][level];
    },
    inkIndex: (u, v) => (at(u, v) === 0 ? 1 : 0),
  });
}

/* Progress: a fill with a moving sheen and a soft leading edge. */
export function progress(host, value = 0.52) {
  return surface(host, {
    cell: 2,
    matrix: "bayer4",
    animated: true,
    still: 0,
    field(u, v, t) {
      if (u > value) return 0.6 * exp(-(u - value) * 60);
      const sheen = 0.5 + 0.5 * Math.sin((u * 26 - v * 3 - t * 1.6) * 1.2);
      return 0.6 + 0.35 * sheen * smooth(0, 0.25, u);
    },
  });
}

/* Dither button fills. */
export function buttonFill(host, kind) {
  const s = surface(host, {
    cell: 2,
    matrix: "bayer4",
    field(u, v) {
      const hover = s.hover ? 0.1 : 0;
      // Keep the label's band clear; texture lives at the edges.
      const edge = smooth(0.62, 1, v);
      return kind === "primary" ? 1 - 0.55 * edge + hover : 0.3 * edge + hover;
    },
  });
  const button = host.parentElement;
  const set = (on) => {
    s.hover = on;
    s.set({});
  };
  button.addEventListener("pointerenter", () => set(true));
  button.addEventListener("pointerleave", () => set(false));
  return s;
}

/* Showcase fields: one sentence to the agent, one field in return. */
const clear = (u, v, s) => {
  const ex = Math.min(130, s.w * 0.46);
  const dx = ((u - 0.5) * s.w) / ex;
  const dy = ((v - 0.4) * s.h) / 30;
  return smooth(0, 0.7, Math.hypot(dx, dy) - 1);
};
const topoHeight = (x, y, t) => 0.5 + 0.32 * sn(x * 0.85 + t * 0.025, y * 0.85 + t * 0.012) + 0.14 * sn(x * 1.9 - t * 0.035 + 3.1, y * 1.9 - t * 0.017 + 3.1);
export const SHOWCASE = [
  {
    id: "contours",
    label: "Contours",
    prompt: "A dithered topographic background behind the hero. Slow, and quiet enough to read over.",
    scene: `<div><h3>Field notes for the trail</h3><p>Maps, routes and the long way round.</p></div>`,
    matrix: "bayer4",
    still: 8,
    field(u, v, t, s) {
      const x = u * s.aspect * 1.6 * 2.2;
      const y = (1 - v) * 1.6 * 2.2;
      const e = (1.6 * 2.2) / s.h;
      const h = topoHeight(x, y, t);
      const hx = topoHeight(x + e, y, t);
      const hy = topoHeight(x, y + e, t);
      const value = h * 12;
      const perCell = Math.max(Math.hypot(hx - h, hy - h) * 12, 1e-4);
      const d = Math.abs(fract(value + 0.5) - 0.5) / perCell;
      const line = 1 - smooth(0.3, 1.1, d);
      const major = ((Math.floor(value + 0.5) % 4) + 4) % 4 === 0 ? 1 : 0.5;
      return line * major * clear(u, v, s);
    },
  },
  {
    id: "meter",
    label: "Usage meter",
    prompt: "A radial usage meter for the billing card. 72% used, with the number in the middle.",
    scene: `<div><h3 class="meter-number">72%</h3><p>of storage used</p></div>`,
    matrix: "bayer8",
    still: 3,
    field(u, v, t, s) {
      const cx = (u - 0.5) * s.aspect;
      const cy = 0.4 - v;
      const r = Math.hypot(cx, cy);
      const angle = fract(Math.atan2(cx, cy) / TAU + 1);
      const cellSize = 1 / s.h;
      const radius = 0.27;
      const band = 1 - smooth(0, 1.5 * cellSize, Math.abs(r - radius) - 0.045);
      if (band <= 0) return 0;
      const value = 0.72 * smooth(0, 1, Math.min(t * 0.6, 1));
      const filled = angle <= value;
      const along = angle / Math.max(value, 0.001);
      const inner = smooth(radius - 0.05, radius + 0.05, r);
      const arc = mix(0.45, 1, along * along) * mix(0.75, 1, inner);
      return band * (filled ? arc : 0.12);
    },
  },
  {
    id: "ripples",
    label: "Ripples",
    prompt: "Ripples spreading out from behind the launch headline, like a drop in still water.",
    scene: `<div><h3>Launch day</h3><p>Everything we shipped this week.</p></div>`,
    matrix: "bayer4",
    still: 5,
    field(u, v, t, s) {
      const px = u * s.aspect;
      const py = 1 - v;
      const wobble = fbm(px * 2.2 + t * 0.03, py * 2.2) - 0.5;
      const d = Math.hypot(px - 0.5 * s.aspect, py - 0.6) + wobble * 0.08;
      const rings = 0.5 + 0.5 * Math.cos(d * 38 - t * 0.9);
      return 0.9 * rings * exp(-d * 1.6) * clear(u, v, s);
    },
  },
];
export function showcase(host, item) {
  return surface(host, {
    cell: 3,
    maxCells: 150_000,
    animated: true,
    matrix: item.matrix,
    still: item.still,
    field: item.field,
  });
}

export { rgbVar };

/* Globe and loader: rendered by the Rust engine (Globe, LoaderVariant). */
export const OFFICES = [
  [37.77, -122.42], [40.71, -74.01], [-23.55, -46.63], [51.51, -0.13],
  [6.52, 3.38], [1.35, 103.82], [35.68, 139.69], [-33.87, 151.21],
];
export const ROUTES = [
  [37.77, -122.42, 51.51, -0.13], [40.71, -74.01, -23.55, -46.63], [51.51, -0.13, 1.35, 103.82],
  [1.35, 103.82, -33.87, 151.21], [35.68, 139.69, 37.77, -122.42], [51.51, -0.13, 6.52, 3.38],
];
export function rustGlobe(host, { arcs = true, markers = true, cell = 2, speed = 6, tilt = 18 } = {}) {
  const m = new Float32Array(markers ? OFFICES.flat() : []);
  const a = new Float32Array(arcs ? ROUTES.flat() : []);
  return surface(host, {
    cell,
    animated: true,
    still: 0,
    maxCells: 90_000,
    raster: (w, h, t, s) =>
      render_globe(w, h, t, speed, tilt, new Uint8Array(rgbVar(s.host, "--ink-rgb")), new Uint8Array(rgbVar(s.host, "--warning-rgb")), m, a),
  });
}
export const LOADER_SIZES = { sm: [24, 1], md: [40, 2], lg: [64, 2] };
export function rustLoader(host, variant = "ring", size = "md") {
  const [height, cell] = LOADER_SIZES[size];
  const [cols, rows] = loader_grid(variant, Math.round(height / cell));
  host.style.width = `${cols * cell}px`;
  host.style.height = `${rows * cell}px`;
  host.style.position = "relative";
  host.setAttribute("role", "progressbar");
  host.setAttribute("aria-busy", "true");
  host.setAttribute("aria-label", "Loading");
  const still = { ring: 0.4, dots: 0.2, pulse: 0.5, bar: 0.9 }[variant];
  return surface(host, {
    cell,
    animated: true,
    still,
    raster: (w, h, t, s) => render_loader(variant, w, h, t, new Uint8Array(rgbVar(s.host, "--ink-rgb"))),
  });
}

/* Avatar: `zkit_dither::Avatar`, an 8 × 8 symmetric pattern and a hue from the name. */
export const avatarHueOf = (name) => avatar_hue(name);
export const avatarColor = (name, hue = avatar_hue(name)) => oklch(theme.appearance === "dark" ? 0.74 : 0.56, 0.17, hue);
export function rustAvatar(host, name, size = 40, hue) {
  const scale = Math.max(1, Math.round(size / 16));
  host.style.width = host.style.height = `${size}px`;
  host.style.position = "relative";
  host.setAttribute("role", "img");
  host.setAttribute("aria-label", `Avatar for ${name}`);
  return surface(host, {
    cell: size / (8 * scale),
    raster: () => render_avatar(name, scale, new Uint8Array(avatarColor(name, hue))),
  });
}

/* Dither button faces, mirroring `DitherButton` in Rust. */
export function buttonFace(host, { fill = "gradient", color = null } = {}) {
  let boost = 0;
  const s = surface(host, {
    cell: 2,
    raster(w, h, t, st) {
      const ink = color ?? rgbVar(st.host, "--ink-rgb");
      const out = new Uint8Array(w * h * 4);
      for (let y = 0; y < h; y++) {
        const depth = Math.min(1, 0.15 + boost + (0.85 * (y + 0.5)) / h);
        for (let x = 0; x < w; x++) {
          const threshold = Matrix4(x, y);
          const on =
            fill === "hatched" ? (x + y) % 4 < 2 || depth > 0.95
            : fill === "dotted" ? 0.3 + boost > threshold
            : fill === "solid" ? true
            : 0.22 + 0.78 * depth ** 0.85 > threshold;
          if (!on) continue;
          const i = (y * w + x) * 4;
          out[i] = ink[0];
          out[i + 1] = ink[1];
          out[i + 2] = ink[2];
          out[i + 3] = 217;
        }
      }
      return out;
    },
  });
  const button = host.closest("button");
  const set = (b) => {
    boost = button.disabled ? 0 : b;
    s.dirty = true;
    schedule();
  };
  button.addEventListener("pointerenter", () => set(0.3));
  button.addEventListener("pointerleave", () => set(0));
  button.addEventListener("pointerdown", () => set(0.55));
  button.addEventListener("pointerup", () => set(0.3));
  return s;
}
