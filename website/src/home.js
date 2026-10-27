import "./style.css";
import { ready, surface, rgbVar } from "./engine.js";
import { header, footer, icon, $, copyText } from "./shell.js";
import { button, badge, menu, hydrate } from "./components.js";
import { appWindow, wireWindow } from "./app-window.js";
import { CURVES, tick, reducesMotion } from "./motion.js";

const RUN = "cargo run -p zkit-components --example gallery";

function features() {
  return `<section class="features" aria-label="What's in zkit">
    <div class="feature">
      <div class="feature-art"><div class="z-card" style="min-width:180px">${menu([{ label: "New file", shortcut: "secondary-n" }, { label: "Open…", shortcut: "secondary-o" }, "-", { label: "Delete", danger: true }])}</div></div>
      <h3>Components</h3>
      <p>Buttons, menus, popovers, dialogs, sidebars and loaders with keyboard and screen-reader behaviour built in.</p>
      <a href="/docs.html#/button">Browse components ${icon.chevronRight}</a>
    </div>
    <div class="feature">
      <div class="feature-art" id="feature-motion"><svg viewBox="0 0 200 100" style="width:72%;height:70%;overflow:visible" aria-hidden="true"><path d="" fill="none" stroke="var(--border-strong)" stroke-width="1.5" id="feature-curve"/><circle r="5" fill="var(--accent)" id="feature-dot"/></svg></div>
      <h3>Motion</h3>
      <p>One catalog of durations and curves for entrances, menus, dialogs and resizes, with a reduced-motion policy every animation honours.</p>
      <a href="/docs.html#/motion">See the motion catalog ${icon.chevronRight}</a>
    </div>
    <div class="feature">
      <div class="feature-art"><div id="feature-dither"></div></div>
      <h3>Dither</h3>
      <p>A portable Rust engine for dithered surfaces, charts, avatars and images. It draws natively and, through WebAssembly, on this page.</p>
      <a href="/docs.html#/dither">Explore dither ${icon.chevronRight}</a>
    </div>
  </section>`;
}

function animateCurve() {
  const curve = CURVES.EASE_OUT_EXPO;
  const pts = [];
  for (let i = 0; i <= 40; i++) pts.push(`${i ? "L" : "M"}${(i / 40) * 200} ${100 - curve.eval(i / 40) * 100}`);
  $("#feature-curve").setAttribute("d", pts.join(" "));
  const dot = $("#feature-dot");
  tick($("#feature-motion"), (now) => {
    const x = reducesMotion() ? 1 : Math.min(1, ((now % 2200) / 2200) * 1.6);
    dot.setAttribute("cx", x * 200);
    dot.setAttribute("cy", 100 - curve.eval(x) * 100);
  });
}

async function boot() {
  header();
  $("#home").innerHTML = `
    <section class="home-hero">
      <div class="row" style="margin-bottom:18px">${badge("Rust")}${badge("GPUI")}${badge("WebAssembly")}</div>
      <h1>Interface components for native and browser apps.</h1>
      <p>zkit is a GPU interface toolkit in Rust: accessible components, a shared motion system and a dither engine. One codebase for macOS, Linux, Windows and the web.</p>
      <div class="home-actions">
        <a class="z-button primary lg" href="/docs.html#/introduction">Get started</a>
        <a class="z-button secondary lg" href="/docs.html#/button">Browse components</a>
        <span class="command"><span><span style="color:var(--text-faint)">$</span> <b>${RUN}</b></span>${button("", { variant: "ghost", size: "sm", iconOnly: true, label: "Copy command", leading: icon.copy, attrs: 'id="copy-run"' })}</span>
      </div>
    </section>
    <section class="app-window" aria-label="Interactive example">${appWindow()}</section>
    ${features()}`;
  footer();
  $("#copy-run").addEventListener("click", () => copyText(RUN, "Command copied."));
  await ready();
  hydrate($("#home"));
  wireWindow($("#app"));
  surface($("#feature-dither"), {
    preset: "contours",
    cell: 3,
    animated: true,
    palette: (s) => {
      const ink = rgbVar(s.host, "--ink-rgb");
      const paper = rgbVar(s.host, "--paper-rgb");
      return [paper, paper.map((p, i) => Math.round(p + (ink[i] - p) * 0.4)), ink];
    },
  });
  $("#feature-dither").style.cssText = "position:absolute;inset:0";
  animateCurve();
}
boot();
