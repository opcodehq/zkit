// zkit's theme tokens (crates/components/src/theme.rs), projected into CSS
// custom properties. Colours are authored in OKLCH and converted exactly as
// the Rust crate does, so the site matches native surfaces.

const gamma = (x) => {
  x = Math.min(1, Math.max(0, x));
  return x <= 0.0031308 ? 12.92 * x : 1.055 * x ** (1 / 2.4) - 0.055;
};
/** OKLCH (CSS notation) to 0..255 sRGB. */
export function oklch(l, c, h) {
  const r = (h * Math.PI) / 180;
  const a = c * Math.cos(r);
  const b = c * Math.sin(r);
  const l_ = (l + 0.39633778 * a + 0.21580376 * b) ** 3;
  const m_ = (l - 0.105561346 * a - 0.06385417 * b) ** 3;
  const s_ = (l - 0.08948418 * a - 1.2914855 * b) ** 3;
  return [
    4.0767417 * l_ - 3.3077116 * m_ + 0.23096993 * s_,
    -1.268438 * l_ + 2.6097574 * m_ - 0.3413194 * s_,
    -0.0041960863 * l_ - 0.7034186 * m_ + 1.7076147 * s_,
  ].map((v) => Math.round(gamma(v) * 255));
}
const neutral = (l) => oklch(l, 0, 0);
const grey = (v) => [v, v, v];

export const ACCENTS = {
  indigo: { label: "Indigo", dark: [[0.673, 0.182, 276.935], [0.56, 0.24, 277.117]], light: [[0.511, 0.262, 276.966], [0.511, 0.262, 276.966]] },
  blue: { label: "Blue", dark: [[0.7, 0.17, 255], [0.5, 0.2, 255]], light: [[0.47, 0.21, 255], [0.47, 0.21, 255]] },
  cyan: { label: "Cyan", dark: [[0.76, 0.13, 205], [0.49, 0.12, 205]], light: [[0.45, 0.11, 205], [0.45, 0.11, 205]] },
  green: { label: "Green", dark: [[0.75, 0.17, 150], [0.5, 0.15, 150]], light: [[0.46, 0.14, 150], [0.46, 0.14, 150]] },
  amber: { label: "Amber", dark: [[0.8, 0.17, 84], [0.52, 0.14, 84]], light: [[0.48, 0.14, 84], [0.48, 0.14, 84]] },
  orange: { label: "Orange", dark: [[0.75, 0.18, 55], [0.54, 0.19, 55]], light: [[0.5, 0.19, 55], [0.5, 0.19, 55]] },
  pink: { label: "Pink", dark: [[0.72, 0.18, 350], [0.51, 0.2, 350]], light: [[0.48, 0.2, 350], [0.48, 0.2, 350]] },
};

/** The semantic colour roles of `Theme::dark` / `Theme::light`. */
export function colors(appearance, accent) {
  const [p, s] = ACCENTS[accent][appearance];
  const primary = oklch(...p);
  const strong = oklch(...s);
  if (appearance === "dark") {
    const danger = oklch(0.704, 0.191, 22.216);
    return {
      background: grey(0x16), surface: grey(0x11), "surface-raised": grey(0x1b), "surface-raised-hover": grey(0x24),
      "surface-dialog": grey(0x1b), "surface-overlay": grey(0x1b),
      hover: "rgb(255 255 255 / 0.05)", active: "rgb(255 255 255 / 0.08)", selected: "rgb(255 255 255 / 0.07)",
      border: "rgb(255 255 255 / 0.07)", "border-strong": "rgb(255 255 255 / 0.10)", "focus-ring": primary,
      text: neutral(0.97), "text-muted": neutral(0.7), "text-faint": neutral(0.58),
      solid: neutral(0.97), "solid-hover": neutral(0.9), "on-solid": neutral(0.269),
      accent: primary, "accent-strong": strong, "accent-wash": [...primary, 0.16], "on-accent": neutral(0.985),
      danger, "danger-strong": oklch(0.51, 0.2, 25), "danger-wash": [...danger, 0.14], "on-danger": neutral(0.985),
      warning: oklch(0.828, 0.189, 84.429), success: oklch(0.765, 0.177, 163.223),
      selection: [...primary, 0.35], scrim: "rgb(0 0 0 / 0.48)", shadow: "rgb(0 0 0 / 0.45)",
      chart: [oklch(0.68, 0.17, 252), oklch(0.63, 0.2, 293), oklch(0.75, 0.18, 152), oklch(0.77, 0.16, 62), oklch(0.71, 0.19, 355)],
    };
  }
  const danger = oklch(0.577, 0.245, 27.325);
  return {
    background: grey(0xff), surface: neutral(0.985), "surface-raised": grey(0xff), "surface-raised-hover": neutral(0.97),
    "surface-dialog": grey(0xff), "surface-overlay": grey(0xff),
    hover: "rgb(0 0 0 / 0.04)", active: "rgb(0 0 0 / 0.07)", selected: "rgb(0 0 0 / 0.055)",
    border: "rgb(0 0 0 / 0.08)", "border-strong": "rgb(0 0 0 / 0.12)", "focus-ring": primary,
    text: neutral(0.269), "text-muted": neutral(0.48), "text-faint": neutral(0.6),
    solid: neutral(0.269), "solid-hover": neutral(0.33), "on-solid": neutral(0.985),
    accent: primary, "accent-strong": strong, "accent-wash": [...primary, 0.1], "on-accent": neutral(0.985),
    danger, "danger-strong": oklch(0.51, 0.2, 25), "danger-wash": [...danger, 0.1], "on-danger": neutral(0.985),
    warning: oklch(0.555, 0.163, 48.998), success: oklch(0.596, 0.145, 163.225),
    selection: [...primary, 0.24], scrim: "rgb(0 0 0 / 0.32)", shadow: "rgb(0 0 0 / 0.14)",
    chart: [oklch(0.58, 0.19, 255), oklch(0.54, 0.22, 293), oklch(0.6, 0.16, 152), oklch(0.66, 0.17, 55), oklch(0.58, 0.21, 355)],
  };
}

const css = (v) => (typeof v === "string" ? v : v.length === 4 ? `rgb(${v[0]} ${v[1]} ${v[2]} / ${v[3]})` : `rgb(${v.join(" ")})`);

export const theme = { appearance: "dark", accent: "indigo" };
const listeners = new Set();
export const onThemeChange = (fn) => listeners.add(fn);

export function applyTheme(next = {}) {
  Object.assign(theme, next);
  const c = colors(theme.appearance, theme.accent);
  const root = document.documentElement;
  for (const [name, value] of Object.entries(c)) {
    if (name === "chart") {
      value.forEach((rgb, i) => {
        root.style.setProperty(`--chart-${i + 1}`, css(rgb));
        root.style.setProperty(`--chart-${i + 1}-rgb`, rgb.join(" "));
      });
    } else root.style.setProperty(`--${name}`, css(value));
  }
  // The dither engine reads paper and ink as RGB triples.
  root.style.setProperty("--ink-rgb", c.accent.join(" "));
  root.style.setProperty("--paper-rgb", c.surface.join(" "));
  root.style.setProperty("--warning-rgb", c.warning.join(" "));
  root.style.setProperty("--background-rgb", c.background.join(" "));
  root.dataset.appearance = theme.appearance;
  root.style.colorScheme = theme.appearance;
  try {
    localStorage.setItem("zkit-theme", JSON.stringify(theme));
  } catch {}
  for (const fn of listeners) fn(theme);
}

export function loadTheme() {
  try {
    const saved = JSON.parse(localStorage.getItem("zkit-theme") ?? "{}");
    if (saved.appearance === "light" || saved.appearance === "dark") theme.appearance = saved.appearance;
    if (ACCENTS[saved.accent]) theme.accent = saved.accent;
  } catch {}
  applyTheme();
}
