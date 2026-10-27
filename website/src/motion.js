// zkit's motion vocabulary (crates/components/src/motion.rs and pulse.rs),
// ported so the docs animate with the same curves, durations and loader
// math as native surfaces.

export class CubicBezier {
  constructor(x1, y1, x2, y2) {
    Object.assign(this, { x1, y1, x2, y2 });
  }
  static coefficients(a, b) {
    const c = 3 * a;
    const bb = 3 * (b - a) - c;
    return [1 - c - bb, bb, c];
  }
  sampleX(t) {
    const [a, b, c] = CubicBezier.coefficients(this.x1, this.x2);
    return ((a * t + b) * t + c) * t;
  }
  sampleY(t) {
    const [a, b, c] = CubicBezier.coefficients(this.y1, this.y2);
    return ((a * t + b) * t + c) * t;
  }
  derivativeX(t) {
    const [a, b, c] = CubicBezier.coefficients(this.x1, this.x2);
    return (3 * a * t + 2 * b) * t + c;
  }
  solve(x) {
    let t = x;
    for (let i = 0; i < 8; i++) {
      const err = this.sampleX(t) - x;
      if (Math.abs(err) < 1e-6) return t;
      const d = this.derivativeX(t);
      if (Math.abs(d) < 1e-6) break;
      t -= err / d;
    }
    let lo = 0;
    let hi = 1;
    for (let i = 0; i < 32; i++) {
      const mid = (lo + hi) / 2;
      if (this.sampleX(mid) < x) lo = mid;
      else hi = mid;
    }
    return (lo + hi) / 2;
  }
  eval(x) {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    return Math.min(1, Math.max(0, this.sampleY(this.solve(x))));
  }
  get css() {
    return `cubic-bezier(${this.x1}, ${this.y1}, ${this.x2}, ${this.y2})`;
  }
}

export const CURVES = {
  EASE_OUT_EXPO: new CubicBezier(0.16, 1, 0.3, 1),
  EASE_OUT: new CubicBezier(0, 0, 0.58, 1),
  EASE: new CubicBezier(0.25, 0.1, 0.25, 1),
  EASE_IN_OUT: new CubicBezier(0.42, 0, 0.58, 1),
  EASE_OUT_QUINT: new CubicBezier(0.22, 1, 0.36, 1),
  EASE_STANDARD: new CubicBezier(0.4, 0, 0.2, 1),
};

export const SPECS = [
  { name: "ENTER", ms: 320, curve: "EASE_OUT_EXPO", use: "Content entrances", demo: "rise" },
  { name: "FADE_QUICK", ms: 150, curve: "EASE", use: "Opacity-only swaps", demo: "fade" },
  { name: "MENU_IN", ms: 140, curve: "EASE", use: "Popovers, menus", demo: "menu" },
  { name: "MENU_OUT", ms: 100, curve: "EASE", use: "Popover exits", demo: "fade-out" },
  { name: "DIALOG_IN", ms: 180, curve: "EASE", use: "Modal dialogs", demo: "dialog" },
  { name: "RESIZE", ms: 200, curve: "EASE_OUT", use: "Sidebar and pane sizes", demo: "slide" },
  { name: "COLLAPSE", ms: 180, curve: "EASE_OUT", use: "Disclosure height", demo: "slide" },
  { name: "HOVER_FADE", ms: 150, curve: "EASE_STANDARD", use: "Colour transitions", demo: "slide" },
];

/* Pulse math (pulse.rs). */
export const PULSE_MS = 2400;
export const SPIN_MS = 750;
export const PULSE_MIN_OPACITY = 0.12;
export const PULSE_MIN_SCALE = 0.9;
export const PULSE_STAGGER = 0.15 / 2.4;
export const SPIN_DIM = 0.15;
const TAU = Math.PI * 2;
const lerp = (a, b, t) => a + (b - a) * t;
const rem = (x) => ((x % 1) + 1) % 1;
export const staggeredPhase = (raw, index, stagger) => rem(raw - index * stagger);
export const pulseWave = (phase) => 0.5 - 0.5 * Math.cos(phase * TAU);
export const pulseOpacity = (phase) => PULSE_MIN_OPACITY + (1 - PULSE_MIN_OPACITY) * pulseWave(phase);
export const pulseScale = (phase) => PULSE_MIN_SCALE + (1 - PULSE_MIN_SCALE) * pulseWave(phase);
export function spinOpacity(t, dim) {
  t = rem(t);
  if (t < 0.45) return lerp(1, dim, t / 0.45);
  if (t < 0.92) return dim;
  return lerp(dim, 1, (t - 0.92) / 0.08);
}
export function spinCellPhase(row, col, side) {
  const last = side - 1;
  const centre = last / 2;
  const max = last + centre;
  const d = last - row + Math.abs(col - centre);
  return max === 0 ? 0 : d / (max + 1);
}

/* Reduced-motion policy (MotionPolicy). */
const systemQuery = matchMedia("(prefers-reduced-motion: reduce)");
export const policy = { preference: "system" };
const listeners = new Set();
export const onMotionChange = (fn) => listeners.add(fn);
/** Whether motion is reduced: the user's choice, else the system setting. */
export const reducesMotion = () =>
  policy.preference === "reduced" || (policy.preference === "system" && systemQuery.matches);
function sync() {
  document.documentElement.dataset.motion = reducesMotion() ? "reduced" : "full";
  for (const fn of listeners) fn();
}
export function setMotionPreference(preference) {
  policy.preference = preference;
  try {
    localStorage.setItem("zkit-motion", preference);
  } catch {}
  sync();
}
try {
  const saved = localStorage.getItem("zkit-motion");
  if (["system", "reduced", "full"].includes(saved)) policy.preference = saved;
} catch {}
systemQuery.addEventListener("change", sync);
sync();

/** A shared clock for loaders. It parks when nothing visible asks for frames. */
const tickers = new Set();
let raf = 0;
function loop(now) {
  raf = 0;
  for (const t of tickers) {
    if (t.el.isConnected) t.fn(now);
    else tickers.delete(t);
  }
  if (tickers.size && !document.hidden) raf = requestAnimationFrame(loop);
}
const seen = new IntersectionObserver((entries) => {
  for (const e of entries) {
    const t = e.target.__ticker;
    if (!t) continue;
    if (e.isIntersecting) tickers.add(t);
    else tickers.delete(t);
  }
  if (tickers.size && !raf) raf = requestAnimationFrame(loop);
});
document.addEventListener("visibilitychange", () => {
  if (!document.hidden && tickers.size && !raf) raf = requestAnimationFrame(loop);
});
export function tick(el, fn) {
  el.__ticker = { el, fn };
  seen.observe(el);
  fn(performance.now());
}
