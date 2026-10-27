// Chrome shared by both pages: header, search, accent and appearance, footer.
import { ACCENTS, theme, applyTheme, loadTheme } from "./theme.js";
import { repaintAll } from "./engine.js";
import { menu, kbd, hydrate } from "./components.js";
import { icon } from "./icons.js";

export const GITHUB = "https://github.com/opcodehq/zkit";

export { icon };

// A 4 × 4 ordered-dither ramp: the zkit mark.
export const LOGO = `<svg class="logo-mark" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><rect width="4" height="4"/><rect x="8" width="4" height="4"/><rect x="12" y="4" width="4" height="4"/><rect x="4" y="4" width="4" height="4" opacity=".5"/><rect x="8" y="8" width="4" height="4"/><rect x="12" y="8" width="4" height="4"/><rect y="8" width="4" height="4" opacity=".5"/><rect x="4" y="12" width="4" height="4"/><rect x="8" y="12" width="4" height="4"/><rect x="12" y="12" width="4" height="4"/></svg>`;

/** Every docs page, in reading order. The docs renderer supplies content. */
export const ROUTES = [
  ["Get started", [
    ["introduction", "Introduction"],
    ["installation", "Installation"],
    ["theming", "Theming"],
    ["accessibility", "Accessibility"],
    ["credits", "Credits"],
  ]],
  ["Components", [
    ["button", "Button"],
    ["kbd", "Kbd"],
    ["badge", "Badge"],
    ["divider", "Divider"],
    ["toolbar", "Toolbar"],
    ["panel", "Panel (frame)"],
    ["sidebar", "Sidebar"],
    ["card", "Card"],
    ["popover", "Popover"],
    ["menu", "Menu"],
    ["dialog", "Dialog"],
    ["progress-bar", "Progress bar"],
    ["spinner", "Spinner"],
    ["pulse-loader", "Pulse loader"],
    ["glass-surface", "Glass surface"],
  ]],
  ["Motion", [
    ["motion", "Motion catalog"],
    ["entrances", "Entrances"],
    ["loaders", "Loaders"],
    ["reduced-motion", "Reduced motion"],
  ]],
  ["Dither", [
    ["dither", "Dither surface"],
    ["dither-fields", "Fields"],
    ["dither-loader", "Loader"],
    ["dither-globe", "Globe"],
    ["dither-button", "Dither button"],
    ["dither-chart", "Dither chart"],
    ["dither-avatar", "Avatar"],
    ["dither-image", "Dither image"],
    ["dither-states", "Cards and states"],
    ["dither-examples", "Examples"],
    ["dither-engine", "Engine"],
  ]],
];
export const NAV = [
  ["Docs", "introduction", "Get started"],
  ["Components", "button", "Components"],
  ["Motion", "motion", "Motion"],
  ["Dither", "dither", "Dither"],
];

export const $ = (s, root = document) => root.querySelector(s);
export const $$ = (s, root = document) => [...root.querySelectorAll(s)];

let extraSearch = () => [];
export const setSearchExtras = (fn) => (extraSearch = fn);

export function header() {
  loadTheme();
  const el = document.createElement("header");
  el.className = "site-header";
  el.innerHTML = `<div class="header-inner">
    <button class="icon-button menu-button" type="button" aria-label="Open menu" aria-expanded="false">${icon.menu}</button>
    <a class="wordmark" href="/" aria-label="zkit home">${LOGO}zkit</a>
    <nav class="main-nav" aria-label="Main">${NAV.map(([label, route, group]) => `<a href="/docs.html#/${route}" data-group="${group}">${label}</a>`).join("")}</nav>
    <div class="header-actions">
      <button class="search-trigger" type="button" aria-label="Search docs">${icon.search}<span>Search docs…</span>${kbd("secondary-k")}</button>
      <div class="z-popover-anchor" data-popover id="accent-picker">
        <button class="icon-button" type="button" aria-label="Accent colour"><span class="accent-dot"></span></button>
        <div class="z-floating z-card" role="menu" aria-label="Accent" style="left:auto;right:0">${menu([{ heading: "Accent" }, ...Object.entries(ACCENTS).map(([id, a]) => ({ label: a.label, value: id, group: "accent", checked: theme.accent === id }))])}</div>
      </div>
      <button class="icon-button theme-toggle" type="button"></button>
      <a class="icon-button" href="${GITHUB}" target="_blank" rel="noreferrer" aria-label="zkit on GitHub">${icon.github}</a>
    </div>
  </div>
  <nav class="mobile-menu z-card" aria-label="Menu" hidden>${ROUTES.map(([group, items]) => `<div class="z-menu-heading">${group}</div>${items.map(([r, t]) => `<a href="/docs.html#/${r}">${t}</a>`).join("")}`).join("")}</nav>`;
  document.body.prepend(el);
  hydrate(el);
  $("#accent-picker", el).addEventListener("select", (e) => {
    applyTheme({ accent: e.detail });
    repaintAll();
  });
  const toggle = $(".theme-toggle", el);
  const sync = () => {
    const dark = theme.appearance === "dark";
    toggle.innerHTML = dark ? icon.sun : icon.moon;
    toggle.setAttribute("aria-label", dark ? "Switch to light appearance" : "Switch to dark appearance");
  };
  toggle.addEventListener("click", () => {
    applyTheme({ appearance: theme.appearance === "dark" ? "light" : "dark" });
    sync();
    repaintAll();
  });
  sync();
  $(".search-trigger", el).addEventListener("click", openSearch);
  const menuButton = $(".menu-button", el);
  const mobile = $(".mobile-menu", el);
  const setMenu = (open) => {
    mobile.hidden = !open;
    menuButton.setAttribute("aria-expanded", String(open));
    menuButton.innerHTML = open ? icon.close : icon.menu;
  };
  menuButton.addEventListener("click", () => setMenu(mobile.hidden));
  mobile.addEventListener("click", (e) => e.target.closest("a") && setMenu(false));
  document.addEventListener("keydown", (e) => e.key === "Escape" && setMenu(false));
  return el;
}

export function setActiveNav(group) {
  for (const a of $$(".main-nav a")) {
    if (a.dataset.group === group) a.setAttribute("aria-current", "page");
    else a.removeAttribute("aria-current");
  }
}

export function footer() {
  const el = document.createElement("footer");
  el.className = "site-footer";
  el.innerHTML = `<div class="footer-inner"><span>zkit: a GPU interface toolkit for native and browser apps.</span><span class="row" style="gap:16px"><a href="/docs.html#/introduction">Docs</a><a href="/docs.html#/button">Components</a><a href="/docs.html#/dither">Dither</a><a href="/docs.html#/credits">Credits</a><a href="${GITHUB}" target="_blank" rel="noreferrer">GitHub</a></span></div>`;
  document.body.append(el);
}

/* Search */
let dialog;
export function openSearch() {
  if (!dialog) {
    dialog = document.createElement("dialog");
    dialog.className = "search";
    dialog.setAttribute("aria-label", "Search");
    dialog.innerHTML = `<div class="search-input">${icon.search}<input type="search" placeholder="Search components, motion and fields…" aria-label="Search" /></div><div class="search-results" role="listbox"></div>`;
    document.body.append(dialog);
    const input = $("input", dialog);
    const results = $(".search-results", dialog);
    const render = () => {
      const q = input.value.trim().toLowerCase();
      const rows = [...ROUTES.flatMap(([group, items]) => items.map(([r, t]) => ({ group, label: t, href: `/docs.html#/${r}` }))), ...extraSearch()].filter((r) => !q || `${r.label} ${r.group}`.toLowerCase().includes(q));
      const groups = new Map();
      for (const r of rows) groups.set(r.group, [...(groups.get(r.group) ?? []), r]);
      results.innerHTML = rows.length
        ? [...groups].map(([g, items]) => `<h5>${g}</h5>${items.map((r) => `<a href="${r.href}" role="option">${r.label}</a>`).join("")}`).join("")
        : `<p class="search-empty">No results.</p>`;
      results.querySelector("a")?.classList.add("active");
    };
    input.addEventListener("input", render);
    input.addEventListener("keydown", (e) => {
      const links = $$("a", results);
      const at = links.findIndex((a) => a.classList.contains("active"));
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const next = links[(at + (e.key === "ArrowDown" ? 1 : -1) + links.length) % links.length];
        links.forEach((a) => a.classList.toggle("active", a === next));
        next?.scrollIntoView({ block: "nearest" });
      } else if (e.key === "Enter" && links[at]) {
        e.preventDefault();
        links[at].click();
      }
    });
    results.addEventListener("click", (e) => e.target.closest("a") && dialog.close());
    dialog.addEventListener("click", (e) => e.target === dialog && dialog.close());
  }
  $("input", dialog).value = "";
  $("input", dialog).dispatchEvent(new Event("input"));
  dialog.showModal();
}
document.addEventListener("keydown", (e) => {
  if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
    e.preventDefault();
    openSearch();
  }
});

let toastTimer;
export function toast(message) {
  let el = $("#toast");
  if (!el) {
    el = document.createElement("div");
    el.id = "toast";
    el.setAttribute("role", "status");
    document.body.append(el);
  }
  el.textContent = message;
  el.classList.add("visible");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.classList.remove("visible"), 2200);
}
export async function copyText(text, label = "Copied.") {
  try {
    await navigator.clipboard.writeText(text);
    toast(label);
  } catch {
    toast("Clipboard unavailable. Select the text and copy it instead.");
  }
}
export function download(blob, name) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
