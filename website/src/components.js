// Web renderings of zkit-components for the docs. Markup follows the Rust
// render functions (button.rs, layout.rs, surface.rs, progress.rs), and the
// behaviours follow docs/components.md: keyboard activation, menus with
// arrow keys, dialogs that trap focus and restore it on close.
import { icon } from "./icons.js";
import { CURVES, PULSE_MS, PULSE_STAGGER, policy, reducesMotion, pulseOpacity, pulseScale, pulseWave, staggeredPhase, tick } from "./motion.js";

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
export const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

/** `format_keystroke`: gpui notation to platform key caps. */
export function formatKeystroke(keystroke, mac = isMac) {
  return keystroke
    .split(" ")
    .map((chord) => {
      const parts = chord.endsWith("-") ? [...chord.slice(0, -1).split("-").filter(Boolean), "-"] : chord.split("-");
      const key = parts.pop();
      const mods = parts.map((m) => {
        const table = mac
          ? { cmd: "⌘", secondary: "⌘", ctrl: "⌃", alt: "⌥", shift: "⇧", fn: "fn" }
          : { cmd: "Ctrl+", secondary: "Ctrl+", ctrl: "Ctrl+", alt: "Alt+", shift: "Shift+", fn: "Fn+" };
        return table[m] ?? m;
      });
      const named = {
        enter: mac ? "↩" : "Enter",
        escape: "Esc",
        backspace: mac ? "⌫" : "Backspace",
        tab: mac ? "⇥" : "Tab",
        space: "Space",
        up: "↑",
        down: "↓",
        left: "←",
        right: "→",
      };
      const k = named[key] ?? (key.length === 1 ? key.toUpperCase() : key[0].toUpperCase() + key.slice(1));
      return mods.join("") + k;
    })
    .join(" ");
}

export const kbd = (keys) => `<kbd class="z-kbd">${esc(formatKeystroke(keys))}</kbd>`;
export const badge = (label, accent = false) => `<span class="z-badge${accent ? " accent" : ""}">${esc(label)}</span>`;

/** `Button`. Variants: primary, accent, secondary, ghost, danger. */
export function button(label, o = {}) {
  const cls = ["z-button", o.variant ?? "secondary", o.size, o.iconOnly && "icon-only", o.full && "full"].filter(Boolean).join(" ");
  const attrs = [
    `class="${cls}"`,
    'type="button"',
    o.disabled && "disabled",
    o.loading && 'aria-busy="true" aria-disabled="true"',
    o.pressed !== undefined && `aria-pressed="${o.pressed}"`,
    o.expanded !== undefined && `aria-expanded="${o.expanded}"`,
    o.shortcut && `aria-keyshortcuts="${esc(o.shortcut)}"`,
    o.iconOnly && `aria-label="${esc(o.label ?? label)}"`,
    o.attrs,
  ]
    .filter(Boolean)
    .join(" ");
  const leading = o.loading ? spinner({ size: 14, color: "currentColor" }) : (o.leading ?? "");
  return `<button ${attrs}>${leading}${o.iconOnly ? "" : esc(label)}${o.trailing ? `<span aria-hidden="true">${o.trailing}</span>` : ""}${o.shortcut ? kbd(o.shortcut) : ""}</button>`;
}

/** `Spinner`: a faint ring with a quarter arc that turns once every 900 ms. */
export function spinner({ size = 16, color } = {}) {
  const stroke = Math.max(1.5, size / 8);
  const r = size / 2 - stroke / 2;
  const c = 2 * Math.PI * r;
  const ink = color ?? "var(--accent)";
  return `<svg class="z-spinner" data-z="spinner" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" aria-hidden="true"><circle cx="${size / 2}" cy="${size / 2}" r="${r}" fill="none" stroke="${ink}" stroke-opacity="0.2" stroke-width="${stroke}"/><circle class="arc" cx="${size / 2}" cy="${size / 2}" r="${r}" fill="none" stroke="${ink}" stroke-width="${stroke}" stroke-dasharray="${c * 0.25} ${c}" transform="rotate(-90 ${size / 2} ${size / 2})"/></svg>`;
}

/** `PulseLoader`: staggered cells that breathe. */
export function pulseLoader({ cells = 5, cell = 6 } = {}) {
  return `<span class="z-pulse" data-z="pulse" style="gap:${cell / 2}px">${Array.from({ length: cells }, () => `<span style="width:${cell}px;height:${cell}px"><i></i></span>`).join("")}</span>`;
}

/** `ProgressBar`: determinate with a value, or an indeterminate sweep. */
export function progress({ value, label, height = 4 } = {}) {
  const aria = value === undefined ? 'aria-busy="true"' : `aria-valuenow="${value}" aria-valuemin="0" aria-valuemax="100"`;
  const inner = value === undefined ? '<i data-z="sweep"></i>' : `<i style="width:${value}%"></i>`;
  return `<div class="z-progress" role="progressbar" ${aria}${label ? ` aria-label="${esc(label)}"` : ""} style="height:${height}px;border-radius:${height / 2}px">${inner}</div>`;
}

/** `Menu` rows. Items: { label, shortcut, checked, danger, disabled } | "-" | { heading }. */
export function menu(items) {
  return items
    .map((it) => {
      if (it === "-") return '<div class="z-menu-separator" role="separator"></div>';
      if (it.heading) return `<div class="z-menu-heading">${esc(it.heading)}</div>`;
      const role = it.group ? 'role="menuitemradio"' : it.checked !== undefined ? 'role="menuitemcheckbox"' : 'role="menuitem"';
      return `<button class="z-menu-item${it.danger ? " danger" : ""}" ${role}${it.checked !== undefined ? ` aria-checked="${it.checked}"` : ""} tabindex="-1" type="button"${it.disabled ? " disabled" : ""}${it.value ? ` data-value="${esc(it.value)}"` : ""}${it.group ? ` data-group="${esc(it.group)}"` : ""}><span class="check">${it.checked ? icon.check : ""}</span><span class="label">${esc(it.label)}</span>${it.shortcut ? kbd(it.shortcut) : ""}</button>`;
    })
    .join("");
}

/* Behaviours */
const SWEEP_MS = 1400;
function animateLoaders(root) {
  for (const el of root.querySelectorAll('[data-z="spinner"]')) {
    const arc = el.querySelector(".arc");
    const half = el.viewBox.baseVal.width / 2;
    tick(el, (now) => {
      // Reduced motion: the arc holds still; a system-only reduction keeps a slow breath.
      const reduced = reducesMotion();
      const turn = reduced ? 0.125 : (now % 900) / 900;
      const opacity = !reduced ? 1 : policy.preference === "system" ? 0.6 + 0.2 * pulseWave((now % PULSE_MS) / PULSE_MS) : 0.6;
      arc.setAttribute("transform", `rotate(${turn * 360 - 90} ${half} ${half})`);
      el.style.opacity = opacity;
    });
  }
  for (const el of root.querySelectorAll('[data-z="pulse"]')) {
    const dots = [...el.querySelectorAll("i")];
    const size = parseFloat(el.firstElementChild.style.width);
    tick(el, (now) => {
      const reduced = reducesMotion();
      const raw = (now % PULSE_MS) / PULSE_MS;
      dots.forEach((d, i) => {
        const phase = staggeredPhase(raw, i, PULSE_STAGGER);
        const [o, s] = reduced ? [0.5, 1] : [pulseOpacity(phase), pulseScale(phase)];
        d.style.opacity = o;
        d.style.width = d.style.height = `${size * s}px`;
      });
    });
  }
  for (const el of root.querySelectorAll('[data-z="sweep"]')) {
    tick(el, (now) => {
      if (reducesMotion()) {
        el.style.left = "0";
        el.style.width = "100%";
        el.style.opacity = "0.4";
        return;
      }
      el.style.opacity = "1";
      const eased = CURVES.EASE_IN_OUT.eval((now % SWEEP_MS) / SWEEP_MS);
      el.style.left = `${(-0.35 + 1.35 * eased) * 100}%`;
      el.style.width = "35%";
    });
  }
}

/** Popovers and menus: open on press or Enter/Space, Escape and outside presses close, focus returns to the trigger. */
function popovers(root) {
  for (const anchor of root.querySelectorAll("[data-popover]")) {
    const trigger = anchor.querySelector(":scope > button");
    const floating = anchor.querySelector(":scope > .z-floating");
    const isMenu = floating.getAttribute("role") === "menu";
    trigger.setAttribute("aria-haspopup", isMenu ? "menu" : "dialog");
    trigger.setAttribute("aria-expanded", "false");
    floating.hidden = true;
    const items = () => [...floating.querySelectorAll(".z-menu-item:not(:disabled)")];
    const close = (restore = true) => {
      if (floating.hidden) return;
      floating.hidden = true;
      trigger.setAttribute("aria-expanded", "false");
      document.removeEventListener("pointerdown", outside, true);
      if (restore) trigger.focus();
    };
    const outside = (e) => {
      if (!anchor.contains(e.target)) close(false);
    };
    const open = (focusFirst) => {
      floating.hidden = false;
      floating.style.animation = "none";
      void floating.offsetWidth;
      floating.style.animation = "";
      trigger.setAttribute("aria-expanded", "true");
      document.addEventListener("pointerdown", outside, true);
      const first = isMenu ? items()[0] : floating.querySelector("button, input, [tabindex]");
      if (focusFirst || isMenu) first?.focus();
    };
    trigger.addEventListener("click", (e) => (floating.hidden ? open(e.detail === 0) : close()));
    floating.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        close();
        return;
      }
      if (!isMenu) return;
      const list = items();
      const at = list.indexOf(document.activeElement);
      const go = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: list.length - 1 }[e.key];
      if (go !== undefined) {
        e.preventDefault();
        list[(go + list.length) % list.length]?.focus();
      } else if (e.key === "Tab") close(false);
    });
    floating.addEventListener("click", (e) => {
      const item = e.target.closest(".z-menu-item");
      if (!item) return;
      if (item.dataset.group) {
        for (const other of floating.querySelectorAll(`[data-group="${item.dataset.group}"]`)) {
          other.setAttribute("aria-checked", String(other === item));
          other.querySelector(".check").innerHTML = other === item ? icon.check : "";
        }
      }
      if (item.getAttribute("role") === "menuitemcheckbox") {
        const on = item.getAttribute("aria-checked") !== "true";
        item.setAttribute("aria-checked", String(on));
        item.querySelector(".check").innerHTML = on ? icon.check : "";
      }
      anchor.dispatchEvent(new CustomEvent("select", { detail: item.dataset.value ?? item.textContent.trim() }));
      if (item.dataset.dismiss !== "false") close();
    });
  }
}

/** `Dialog`: focus moves in, Tab cycles inside, Escape or backdrop dismisses, focus returns. */
export function openDialog({ host, title, description, body = "", actions, onDismiss, closeButton = true }) {
  const previous = document.activeElement;
  const scrim = document.createElement("div");
  scrim.className = host ? "z-scrim" : "z-scrim fixed";
  scrim.innerHTML = `<div class="z-dialog" role="dialog" aria-modal="true" aria-labelledby="dlg-title"${description ? ' aria-describedby="dlg-desc"' : ""}>
    <div class="z-dialog-header"><h3 id="dlg-title">${esc(title)}</h3>${description ? `<p id="dlg-desc">${esc(description)}</p>` : ""}</div>
    ${body ? `<div class="z-dialog-body">${body}</div>` : ""}
    ${actions ? `<div class="z-dialog-actions">${actions}</div>` : ""}
    ${closeButton ? `<span class="z-dialog-close">${button("", { variant: "ghost", iconOnly: true, label: "Close dialog", leading: icon.close, attrs: 'data-action="close"' })}</span>` : ""}
  </div>`;
  (host ?? document.body).append(scrim);
  const panel = scrim.firstElementChild;
  const focusables = () => [...panel.querySelectorAll("button:not(:disabled), input, select, [tabindex='0']")];
  const close = (reason) => {
    scrim.remove();
    previous?.focus?.();
    onDismiss?.(reason);
  };
  scrim.addEventListener("pointerdown", (e) => e.target === scrim && close("backdrop"));
  panel.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      close("escape");
    } else if (e.key === "Tab") {
      const list = focusables();
      const i = list.indexOf(document.activeElement);
      e.preventDefault();
      list[(i + (e.shiftKey ? -1 : 1) + list.length) % list.length]?.focus();
    }
  });
  panel.addEventListener("click", (e) => {
    const b = e.target.closest("button[data-action]");
    if (b) close(b.dataset.action);
  });
  (panel.querySelector(".z-dialog-body input") ?? focusables()[0])?.focus();
  return close;
}


/** `Card`: header with title, description and action; body; footer. */
export function card({ title, description, action = "", body = "", footer = "", style = "" }) {
  const head = title || description || action;
  return `<section class="z-content-card" role="group"${title ? ` aria-label="${esc(title)}"` : ""} style="${style}">${
    head ? `<div class="z-card-header"${body || footer ? "" : ' style="padding-bottom:24px"'}><div>${title ? `<div class="z-card-title">${esc(title)}</div>` : ""}${description ? `<div class="z-card-description">${esc(description)}</div>` : ""}</div>${action}</div>` : ""
  }${body ? `<div class="z-card-body"${head ? "" : ' style="padding-top:24px"'}>${body}</div>` : ""}${footer ? `<div class="z-card-footer">${footer}</div>` : ""}</section>`;
}

/** A labelled text field in the toolkit's control style. */
let fieldId = 0;
export function field(label, { value = "", placeholder = "" } = {}) {
  const id = `field-${fieldId++}`;
  return `<div class="z-field"><label for="${id}">${esc(label)}</label><input class="z-input" id="${id}" value="${esc(value)}" placeholder="${esc(placeholder)}" /></div>`;
}

/** `Panel`: a frame with a title band, an inset body and a footer. */
export function panel({ title = "", actions = "", body = "", footer = "", bodyStyle = "", style = "" }) {
  return `<section class="z-panel" role="region"${title ? ` aria-label="${esc(title)}"` : ""} style="${style}">${title || actions ? `<div class="z-panel-header"><span>${esc(title)}</span>${actions}</div>` : ""}<div class="z-panel-body" style="${bodyStyle}">${body}</div>${footer ? `<div class="z-panel-footer">${footer}</div>` : ""}</section>`;
}

/** `SidebarSection`: Up/Down/Home/End move within a section; Enter or Space activates. */
function sidebars(root) {
  for (const section of root.querySelectorAll(".z-section[role='list']")) {
    section.addEventListener("keydown", (e) => {
      const items = [...section.querySelectorAll(".z-item:not(:disabled)")];
      const at = items.indexOf(document.activeElement);
      const go = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: items.length - 1 }[e.key];
      if (go === undefined || at < 0) return;
      e.preventDefault();
      items[Math.max(0, Math.min(items.length - 1, go))].focus();
    });
  }
  for (const sidebar of root.querySelectorAll(".z-sidebar")) {
    sidebar.addEventListener("click", (e) => {
      const item = e.target.closest(".z-item");
      if (!item || item.disabled || item.closest(".z-section")?.dataset.static) return;
      for (const other of sidebar.querySelectorAll(".z-item")) other.setAttribute("aria-current", String(other === item));
    });
  }
}

/** Toggle buttons flip aria-pressed; loading buttons stay inert. */
function toggles(root) {
  for (const b of root.querySelectorAll("[data-toggle]")) {
    b.addEventListener("click", () => b.setAttribute("aria-pressed", String(b.getAttribute("aria-pressed") !== "true")));
  }
}

export function hydrate(root = document) {
  animateLoaders(root);
  popovers(root);
  sidebars(root);
  toggles(root);
}
