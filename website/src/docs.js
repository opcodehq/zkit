import "./style.css";
import { ready, surface, rgbVar, export_svg, convert_image, COLORS } from "./engine.js";
import { FIELDS, GROUPS, loadFields, variant } from "./fields.js";
import { header, footer, setActiveNav, setSearchExtras, icon, ROUTES, $, $$, copyText, download, toast } from "./shell.js";
import { button, badge, kbd, menu, progress, spinner, pulseLoader, hydrate, openDialog, formatKeystroke, card, field, panel } from "./components.js";
import { CURVES, SPECS, policy, reducesMotion, setMotionPreference, onMotionChange, tick } from "./motion.js";
import { ACCENTS, theme, colors as themeColors, onThemeChange } from "./theme.js";
import { appWindow, wireWindow } from "./app-window.js";
import * as demo from "./demos.js";
import { chart, FILLS } from "./charts.js";

/* Content helpers */
const esc = (t) => t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const KEYWORDS = /^(use|let|fn|pub|struct|impl|move|if|else|match|mut|for|in|return|true|false|Some|None|Self|self|const|cargo|npm|cd)$/;
function highlight(src) {
  return src.replace(/(\/\/[^\n]*)|("(?:[^"\\]|\\.)*")|(\b\d+(?:\.\d*)?\b)|([A-Za-z_][A-Za-z0-9_]*)|([&<>])/g, (m, comment, str, num, word, ch) => {
    if (comment) return `<span class="tok-c">${esc(comment)}</span>`;
    if (str) return `<span class="tok-s">${esc(str)}</span>`;
    if (num) return `<span class="tok-n">${num}</span>`;
    if (word) return KEYWORDS.test(word) ? `<span class="tok-k">${word}</span>` : word;
    return esc(ch);
  });
}
const code = (src) => `<pre class="code"><code>${highlight(src.trim())}</code>${copyButton()}</pre>`;
const copyButton = () => `<button class="z-button ghost sm icon-only copy-code" type="button" aria-label="Copy code">${icon.copy}</button>`;
const api = (rows) => `<table><thead><tr><th>Method</th><th>Description</th></tr></thead><tbody>${rows.map(([m, d]) => `<tr><td><code>${esc(m)}</code></td><td>${d}</td></tr>`).join("")}</tbody></table>`;
const keys = (rows) => `<table><thead><tr><th>Key</th><th>Action</th></tr></thead><tbody>${rows.map(([k, d]) => `<tr><td>${k.split(" / ").map((x) => kbd(x)).join(" ")}</td><td>${d}</td></tr>`).join("")}</tbody></table>`;

/** A live preview framed with a footer: caption, copy and a code toggle. */
const INFO = icon.info;
function preview({ stage, rust, controls = "", stageClass = "", height, caption = "" }) {
  return `<div class="preview">
    <div class="preview-stage ${stageClass}" data-pane="preview"${height ? ` style="min-height:${height}px"` : ""}>${stage}</div>
    <pre class="code" data-pane="code" hidden><code>${highlight(rust.trim())}</code></pre>
    ${controls ? `<div class="preview-controls">${controls}</div>` : ""}
    <div class="preview-foot"><span class="caption">${caption ? `${INFO}${caption}` : ""}</span>${button("", { variant: "ghost", size: "sm", iconOnly: true, label: "Copy code", leading: icon.copy, attrs: 'data-copy-preview' })}${button("View code", { size: "sm", attrs: 'data-toggle-code aria-pressed="false"' })}</div>
  </div>`;
}
function wirePreviews(root) {
  for (const box of $$(".preview", root)) {
    const toggle = $("[data-toggle-code]", box);
    toggle?.addEventListener("click", () => {
      const showing = toggle.getAttribute("aria-pressed") === "true";
      toggle.setAttribute("aria-pressed", String(!showing));
      toggle.textContent = showing ? "View code" : "View preview";
      $("[data-pane='preview']", box).hidden = !showing;
      $("[data-pane='code']", box).hidden = showing;
    });
    $("[data-copy-preview]", box)?.addEventListener("click", () => copyText($("[data-pane='code'] code", box).textContent, "Code copied."));
  }
  for (const b of $$(".copy-code", root)) b.addEventListener("click", () => copyText(b.parentElement.querySelector("code").textContent, "Code copied."));
}

/* Pages */
const PAGES = {};
const page = (route, def) => (PAGES[route] = { route, ...def });

page("introduction", {
  description: "A GPU interface toolkit for native and browser apps, written in Rust.",
  html: () => `
<p>zkit maintains GPUI's platform, rendering, text and input foundations together with a set of styled, accessible components. Native targets and the browser share the same Rust view code; platform specifics stay behind explicit interfaces.</p>
${preview({ stage: appWindow(), stageClass: "flush", rust: `
Root::new().child(
    h_stack()
        .child(Sidebar::new("nav", "Workspace").child(
            SidebarSection::new("views").title("Views").items([
                SidebarItem::new("overview", "Overview"),
                SidebarItem::new("changes", "Changes").selected(true)
                    .trailing(Badge::new("3").accent()),
            ]),
        ))
        .child(Panel::new("changes").title("Changes").child(
            Button::new("discard", "Discard").danger(),
        )),
)` })}
<h2 id="whats-inside">What's inside</h2>
<table><thead><tr><th>Crate</th><th>Role</th></tr></thead><tbody>
<tr><td><code>zkit-components</code></td><td>Styled presentation layer: theme, motion, buttons, layout, surfaces, progress, glass and dither widgets.</td></tr>
<tr><td><code>gpui-base</code></td><td>Foundation: interaction, focus, accessibility semantics and positioning. Re-exported as <code>zkit_components::base</code>.</td></tr>
<tr><td><code>zkit-dither</code></td><td>Portable dither engine with no GPUI dependency. Compiles to WebAssembly.</td></tr>
<tr><td><code>gpui</code>, <code>gpui_web</code>, <code>gpui_wgpu</code></td><td>UI primitives, the browser backend and portable GPU rendering.</td></tr>
</tbody></table>
<h2 id="about-these-docs">About these docs</h2>
<p>Previews on this site are web renderings of the native components, built from the same theme tokens, metrics, motion curves and loader math, and following the same keyboard behaviour. The dither previews run the real Rust engine compiled to WebAssembly. Run the native gallery to see the GPUI originals:</p>
${code("cargo run -p zkit-components --example gallery")}`,
  mount: (root) => wireWindow($("#app", root)),
});

page("installation", {
  description: "Add the components to a GPUI app and install a theme.",
  html: () => `
<h2 id="dependency">Dependency</h2>
<p><code>zkit-components</code> depends only on <code>gpui</code>, <code>gpui-base</code>, <code>serde</code>, <code>smallvec</code> and <code>web-time</code>. It has no platform crates, so the same view code compiles for native targets and <code>wasm32-unknown-unknown</code>.</p>
${code(`
[dependencies]
zkit-components = { path = "crates/components" }
`)}
<h2 id="setup">Setup</h2>
${code(`
gpui_platform::application().run(|cx| {
    zkit_components::init(cx);               // foundation, key bindings, defaults
    Theme::dark(Accent::Indigo).install(cx); // or Theme::light(..)
    MotionPolicy { preference: saved_preference, ..Default::default() }.install(cx);
    // Platform layer: report the OS setting when known and when it changes.
    zkit_components::motion::set_system_reduces_motion(os_reduces_motion, cx);
    // open windows...
});
`)}
<p>Render <code>Root::new().child(...)</code> at the top of each window. <code>Root</code> paints the base surface, sets the default text style and handles Tab and Shift-Tab so focus stays inside an open dialog.</p>
<h2 id="examples">Examples</h2>
${code(`
cargo run -p zkit-components --example gallery
cargo run -p zkit-components --example dither
cargo run -p zkit-components --example glass
`)}
<p>The gallery also has a <code>wasm32</code> entry point following the GPUI example pattern.</p>`,
});

page("theming", {
  description: "Two designed appearances, seven accents and one set of semantic tokens.",
  html: () => `
<p>Colours are authored on the OKLCH scale and named by purpose, never by component. Light is designed rather than inverted: surfaces flip their order, washes keep their alpha, and accents move down the lightness scale so contrast survives. Contrast pairs are asserted in tests.</p>
${preview({
  stage: `<div class="stack" style="width:100%;gap:16px">
    <div class="row" id="accent-row"></div>
    <div id="token-grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:8px;width:100%"></div></div>`,
  rust: `
Theme::dark(Accent::Indigo).install(cx);
Theme::light(Accent::Green).install(cx);

let colors = &cx.theme().colors;
div().bg(colors.surface).border_color(colors.border).text_color(colors.text_muted)`,
})}
<h2 id="tokens">Colour roles</h2>
<table><thead><tr><th>Group</th><th>Roles</th></tr></thead><tbody>
<tr><td>Surfaces</td><td><code>background</code>, <code>surface</code>, <code>surface_raised</code>, <code>surface_raised_hover</code>, <code>surface_dialog</code>, <code>surface_overlay</code></td></tr>
<tr><td>Washes</td><td><code>hover</code>, <code>active</code>, <code>selected</code></td></tr>
<tr><td>Edges</td><td><code>border</code>, <code>border_strong</code>, <code>focus_ring</code></td></tr>
<tr><td>Text</td><td><code>text</code>, <code>text_muted</code>, <code>text_faint</code></td></tr>
<tr><td>Solid and accent</td><td><code>solid</code>, <code>solid_hover</code>, <code>on_solid</code>, <code>accent</code>, <code>accent_strong</code>, <code>accent_wash</code>, <code>on_accent</code></td></tr>
<tr><td>Status</td><td><code>danger</code>, <code>danger_strong</code>, <code>danger_wash</code>, <code>warning</code>, <code>success</code></td></tr>
</tbody></table>
<h2 id="metrics">Metrics</h2>
<p>32 px controls with 10 px corners; dialogs, cards and frames use 18 px corners. Plates separate with a hairline and a soft resting shadow, and filled buttons carry a one-pixel top highlight.</p>
<table><thead><tr><th>Metric</th><th>Value</th></tr></thead><tbody>
<tr><td>Radius sm / md / lg / xl</td><td>6 / 8 / 10 / 18 px</td></tr>
<tr><td>Control height sm / md / lg</td><td>28 / 32 / 36 px</td></tr>
<tr><td>Text xs / sm / md / lg</td><td>12 / 13 / 14 / 18 px</td></tr>
<tr><td>Sidebar, panel header, toolbar</td><td>240, 44, 44 px</td></tr>
</tbody></table>`,
  mount(root) {
    const row = $("#accent-row", root);
    row.innerHTML = `<span class="muted" style="font-size:var(--text-sm);margin-right:4px">Accent</span>${Object.entries(ACCENTS)
      .map(([id, a]) => `<button class="swatch" type="button" style="--swatch:rgb(${themeColors(theme.appearance, id).accent.join(" ")})" aria-label="${a.label}" aria-pressed="${theme.accent === id}" data-accent="${id}"></button>`)
      .join("")}`;
    row.addEventListener("click", (e) => {
      const b = e.target.closest("[data-accent]");
      if (b) document.querySelector(`#accent-picker [data-value="${b.dataset.accent}"]`)?.click();
    });
    const render = () => {
      for (const b of $$("[data-accent]", row)) b.setAttribute("aria-pressed", String(b.dataset.accent === theme.accent));
      const c = themeColors(theme.appearance, theme.accent);
      $("#token-grid", root).innerHTML = Object.keys(c)
        .filter((k) => !["shadow", "selection"].includes(k))
        .map((k) => `<div class="row" style="gap:8px;font-size:var(--text-sm)"><span style="width:18px;height:18px;border-radius:5px;border:1px solid var(--border);background:var(--${k})"></span><code style="font-size:11.5px">${k.replace(/-/g, "_")}</code></div>`)
        .join("");
    };
    render();
    onThemeChange(() => root.isConnected && render());
  },
});

page("accessibility", {
  description: "Every component is keyboard-operable and announces its state to assistive technology.",
  html: () => `
<h2 id="contract">Behaviour contract</h2>
<ul>
<li><strong>Buttons</strong> are tab stops. Enter or Space activates the focused button, and the focus ring appears only for keyboard focus. Disabled and loading buttons ignore activation. Assistive technology hears disabled, toggled, expanded, busy and keyboard shortcuts.</li>
<li><strong>Popover</strong> opens on press, or on Enter or Space on the focused trigger. Focus moves into the surface; Escape or a press outside closes it, and focus returns to the trigger. Placement flips and clamps to the window.</li>
<li><strong>Menus</strong> add Up, Down, Home and End navigation. <code>MenuItem::dismisses(handle)</code> closes the menu after selection.</li>
<li><strong>Dialog</strong> records the current focus and focuses itself; Tab cycles inside; Escape, the close button and backdrop presses call <code>on_dismiss(reason)</code>. Closing restores focus. Enter is not bound at the dialog level, so a focused button inside it activates normally.</li>
<li><strong>SidebarSection</strong> is a list. Up, Down, Home and End move between items without leaving the section.</li>
<li><strong>Motion</strong> follows the reduced-motion policy. See <a href="#/reduced-motion">Reduced motion</a>.</li>
</ul>
<h2 id="contrast">Contrast</h2>
<p>Unit tests assert WCAG contrast for body text, filled-button labels at rest and on hover, and focus rings, across both appearances and every accent.</p>`,
});

page("credits", {
  description: "The projects zkit's dither work builds on.",
  html: () => `
<h2 id="dithercn">dithercn</h2>
<p><a href="https://dithercn.dev" target="_blank" rel="noreferrer">dithercn</a> by Leo (<a href="https://github.com/opencoredev/dithercn" target="_blank" rel="noreferrer">opencoredev/dithercn</a>, MIT) is a dither engine and component set for React. zkit's dither engine adapts its techniques: the blue-noise threshold data, the field-based approach to dithered surfaces, the globe (including its Natural Earth land mask, sun-lit shading and arcs), the loaders, and many of the example fields. Its copyright and licence notice are kept in <code>crates/dither/LICENSE-MIT</code>.</p>
<h2 id="dither-kit">dither-kit</h2>
<p><a href="https://tripwire.sh/dither-kit" target="_blank" rel="noreferrer">dither-kit</a> by ripgrim is a set of dithered charts, buttons, avatars and gradients. zkit's charts, dithered buttons and generative avatars follow its look: gradient and hatched series fills, stacking, ribbon lines, radar charts, and name-derived symmetric avatars with a hue per name. zkit's implementations are written from scratch in Rust and include no dither-kit source.</p>
<h2 id="licences">Licences in the repository</h2>
<ul><li><code>NOTICE</code> and <code>crates/dither/NOTICE</code> list what was adapted and from where.</li><li><code>crates/dither/LICENSE-MIT</code> carries dithercn's MIT licence and copyright.</li><li>Source files that adapt either project say so in their header comment.</li></ul>
<h2 id="data">Data and media</h2>
<ul>
<li>World land: <a href="https://www.naturalearthdata.com" target="_blank" rel="noreferrer">Natural Earth</a>, public domain.</li>
<li>Sample photo: Buzz Aldrin on the Moon, Apollo 11. NASA, public domain.</li>
<li>Icons on this site: <a href="https://lucide.dev" target="_blank" rel="noreferrer">Lucide</a>, ISC licence.</li>
</ul>
<p>Thank you to everyone who made these.</p>`,
});

/* Components */
page("button", {
  description: "A compact, accessible button with five variants, three sizes and full state support.",
  html: () => `
${preview({
  stage: `<div class="stack" style="align-items:center">
    <div class="row" style="justify-content:center">${button("Commit", { variant: "primary", shortcut: "secondary-enter" })}${button("Create branch", { variant: "accent" })}${button("Fetch")}${button("Stash", { variant: "ghost" })}${button("Discard", { variant: "danger" })}</div>
    <div class="row" style="justify-content:center">${button("Small", { size: "sm" })}${button("Medium")}${button("Large", { size: "lg" })}${button("", { iconOnly: true, label: "Add", leading: icon.plus })}</div>
    <div class="row" style="justify-content:center">${button("Disabled", { disabled: true })}<span id="save-slot">${button("Save", { variant: "primary", attrs: 'id="save"' })}</span>${button("Word wrap", { variant: "ghost", pressed: false, attrs: "data-toggle" })}${button("Branch", { trailing: icon.chevronDown, expanded: false })}</div>
  </div>`,
  rust: `
Button::new("commit", "Commit").primary().shortcut("secondary-enter")
Button::new("branch", "Create branch").accent()
Button::new("fetch", "Fetch")
Button::new("stash", "Stash").ghost()
Button::new("discard", "Discard").danger()

Button::new("save", "Save").primary().loading(self.saving)
Button::new("wrap", "Word wrap").ghost().toggled(self.wrap)
Button::icon("add", IconName::Plus, "Add")`,
})}
<h2 id="variants">Variants</h2>
<table><thead><tr><th>Variant</th><th>Paint</th><th>Use</th></tr></thead><tbody>
<tr><td><code>primary()</code></td><td><code>solid</code> / <code>on_solid</code></td><td>The one main action in a view.</td></tr>
<tr><td><code>accent()</code></td><td><code>accent_strong</code> / <code>on_accent</code></td><td>Creative or forward actions.</td></tr>
<tr><td><code>secondary()</code></td><td><code>surface_raised</code>, bordered</td><td>The default.</td></tr>
<tr><td><code>ghost()</code></td><td>Clear, <code>hover</code> wash</td><td>Toolbars and dense rows.</td></tr>
<tr><td><code>danger()</code></td><td><code>danger_strong</code> / <code>on_danger</code></td><td>Destructive actions.</td></tr>
</tbody></table>
<h2 id="api">API</h2>
${api([
  ["Button::new(id, label)", "A labelled button."],
  ["Button::icon(id, icon, label)", "Icon-only; the label becomes the accessible name."],
  [".small() / .large() / .size(..)", "28, 32 or 36 px tall, with 10 px corners."],
  [".leading(el) / .trailing(el)", "Content before or after the label."],
  [".shortcut(keys)", "Shows a <code>Kbd</code> hint and sets <code>aria-keyshortcuts</code>."],
  [".disabled(bool)", "Inert and dimmed to 45%."],
  [".loading(bool)", "Inert, shows a spinner and reports busy."],
  [".toggled(bool)", "Pressed state for toggle buttons."],
  [".selected(bool) / .expanded(bool)", "Persistent selection; open state for triggers."],
  [".full_width()", "Stretches to its container."],
  [".on_click(handler)", "Runs on pointer or keyboard activation."],
])}
<h2 id="keyboard">Keyboard</h2>
${keys([["enter / space", "Activate the focused button."], ["tab", "Move to the next button."]])}`,
  mount: mountButton,
});
function mountButton(root) {
  $("#save", root).addEventListener("click", () => {
    const slot = $("#save-slot", root);
    slot.innerHTML = button("Saving", { variant: "primary", loading: true });
    hydrate(slot);
    setTimeout(() => {
      if (!slot.isConnected) return;
      slot.innerHTML = button("Save", { variant: "primary", attrs: 'id="save"' });
      mountButton(root);
    }, 1600);
  });
}

page("kbd", {
  description: "Keyboard shortcut hints, rendered as platform key caps.",
  html: () => `
${preview({
  stage: `<div class="stack" style="align-items:center"><div class="row">${kbd("secondary-k")}${kbd("secondary-shift-p")}${kbd("alt-enter")}${kbd("escape")}${kbd("up")}${kbd("ctrl-c ctrl-v")}</div><div class="row">${button("Command palette", { shortcut: "secondary-shift-p" })}</div></div>`,
  rust: `
Kbd::new("secondary-shift-p")
Button::new("palette", "Command palette").shortcut("secondary-shift-p")

format_keystroke("cmd-shift-p", true)  // "⌘⇧P"
format_keystroke("cmd-shift-p", false) // "Ctrl+Shift+P"`,
})}
<p>Takes gpui keystroke notation and renders symbols on macOS (<code>${esc(formatKeystroke("secondary-shift-p", true))}</code>) and words elsewhere (<code>${esc(formatKeystroke("secondary-shift-p", false))}</code>). <code>secondary</code> is Command on macOS and Control elsewhere. This page detected ${isMacLabel()}.</p>
${api([["Kbd::new(keys)", "Key caps for a keystroke or chord sequence."], [".muted_on(color)", "Borderless caps tinted for a filled button."], ["format_keystroke(keys, mac)", "The text form, for menus and tooltips."]])}`,
});
function isMacLabel() {
  return /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent) ? "macOS, so symbols are shown" : "a non-Mac platform, so words are shown";
}

page("badge", {
  description: "A small count or status pill.",
  html: () => `
${preview({ stage: `<div class="row">${badge("3")}${badge("12", true)}${badge("New", true)}${badge("beta")}</div>`, rust: `Badge::new("3")\nBadge::new("12").accent()` })}
${api([["Badge::new(label)", "Neutral: <code>hover</code> wash with muted text."], [".accent()", "<code>accent_wash</code> with accent text."]])}`,
});

page("divider", {
  description: "Hairline separators between groups.",
  html: () => `
${preview({ stage: `<div class="stack" style="width:260px;gap:10px;font-size:var(--text-md)"><span>Above</span><div class="z-divider"></div><span>Below</span><div class="z-divider strong"></div><div class="row" style="height:22px">Left<div class="z-divider vertical"></div>Right</div></div>`, rust: `Divider::horizontal()\nDivider::horizontal().strong()\nDivider::vertical()` })}
${api([["Divider::horizontal() / ::vertical()", "A 1 px <code>border</code> line."], [".strong()", "Uses <code>border_strong</code>."]])}`,
});

page("toolbar", {
  description: "A labelled row of controls at the top of a view.",
  html: () => `
${preview({
  stageClass: "flush",
  stage: `<div class="z-toolbar" role="toolbar" aria-label="Editor actions">${button("", { variant: "ghost", iconOnly: true, label: "Toggle sidebar", leading: icon.sidebar, pressed: true, attrs: "data-toggle" })}${button("Format", { variant: "ghost" })}${button("Wrap", { variant: "ghost", pressed: false, attrs: "data-toggle" })}<span style="flex:1"></span>${button("Run", { variant: "primary", shortcut: "secondary-r" })}</div><div style="height:120px"></div>`,
  rust: `
Toolbar::new("toolbar", "Editor actions")
    .child(Button::icon("sidebar", IconName::Sidebar, "Toggle sidebar").ghost().toggled(open))
    .child(Button::new("format", "Format").ghost())
    .child(spacer())
    .child(Button::new("run", "Run").primary().shortcut("secondary-r"))`,
})}
${api([["Toolbar::new(id, label)", "44 px tall on <code>surface</code> with a bottom border. The label names the toolbar for assistive technology."], [".child(el)", "Controls, usually ghost buttons. Use <code>spacer()</code> to push items apart."]])}`,
});

page("panel", {
  description: "A frame: title and actions in a muted band, the body on an inset plate, and a footer below.",
  html: () => `
${preview({
  caption: "Panel with a title, action and footer",
  stage: panel({
    title: "Problems",
    actions: `${badge("2")}${button("", { variant: "ghost", size: "sm", iconOnly: true, label: "Refresh", leading: icon.replay })}`,
    body: `<div class="stack" style="gap:10px"><div class="row"><span style="color:var(--danger)">●</span> unused variable <code class="muted">x</code></div><div class="row"><span style="color:var(--warning)">●</span> deprecated call to <code class="muted">spawn</code></div></div>`,
    footer: "Last checked 2 minutes ago",
    style: "width:min(440px,100%)",
  }),
  rust: `
Panel::new("problems")
    .title("Problems")
    .action(Button::icon("refresh", "↻", "Refresh").ghost().small())
    .child(problem_list)
    .footer("Last checked 2 minutes ago")`,
})}
${api([["Panel::new(id)", "A muted band with 18 px corners and a 4 px inset; the body plate has 14 px corners, a border and a resting shadow."], [".title(text)", "A semibold heading in the band; also names the region."], [".action(el)", "Controls at the end of the band."], [".footer(el)", "A muted caption row below the plate."], [".borderless()", "Drops the frame, for panels docked edge to edge."]])}`,
});

page("sidebar", {
  description: "Navigation sections with keyboard movement and an animated collapse.",
  html: () => `
${preview({
  stageClass: "flush",
  stage: `<div class="z-toolbar">${button("", { variant: "ghost", iconOnly: true, label: "Toggle sidebar", leading: icon.sidebar, pressed: true, attrs: 'id="sb-toggle"' })}<span style="font-size:var(--text-md)">Workspace</span></div><div style="display:flex;height:260px"><aside class="z-sidebar" id="sb" aria-label="Workspace"><div class="z-sidebar-body"><div class="z-section" role="list" aria-label="Views"><div class="z-section-title">Views</div>${[["Overview", icon.overview], ["Changes", icon.diff, badge("3", true)], ["History", icon.history], ["Archived", icon.archive, "", true]].map(([l, i, t, d], n) => `<button class="z-item" role="listitem" type="button" aria-current="${n === 0}"${d ? " disabled" : ""}><span class="z-icon">${i}</span>${l}${t ?? ""}</button>`).join("")}</div></div></aside><div style="flex:1;padding:16px;font-size:var(--text-md)" class="muted">Use the arrow keys inside a section.</div></div>`,
  rust: `
Sidebar::new("nav", "Workspace")
    .collapsed(!self.sidebar_open)
    .child(SidebarSection::new("views").title("Views").items([
        SidebarItem::new("overview", "Overview").icon("◇").selected(true),
        SidebarItem::new("changes", "Changes").icon("±")
            .trailing(Badge::new("3").accent()),
        SidebarItem::new("history", "History").icon("↺"),
    ]))`,
})}
${api([
  ["Sidebar::new(id, label)", "232 px on <code>surface</code>. Width animates over <code>RESIZE</code> (200 ms ease-out)."],
  [".collapsed(bool) / .collapsed_width(px)", "Collapse fully or to a rail."],
  [".header(el) / .footer(el)", "Fixed regions around the scrolling sections."],
  ["SidebarSection::new(id).title(..).items(..)", "A titled list of items."],
  ["SidebarItem::new(id, label)", "32 px row with <code>.icon</code>, <code>.trailing</code>, <code>.selected</code>, <code>.disabled</code> and <code>.on_click</code>."],
])}
${keys([["up / down", "Previous or next item in the section."], ["home / end", "First or last item."], ["enter / space", "Activate the focused item."]])}`,
  mount(root) {
    $("#sb-toggle", root).addEventListener("click", (e) => {
      const open = $("#sb", root).classList.toggle("collapsed") === false;
      e.currentTarget.setAttribute("aria-pressed", String(open));
    });
  },
});

const INFO_SMALL = icon.info;
const createProject = () =>
  card({
    title: "Create project",
    description: "Deploy your new project in one click.",
    style: "width:min(340px,100%)",
    body: `<div class="stack" style="gap:16px">${field("Name", { placeholder: "Name of your project" })}${field("Framework", { value: "GPUI" })}</div>`,
    footer: button("Deploy", { variant: "primary", full: true }),
  });
page("card", {
  description: "A content card with a header, body and footer. Use it alone or inside a Panel frame.",
  html: () => `
${preview({ caption: "Card", stage: createProject(), rust: `
Card::new("create-project")
    .title("Create project")
    .description("Deploy your new project in one click.")
    .w(px(340.))
    .child(v_stack().gap(px(16.)).child(name_field).child(framework_field))
    .footer(Button::new("deploy", "Deploy").primary().full_width())` })}
<h2 id="framed">Within a frame</h2>
<p>Wrap a card in a <a href="#/panel">Panel</a> to give it a caption band.</p>
${preview({ caption: "Card within a frame and footer", stage: panel({ body: createProject().replace("width:min(340px,100%)", "width:100%"), footer: `${INFO_SMALL} This will take a few seconds to complete.`, style: "width:min(372px,100%)", bodyStyle: "padding:0;border:0;background:none;box-shadow:none" }), rust: `
Panel::new("deploy")
    .plain_body()
    .child(Card::new("create-project") /* … */)
    .footer("This will take a few seconds to complete.")` })}
${api([["Card::new(id)", "18 px corners, <code>surface_raised</code> plate, <code>border_strong</code> hairline and a resting shadow."], [".title(text) / .description(text)", "Semibold 18 px heading and muted body; the title names the group."], [".action(el)", "A control at the header's trailing edge."], [".child(el)", "Body content, inset 24 px."], [".footer(el)", "A row of footer content."]])}
<h2 id="floating">Floating plate</h2>
<p><code>card(theme)</code> is the lower-level plate behind popovers and menus: <code>surface_overlay</code>, a <code>border_strong</code> hairline, 10 px corners, a 4 px inset and the overlay shadow.</p>
${preview({ caption: "card(theme)", stageClass: "surface", stage: `<div class="z-card" style="width:240px;padding:12px;gap:6px"><strong>Branch protection</strong><span class="muted">Require review before merging into main.</span></div>`, rust: `card(cx.theme()).w(px(240.)).child(content)` })}`,
});

page("popover", {
  description: "Floating content anchored to a trigger, opened by pointer or keyboard.",
  html: () => `
${preview({
  height: 260,
  stage: `<div class="z-popover-anchor" data-popover style="margin-bottom:120px">${button("Details", { trailing: icon.chevronDown })}<div class="z-floating z-card" role="dialog" aria-label="Details" style="width:260px;padding:12px;gap:10px"><strong style="font-size:var(--text-md)">Build 4f2a1c9</strong><span class="muted" style="font-size:var(--text-md)">Finished in 42 s on runner linux-x64.</span><div class="row" style="justify-content:flex-end">${button("Done", { size: "sm" })}</div></div></div>`,
  rust: `
Popover::new("details", Button::new("details", "Details").trailing("▾"))
    .width(px(260.))
    .content(|handle, _window, _cx| {
        v_stack()
            .child("Build 4f2a1c9")
            .child(Button::new("done", "Done").small()
                .on_click(move |_, window, cx| handle.dismiss(window, cx)))
    })`,
})}
<p>Entrance uses <code>menu_in</code>: <code>MENU_IN</code> (140 ms, ease), opacity from 0.3 and a 4 px travel from the trigger's side. Placement flips and clamps to the window with the foundation <code>Positioner</code>.</p>
${api([["Popover::new(id, trigger)", "Any <code>Button</code> as the trigger."], [".content(fn)", "Builds the surface; receives a <code>PopoverHandle</code> to dismiss."], [".menu(fn)", "Menu semantics and arrow-key navigation."], [".placement(..) / .align(..)", "Preferred side and alignment."], [".width(px)", "Fixed width; otherwise at least 160 px."]])}
${keys([["enter / space", "Open from the focused trigger."], ["escape", "Close and return focus to the trigger."]])}`,
});

page("menu", {
  description: "Commands in a popover, with headings, separators, shortcuts and checks.",
  html: () => `
${preview({
  height: 380,
  stage: `<div class="z-popover-anchor" data-popover style="margin-bottom:250px">${button("File", { trailing: icon.chevronDown })}<div class="z-floating z-card" role="menu" aria-label="File" style="width:220px">${menu([{ label: "New file", shortcut: "secondary-n" }, { label: "Open…", shortcut: "secondary-o" }, { label: "Revert", disabled: true }, "-", { heading: "View" }, { label: "Word wrap", checked: true }, { label: "Minimap", checked: false }, "-", { label: "Delete", danger: true, shortcut: "secondary-backspace" }])}</div></div>`,
  rust: `
Popover::new("file", Button::new("file", "File").trailing("▾")).menu(|handle, _, _| {
    Menu::new()
        .item(MenuItem::new("new", "New file").shortcut("secondary-n").dismisses(&handle))
        .item(MenuItem::new("open", "Open…").shortcut("secondary-o"))
        .item(MenuItem::new("revert", "Revert").disabled(true))
        .separator()
        .heading("View")
        .item(MenuItem::new("wrap", "Word wrap").checked(wrap))
        .separator()
        .item(MenuItem::new("delete", "Delete").danger())
})`,
})}
${api([["Menu::new()", "A column of rows."], [".item(MenuItem) / .heading(text) / .separator()", "Rows, faint headings and hairlines."], ["MenuItem::new(id, label)", "A 28 px row with 6 px corners."], [".shortcut(keys) / .icon(el) / .checked(bool)", "Hint, leading icon, check mark."], [".danger() / .disabled(bool)", "Destructive colour; inert row."], [".dismisses(&handle) / .on_select(fn)", "Close after selection; handle the choice."]])}
${keys([["up / down", "Move between items."], ["home / end", "First or last item."], ["enter / space", "Select."], ["escape", "Close and restore focus."]])}`,
});

page("dialog", {
  description: "A modal surface with a header, body and a footer band for actions.",
  html: () => `
${preview({
  caption: "Dialog with a form and footer actions",
  height: 420,
  stageClass: "dialog-stage",
  stage: `${button("Edit profile", { attrs: 'id="open-dialog"' })}`,
  rust: `
// In your view
edit: Disclosure,

Dialog::new("edit-profile", &self.edit)
    .title("Edit profile")
    .description("Make changes to your profile here. Click save when you're done.")
    .child(v_stack().gap(px(16.)).child(name_field).child(username_field))
    .action(Button::new("cancel", "Cancel").ghost().on_click(cx.listener(|this, _, w, cx| this.edit.close(w, cx))))
    .action(Button::new("save", "Save").primary().on_click(..))
    .on_dismiss(cx.listener(|this, _reason, w, cx| this.edit.close(w, cx)))`,
})}
<p id="dialog-result" class="muted"></p>
${preview({
  caption: "Confirmation",
  height: 300,
  stageClass: "dialog-stage-2",
  stage: `${button("Discard changes…", { variant: "danger", attrs: 'id="open-confirm"' })}`,
  rust: `
Dialog::new("discard", &self.discard)
    .title("Discard changes?")
    .description("Your 3 uncommitted changes will be lost. This can't be undone.")
    .close_button(false)
    .action(Button::new("cancel", "Cancel").ghost().on_click(..))
    .action(Button::new("confirm", "Discard").danger().on_click(..))`,
})}
<p>The panel has 18 px corners; actions sit in a muted band under a hairline. Entrance uses <code>dialog_in</code>: <code>DIALOG_IN</code> (180 ms, ease) with a 2 px rise over a <code>scrim</code>.</p>
${api([["Dialog::new(id, &disclosure)", "480 px wide on <code>surface_dialog</code>."], [".title(..) / .description(..)", "20 px semibold heading and muted body; they name and describe the dialog."], [".child(el)", "Body content between the header and footer."], [".action(button)", "Right-aligned actions in the footer band."], [".close_button(bool) / .close_on_backdrop(bool)", "Dismissal affordances."], [".on_dismiss(fn)", "Receives <code>DismissReason</code>; the view decides whether to close."]])}
${keys([["tab / shift-tab", "Cycle focus inside the dialog."], ["escape", "Dismiss with <code>DismissReason::Escape</code>."], ["enter", "Activates the focused button. Not bound at the dialog level."]])}`,
  mount(root) {
    const done = (reason) => ($("#dialog-result", root).textContent = `Closed with reason: ${reason}. Focus returned to the trigger.`);
    $("#open-dialog", root).addEventListener("click", () =>
      openDialog({
        host: $(".dialog-stage", root),
        title: "Edit profile",
        description: "Make changes to your profile here. Click save when you're done.",
        body: field("Name", { value: "Margaret Welsh" }) + field("Username", { value: "@maggie.welsh" }),
        actions: button("Cancel", { variant: "ghost", attrs: 'data-action="cancel"' }) + button("Save", { variant: "primary", attrs: 'data-action="save"' }),
        onDismiss: done,
      }),
    );
    $("#open-confirm", root).addEventListener("click", () =>
      openDialog({
        host: $(".dialog-stage-2", root),
        title: "Discard changes?",
        description: "Your 3 uncommitted changes will be lost. This can't be undone.",
        closeButton: false,
        actions: button("Cancel", { variant: "ghost", attrs: 'data-action="cancel"' }) + button("Discard", { variant: "danger", attrs: 'data-action="confirm"' }),
        onDismiss: done,
      }),
    );
  },
});

page("progress-bar", {
  description: "Determinate progress, or an indeterminate sweep while the duration is unknown.",
  html: () => `
${preview({
  stage: `<div class="stack" style="width:min(360px,100%);font-size:var(--text-md)"><div class="stack" style="gap:6px"><div class="row" style="justify-content:space-between"><span>Compiling</span><span class="mono muted" id="pb-value">68%</span></div><div id="pb-slot">${progress({ value: 68, label: "Compiling" })}</div></div><div class="stack" style="gap:6px"><span>Syncing</span>${progress({ label: "Syncing" })}</div><div class="stack" style="gap:6px"><span>Thicker</span>${progress({ value: 40, height: 8 })}</div></div>`,
  controls: `<label>Value <input type="range" min="0" max="100" value="68" id="pb-range" /></label>`,
  rust: `
ProgressBar::new("build").value(68.).label("Compiling")
ProgressBar::new("sync").label("Syncing") // indeterminate
ProgressBar::new("upload").value(40.).height(px(8.))`,
})}
<p>The indeterminate segment is 35% of the track and sweeps over 1.4 s with <code>EASE_IN_OUT</code>, starting and ending hidden. Under reduced motion it becomes a still 40% wash. It reports busy to assistive technology.</p>
${api([["ProgressBar::new(id)", "6 px rounded track on <code>border_strong</code>."], [".value(percent)", "0 to 100. Without it, the bar is indeterminate."], [".label(text) / .color(hsla) / .height(px)", "Accessible name, fill and thickness."]])}`,
  mount(root) {
    $("#pb-range", root).addEventListener("input", (e) => {
      $("#pb-slot i", root).style.width = `${e.target.value}%`;
      $("#pb-value", root).textContent = `${e.target.value}%`;
    });
  },
});

page("spinner", {
  description: "A ring with a quarter arc that turns at a steady rate.",
  html: () => `
${preview({ caption: "16, 20 and 32 px, and inside a loading button", stage: `<div class="row" style="gap:28px">${spinner()}${spinner({ size: 20 })}${spinner({ size: 32 })}${button("Saving", { variant: "primary", loading: true })}${button("Syncing", { loading: true })}</div>`, rust: `Spinner::new()\nSpinner::new().size(px(20.))\nSpinner::new().accessible("indexing", "Indexing workspace")\nButton::new("save", "Saving").primary().loading(true)` })}
<p>A faint track (20% of the colour) with a quarter arc that turns once every 900 ms. The stroke is an eighth of the diameter, never thinner than 1.5 px. Under a system-only motion reduction the arc holds still and breathes slowly, because "work is happening" is essential feedback; an explicit preference keeps it fully still.</p>
${api([["Spinner::new()", "16 px in the accent colour."], [".size(px) / .color(hsla)", "Dimensions and colour."], [".accessible(id, label)", "Adds a progress-indicator role, a name and busy state."]])}`,
});

page("pulse-loader", {
  description: "An ambient loader of staggered, breathing cells.",
  html: () => `
${preview({ stage: `<div class="row" style="gap:28px">${pulseLoader()}${pulseLoader({ cells: 3, cell: 8 })}${pulseLoader({ cells: 7, cell: 4 })}</div>`, rust: `PulseLoader::new()\nPulseLoader::new().cells(3).cell_size(px(8.))` })}
<p>A 2.4 s cosine wave drives opacity (12% to 100%) and scale (90% to 100%), staggered by 150 ms per cell. Under reduced motion every cell holds at 50%.</p>
${api([["PulseLoader::new()", "Five 6 px cells in <code>text_muted</code>."], [".cells(n) / .cell_size(px) / .color(hsla)", "Count, size and colour."]])}`,
});

page("glass-surface", {
  description: "Native AppKit materials on macOS, with a portable backdrop fallback elsewhere.",
  html: () => `
${preview({
  stageClass: "flush",
  stage: `<div style="position:relative;height:280px;overflow:hidden"><div id="glass-bg" style="position:absolute;inset:0"></div><div style="position:absolute;inset:0;display:grid;place-items:center"><div class="z-glass" id="glass-card" style="width:280px;padding:16px;display:flex;flex-direction:column;gap:10px"><strong>Now playing</strong><span class="muted" style="font-size:var(--text-md)">Children stay GPUI elements, with their focus and accessibility.</span><div class="row">${button("Pause", { variant: "primary" })}${button("Next")}</div></div></div></div>`,
  controls: `<label><input type="checkbox" id="glass-opaque" /> Opaque (reduced transparency)</label>`,
  rust: `
// Once, on macOS:
window.enable_scene_overlay();

GlassSurface::new()
    .radius(px(20.))
    .opaque(prefers_reduced_transparency)
    .child(controls)`,
})}
<p>Surfaces defer their content above the base scene. Keep material surfaces as siblings: native materials share one plane, so nested or interleaved materials are not supported. Use uniform radii. On the web, the fallback is a backdrop blur.</p>
${api([["GlassSurface::new()", "Liquid Glass on macOS, 20 px radius."], [".sidebar()", "The system sidebar material instead."], [".radius(px)", "Uniform corner radius."], [".opaque(bool)", "No blur, for reduced transparency or a solid appearance."]])}`,
  mount(root) {
    surface($("#glass-bg", root), { preset: "plasma", cell: 6, animated: true, matrix: "bayer8", palette: (s) => presetPalette(null)(s) });
    $("#glass-opaque", root).addEventListener("change", (e) => $("#glass-card", root).classList.toggle("opaque", e.target.checked));
  },
});

/* Motion */
function curvePath(curve) {
  const pts = [];
  for (let i = 0; i <= 48; i++) pts.push(`${i ? "L" : "M"}${(i / 48) * 100} ${60 - curve.eval(i / 48) * 60}`);
  return pts.join(" ");
}
page("motion", {
  description: "Durations and curves are named once and reused, so every surface moves with the same vocabulary.",
  html: () => `
<p>Select a spec to play it. Curves are CSS <code>cubic-bezier()</code> functions, evaluated exactly by <code>CubicBezier</code> (Newton iteration with a bisection fallback).</p>
<div class="curve-grid">${SPECS.map(
  (s) => `<button class="curve" type="button" data-spec="${s.name}"><b>${s.name}</b><small>${s.ms} ms · ${s.curve.toLowerCase().replace(/_/g, "-")}</small><svg viewBox="0 0 100 60" preserveAspectRatio="none" aria-hidden="true"><path d="M0 60 L100 0" stroke="var(--border)" fill="none" vector-effect="non-scaling-stroke"/><path d="${curvePath(CURVES[s.curve])}" stroke="var(--accent)" stroke-width="1.5" fill="none" vector-effect="non-scaling-stroke"/></svg><div class="track"><i></i></div><small>${s.use}</small></button>`,
).join("")}</div>
${code(`
use zkit_components::motion::{self, ENTER, MENU_IN, MotionSpec, EASE_OUT_EXPO};

element.with_animation("panel-in", ENTER.animation(), |el, t| el.opacity(t));

// A custom spec with a delay folded into the timeline.
const SLOW_ENTER: MotionSpec = MotionSpec::new(480, EASE_OUT_EXPO).with_delay(60);
`)}
<h2 id="curves">Curves</h2>
<table><thead><tr><th>Constant</th><th>Definition</th></tr></thead><tbody>${Object.entries(CURVES).map(([k, c]) => `<tr><td><code>${k}</code></td><td><code>${c.css}</code></td></tr>`).join("")}</tbody></table>
<h2 id="loaders">Loader periods</h2>
<table><thead><tr><th>Spec</th><th>Period</th><th>Use</th></tr></thead><tbody><tr><td><code>PULSE</code></td><td>2400 ms</td><td>Ambient loaders; cells eased individually.</td></tr><tr><td><code>SPIN</code></td><td>750 ms</td><td>Activity indicators.</td></tr></tbody></table>`,
  mount(root) {
    for (const card of $$(".curve", root)) {
      const spec = SPECS.find((s) => s.name === card.dataset.spec);
      const curve = CURVES[spec.curve];
      const dot = $(".track i", card);
      card.addEventListener("click", () => {
        if (reducesMotion()) {
          dot.style.left = "calc(100% - 8px)";
          return;
        }
        const start = performance.now();
        const step = (now) => {
          const x = Math.min(1, (now - start) / spec.ms);
          dot.style.left = `calc(${curve.eval(x)} * (100% - 8px))`;
          if (x < 1) requestAnimationFrame(step);
          else setTimeout(() => (dot.style.left = "0"), 500);
        };
        requestAnimationFrame(step);
      });
    }
  },
});

page("entrances", {
  description: "Helpers that animate an element in with the catalog's specs.",
  html: () => `
${preview({
  height: 240,
  stage: `<div class="stack" style="align-items:center;gap:20px;width:100%"><div class="row" style="justify-content:center">${button("fade_in", { attrs: 'data-play="enter"' })}${button("fade_quick", { attrs: 'data-play="fade"' })}${button("menu_in", { attrs: 'data-play="menu"' })}${button("dialog_in", { attrs: 'data-play="dialog"' })}</div><div id="entrance-target" style="min-height:90px;display:grid;place-items:center"></div></div>`,
  rust: `
motion::fade_in("row-in", row)              // ENTER: opacity, 4 px rise
motion::fade_quick("swap", label)           // FADE_QUICK: opacity only
motion::menu_in("menu-in", -4., surface)    // MENU_IN: from opacity 0.3, travels from the trigger
motion::dialog_in("dialog-in", panel)       // DIALOG_IN: opacity, 2 px rise`,
})}
${api([["fade_in(id, el)", "<code>ENTER</code>: 320 ms expo-out, opacity 0→1 and a 4 px rise."], ["fade_quick(id, el)", "<code>FADE_QUICK</code>: 150 ms ease, opacity only."], ["menu_in(id, from, el)", "<code>MENU_IN</code>: 140 ms ease, opacity 0.3→1, travelling <code>from</code> px."], ["dialog_in(id, el)", "<code>DIALOG_IN</code>: 180 ms ease, opacity and a 2 px rise."]])}
<p>Vertical travel is a relative <code>top</code> inset applied after layout, so like a CSS transform, siblings never move. Divs have no scale transform, so pops are a fade with a short rise. Every helper honours <code>App::reduce_motion</code>: oneshots snap to their end state.</p>`,
  mount(root) {
    const target = $("#entrance-target", root);
    const anim = {
      enter: ["enter", "320ms cubic-bezier(0.16, 1, 0.3, 1)"],
      fade: ["fade", "150ms cubic-bezier(0.25, 0.1, 0.25, 1)"],
      menu: ["menu-in", "140ms cubic-bezier(0.25, 0.1, 0.25, 1)"],
      dialog: ["dialog-in", "180ms cubic-bezier(0.25, 0.1, 0.25, 1)"],
    };
    const content = {
      enter: `<section class="z-panel" style="width:260px"><div class="z-panel-header"><span>New panel</span></div><div class="z-panel-body muted" style="font-size:var(--text-md)">Content entrance</div></section>`,
      fade: `<span class="row" style="font-size:var(--text-lg)">Saved ${icon.check}</span>`,
      menu: `<div class="z-card" style="width:200px">${menu([{ label: "Rename" }, { label: "Duplicate" }, "-", { label: "Delete", danger: true }])}</div>`,
      dialog: `<div class="z-dialog" style="animation:none;width:280px"><div><h3>Dialog</h3><p>Fade with a 2 px rise.</p></div></div>`,
    };
    for (const b of $$("[data-play]", root))
      b.addEventListener("click", () => {
        const [name, timing] = anim[b.dataset.play];
        target.innerHTML = `<div style="animation:${name} ${timing} both">${content[b.dataset.play]}</div>`;
      });
    $("[data-play='enter']", root).click();
  },
});

page("loaders", {
  description: "Three indicators on one shared clock that parks when nothing is visible.",
  html: () => `
${preview({ stage: `<div class="stack" style="width:min(360px,100%);gap:20px"><div class="row" style="gap:24px;justify-content:center">${spinner({ size: 18 })}${pulseLoader()}</div>${progress({ label: "Loading" })}</div>`, rust: `Spinner::new().size(px(18.))\nPulseLoader::new()\nProgressBar::new("load")` })}
<p>Loaders read a lease-based pulse clock at about 30 or 15 fps. It schedules frames only while a loader is painted, so an idle window costs nothing. On this page the same rule holds: animation runs only while a loader is on screen.</p>
<table><thead><tr><th>Loader</th><th>Period</th><th>Reduced motion</th></tr></thead><tbody>
<tr><td>Spinner</td><td>900 ms turn</td><td>Arc holds still and breathes for a system-only reduction; fully still for an explicit preference</td></tr>
<tr><td>Pulse loader</td><td>2400 ms, 150 ms stagger</td><td>Still at 50%</td></tr>
<tr><td>Indeterminate progress</td><td>1400 ms ease-in-out sweep</td><td>Still 40% wash</td></tr>
</tbody></table>`,
});

page("reduced-motion", {
  description: "One policy decides whether anything moves.",
  html: () => `
${preview({
  stage: `<div class="stack" style="align-items:center;gap:18px"><div class="row" id="motion-choice" role="radiogroup" aria-label="Reduce motion">${["system", "reduced", "full"].map((p) => button(p[0].toUpperCase() + p.slice(1), { variant: "ghost", attrs: `role="radio" data-pref="${p}" aria-checked="${policy.preference === p}" aria-pressed="${policy.preference === p}"` })).join("")}</div><div class="row" style="gap:24px">${spinner({ size: 18 })}${pulseLoader()}</div><span class="muted" id="motion-state" style="font-size:var(--text-md)"></span></div>`,
  rust: `
MotionPolicy {
    preference: MotionPreference::System, // or Reduced, Full
    pause_in_background: true,
    ..Default::default()
}
.install(cx);

// The platform layer reports the OS setting:
zkit_components::motion::set_system_reduces_motion(os_reduces_motion, cx);`,
})}
<p>This choice applies to the whole site, including dither surfaces, and is remembered.</p>
${api([["MotionPreference::System", "Follow the operating system."], ["MotionPreference::Reduced", "Always reduce."], ["MotionPreference::Full", "Always animate."], ["pause_in_background", "Also stop while the main window is unfocused."]])}
<p>The toolkit never reads the operating system itself; platform layers report it with <code>set_system_reduces_motion</code>, which keeps the module identical on native and browser targets. When reduced, <code>with_animation</code> oneshots snap to their end state, repeating ones hold their start state, and no frames are scheduled.</p>`,
  mount(root) {
    const sync = () => {
      for (const b of $$("[data-pref]", root)) {
        b.setAttribute("aria-checked", String(b.dataset.pref === policy.preference));
        b.setAttribute("aria-pressed", String(b.dataset.pref === policy.preference));
      }
      $("#motion-state", root).textContent = `Motion is ${reducesMotion() ? "reduced" : "full"}${policy.preference === "system" ? " (following the system)" : ""}.`;
    };
    $("#motion-choice", root).addEventListener("click", (e) => {
      const b = e.target.closest("[data-pref]");
      if (b) setMotionPreference(b.dataset.pref);
    });
    onMotionChange(() => root.isConnected && sync());
    sync();
  },
});

/* Dither */
const presetPalette = (ink, duo = false) => (s) => {
  const c = ink ?? rgbVar(s.host, "--ink-rgb");
  const paper = rgbVar(s.host, "--paper-rgb");
  // Chart fields read crisper as paper and ink alone, like DitherChart.
  if (duo) return [paper, c];
  return [paper, c.map((v, i) => Math.round(paper[i] + (v - paper[i]) * 0.42)), c];
};

page("dither", {
  description: "Paint any tone field behind real GPUI children as batched GPU quads.",
  html: () => `
${preview({
  stageClass: "flush surface",
  stage: `<div style="position:relative;height:320px"><div id="surface-demo" class="stage-fill"></div><div style="position:absolute;inset:0;display:grid;place-items:center"><div class="stack" style="align-items:center;gap:10px"><strong style="font-size:22px;letter-spacing:-0.02em">Ship it.</strong>${button("Deploy", { variant: "primary" })}</div></div></div>`,
  controls: `<label>Field <select class="z-select" id="surface-preset"></select></label><label>Matrix <select class="z-select" id="surface-matrix"><option value="bayer8">Bayer 8</option><option value="bayer4">Bayer 4</option><option value="bayer2">Bayer 2</option><option value="blue-noise">Blue noise</option></select></label>`,
  rust: `
use zkit_components::{DitherSurface, Matrix, Preset};

DitherSurface::new(Preset::Aurora)
    .matrix(Matrix::Bayer8)
    .cell_size(4.)
    .w_full()
    .h(px(320.))
    .child(Button::new("deploy", "Deploy").primary())`,
})}
<p>Paper and ink default to the theme. Try the accent and appearance controls in the header. Foreground controls keep their GPUI focus, activation and accessibility; the texture itself is decorative.</p>
<h2 id="custom">Custom fields</h2>
<p>A field returns a tone from 0 (paper) to 1 (ink) for normalised coordinates, seconds and aspect ratio.</p>
${code(`
DitherSurface::new(Preset::Gradient)
    .field(|x, y, t, _aspect| 0.5 + 0.5 * (x * 12. + y * 4. + t).sin())
`)}
${api([["DitherSurface::new(preset)", "One of 35 built-in <a href=\"#/dither-fields\">fields</a>."], [".field(fn)", "A custom <code>Fn(x, y, seconds, aspect) -> f32</code>."], [".matrix(Matrix)", "Bayer 2, 4, 8 or blue noise. Default Bayer 8."], [".palette(Palette)", "Override theme paper and ink; two or more colours."], [".cell_size(px)", "Logical cell size; default 4. Coarsens to stay near 32,000 samples."], [".contrast(f32)", "Stretch tones around the midpoint."], [".animated(bool) / .still_time(s)", "Motion and the frame held when still or reduced."]])}
<p>Animated surfaces share the leased clock at about 15 fps, stop renewing it outside the content mask, and hold the still frame under reduced motion.</p>`,
  mount(root) {
    const select = $("#surface-preset", root);
    select.innerHTML = FIELDS.filter((f) => f.group === "Backgrounds").map((f) => `<option value="${f.slug}">${f.label}</option>`).join("");
    const s = surface($("#surface-demo", root), { preset: "aurora", cell: 4, matrix: "bayer8", animated: true, palette: presetPalette(null) });
    select.addEventListener("input", () => s.set({ preset: select.value }));
    $("#surface-matrix", root).addEventListener("input", (e) => s.set({ matrix: e.target.value }));
  },
});

page("dither-fields", {
  description: "Every built-in field, live.",
  html: () =>
    GROUPS.map((g) => {
      const items = FIELDS.filter((f) => f.group === g.name);
      return `<h2 id="${g.id}">${g.name} <span class="muted" style="font-weight:400;font-size:var(--text-md)">${items.length}</span></h2><p class="muted">${g.description}</p><div class="catalog-grid">${items.map((f) => `<a class="catalog-item" href="#/dither-fields/${f.slug}"><div class="thumb"><div data-preset="${f.slug}"></div></div>${f.label}</a>`).join("")}</div>`;
    }).join(""),
  mount(root) {
    const charts = new Set(FIELDS.filter((f) => f.group === "Charts").map((f) => f.slug));
    for (const el of $$("[data-preset]", root)) {
      const chart = charts.has(el.dataset.preset);
      surface(el, { preset: el.dataset.preset, cell: chart ? 2 : 3, matrix: "bayer4", animated: false, palette: presetPalette(null, chart) });
    }
  },
});

const MATRICES = [["bayer4", "Bayer 4 × 4", "Bayer4"], ["bayer8", "Bayer 8 × 8", "Bayer8"], ["bayer2", "Bayer 2 × 2", "Bayer2"], ["blue-noise", "Blue noise", "BlueNoise"]];
const INKS = [["Theme", null], ...Object.entries(COLORS).map(([n, c]) => [n[0].toUpperCase() + n.slice(1), c])];
function fieldPage(f) {
  // Chart fields lead with the labelled chart; the raw field stays engine-only.
  if (CHART_FOR[f.slug]) {
    const id = CHART_FOR[f.slug];
    return {
      route: `dither-fields/${f.slug}`,
      title: f.label,
      group: "Dither",
      crumb: "Dither / Fields",
      description: f.description,
      html: () => `
${preview({ caption: "DitherChart with sample data. Hover to read values.", stage: `<div style="width:100%" data-field-chart="${id}"></div>`, rust: rustChart(id, CHARTS[id]) })}
<h2 id="usage">Usage</h2>
${code(rustChart(id, CHARTS[id]))}
<p>See <a href="#/dither-chart">Dither chart</a> for every kind, fill styles, stacking and the full API.</p>
<h2 id="engine-only">Engine only</h2>
<p>The engine also ships <code>Preset::${variant(f.slug)}</code>, a decorative field with built-in sample data, for backgrounds and placeholders without axes or labels.</p>
${code(`
// As a GPUI surface:
DitherSurface::new(Preset::${variant(f.slug)}).w_full().h(px(200.))

// Or as a raster:
let raster = Dither::default().render_preset(320, 180, Preset::${variant(f.slug)}, 6.)?;
`)}`,
      mount(root) {
        for (const el of $$("[data-field-chart]", root)) chart(el, CHARTS[el.dataset.fieldChart]);
      },
    };
  }
  return {
    route: `dither-fields/${f.slug}`,
    title: f.label,
    group: "Dither",
    crumb: "Dither / Fields",
    description: f.description,
    html: () => `
<div class="preview"><div class="preview-tabs"><span class="mono muted" style="font-size:var(--text-sm);padding-left:6px">Preset::${variant(f.slug)}</span><span class="spacer"></span>${button("Replay", { variant: "ghost", size: "sm", leading: icon.replay, attrs: 'id="replay"' })}${button("PNG", { variant: "ghost", size: "sm", leading: icon.download, attrs: 'id="save-png"' })}${button("SVG", { variant: "ghost", size: "sm", leading: icon.download, attrs: 'id="save-svg"' })}</div>
<div class="preview-stage flush surface"><div style="position:relative;height:360px"><div id="field-preview" class="stage-fill"></div></div></div>
<div class="preview-controls"><span class="swatches" id="swatches"></span><label>Matrix <select class="z-select" id="matrix">${MATRICES.map(([id, l]) => `<option value="${id}">${l}</option>`).join("")}</select></label><label>Cell <input type="range" id="cell" min="2" max="8" value="${f.group === "Charts" ? 2 : 3}" /> <span id="cell-out" class="mono">${f.group === "Charts" ? 2 : 3}</span></label><label>Contrast <input type="range" id="contrast" min="0.4" max="2" step="0.1" value="1" /> <span id="contrast-out" class="mono">1.0</span></label></div></div>
<h2 id="usage">Usage</h2><div id="usage"></div>
<h2 id="engine-only">Engine only</h2>
${code(`
let raster = Dither::default().render_preset(320, 180, Preset::${variant(f.slug)}, 6.)?;
std::fs::write("${f.slug}.svg", raster.svg())?;
`)}
<p class="muted">${f.group === "Charts" ? "Chart fields draw sample data. For your own values use <a href=\"#/dither-chart\">DitherChart</a>." : "Backgrounds are decorative. Keep text in real GPUI children on top."}</p>`,
    mount(root) {
      const duo = f.group === "Charts";
      const st = { ink: null, matrix: "bayer4", cell: duo ? 2 : 3, contrast: 1 };
      const s = surface($("#field-preview", root), { preset: f.slug, cell: st.cell, matrix: "bayer4", animated: true, palette: presetPalette(null, duo) });
      const rust = () => `DitherSurface::new(Preset::${variant(f.slug)})
    .matrix(Matrix::${MATRICES.find(([id]) => id === st.matrix)[2]})
    .cell_size(${st.cell}.)
    .contrast(${st.contrast.toFixed(1)})${st.ink ? `\n    .palette(Palette::duo(paper, [${st.ink.join(", ")}]))` : ""}
    .w_full()
    .h(px(320.))`;
      const sync = () => {
        s.set({ matrix: st.matrix, cell: st.cell, contrast: st.contrast, palette: presetPalette(st.ink, duo) });
        $("#cell-out", root).textContent = st.cell;
        $("#contrast-out", root).textContent = st.contrast.toFixed(1);
        $("#usage", root).innerHTML = code(rust());
        wirePreviews($("#usage", root));
      };
      const sw = $("#swatches", root);
      sw.innerHTML = INKS.map(([n, c], i) => `<button class="swatch" type="button" data-i="${i}" style="--swatch:${c ? `rgb(${c})` : "var(--accent)"}" aria-label="${n} ink" aria-pressed="${i === 0}"></button>`).join("");
      sw.addEventListener("click", (e) => {
        const b = e.target.closest("[data-i]");
        if (!b) return;
        st.ink = INKS[+b.dataset.i][1];
        for (const o of sw.children) o.setAttribute("aria-pressed", String(o === b));
        sync();
      });
      $("#matrix", root).addEventListener("input", (e) => ((st.matrix = e.target.value), sync()));
      $("#cell", root).addEventListener("input", (e) => ((st.cell = +e.target.value), sync()));
      $("#contrast", root).addEventListener("input", (e) => ((st.contrast = +e.target.value), sync()));
      $("#replay", root).addEventListener("click", () => s.replay());
      $("#save-png", root).addEventListener("click", () => {
        const out = document.createElement("canvas");
        out.width = s.canvas.width * st.cell;
        out.height = s.canvas.height * st.cell;
        const ctx = out.getContext("2d");
        ctx.fillStyle = `rgb(${rgbVar(s.host, "--paper-rgb")})`;
        ctx.fillRect(0, 0, out.width, out.height);
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(s.canvas, 0, 0, out.width, out.height);
        out.toBlob((b) => b && download(b, `zkit-${f.slug}.png`));
      });
      $("#save-svg", root).addEventListener("click", () => {
        const p = presetPalette(st.ink, duo)(s);
        const svg = export_svg(s.canvas.width, s.canvas.height, f.slug, s.time, st.matrix, new Uint8Array(p.flat()), st.contrast);
        download(new Blob([svg], { type: "image/svg+xml" }), `zkit-${f.slug}.svg`);
      });
      sync();
    },
  };
}

page("dither-loader", {
  description: "Dithered loading indicators: ring, dots, pulse and bar. Rendered by the Rust engine.",
  html: () => `
${preview({ caption: "Four variants at the large size", stage: `<div class="row" style="gap:40px;align-items:center;justify-content:center">${["ring", "dots", "pulse", "bar"].map((v) => `<div class="stack" style="align-items:center;gap:10px"><div data-loader="${v}" data-size="lg"></div><span class="muted mono" style="font-size:var(--text-xs)">${v}</span></div>`).join("")}</div>`, rust: `
DitherLoader::new("sync", "Syncing")                                   // ring
DitherLoader::new("sync", "Syncing").variant(LoaderVariant::Dots)
DitherLoader::new("sync", "Syncing").variant(LoaderVariant::Pulse)
DitherLoader::new("sync", "Syncing").variant(LoaderVariant::Bar)` })}
${preview({ caption: "Sizes: small, medium and large", stage: `<div class="row" style="gap:32px;align-items:center">${["sm", "md", "lg"].map((size) => `<div data-loader="ring" data-size="${size}"></div>`).join("")}</div>`, rust: `DitherLoader::new("a", "Loading").size(LoaderSize::Small)  // 24 px, 1 px cells
DitherLoader::new("b", "Loading")                         // 40 px, 2 px cells
DitherLoader::new("c", "Loading").size(LoaderSize::Large)  // 64 px, 2 px cells` })}
${preview({ caption: "In a button", stage: `<button class="z-button primary" type="button" aria-busy="true"><span data-loader="ring" data-size="sm" style="filter:brightness(0)"></span>Deploying</button>`, rust: `Button::new("deploy", "Deploying").primary().leading(DitherLoader::new("l", "Deploying").size(LoaderSize::Small))` })}
${api([["DitherLoader::new(id, label)", "A busy progress indicator named by <code>label</code>."], [".variant(LoaderVariant)", "<code>Ring</code>, <code>Dots</code>, <code>Pulse</code> or <code>Bar</code>."], [".size(LoaderSize)", "<code>Small</code> (24 px), <code>Medium</code> (40 px) or <code>Large</code> (64 px)."]])}
<p>The ring sweeps a tail that grows and shrinks; the dots breathe in turn; the pulse sends rings out from a beating core; the bar's head eases across a resting track. Each holds a still frame under reduced motion. The tone functions live in <code>zkit_dither::LoaderVariant</code>, so native and web draw the same cells.</p>`,
  mount(root) {
    for (const el of $$("[data-loader]", root)) demo.rustLoader(el, el.dataset.loader, el.dataset.size);
  },
});

page("dither-globe", {
  description: "A rotating dithered globe with a lit terminator, atmosphere, markers and arcs. Rendered by the Rust engine.",
  html: () => `
${preview({ caption: "Offices and routes", stageClass: "flush surface", stage: `<div style="position:relative;height:440px"><div class="stage-fill" id="globe-main"></div></div>`, controls: `<label><input type="checkbox" id="globe-arcs" checked /> Arcs</label><label>Speed <input type="range" id="globe-speed" min="-20" max="20" value="6" /></label><label>Tilt <input type="range" id="globe-tilt" min="-40" max="40" value="18" /></label>`, rust: `
DitherGlobe::new("offices")
    .markers(OFFICES)
    .arcs(ROUTES)
    .speed(6.)
    .tilt(18.)
    .label("Office locations")
    .w_full()
    .h(px(440.))` })}
${api([["DitherGlobe::new(id)", "A globe centred in its box; land in the accent, markers and arcs in the warning colour."], [".markers(places) / .arcs(pairs)", "Places as <code>(lat, lon)</code>; arcs lift off the surface between two places."], [".speed(deg/s) / .tilt(deg)", "Spin and lean. Defaults 6 and 18."], [".cell_size(px)", "Dither cell size; coarsens to stay near 40,000 cells."], [".label(text)", "Makes it an image with an accessible name; decorative otherwise."]])}
<p>Land comes from a 480 × 240 Natural Earth mask (public domain) shipped in <code>zkit-dither</code>. Light is Lambertian with a soft terminator; oceans brighten towards the rim; markers pulse every 2.6 s and a bright stretch travels along each arc. Under reduced motion the globe holds still.</p>`,
  mount(root) {
    let s = demo.rustGlobe($("#globe-main", root));
    const rebuild = () => {
      s.destroy();
      s = demo.rustGlobe($("#globe-main", root), { arcs: $("#globe-arcs", root).checked, speed: +$("#globe-speed", root).value, tilt: +$("#globe-tilt", root).value });
    };
    for (const id of ["#globe-arcs", "#globe-speed", "#globe-tilt"]) $(id, root).addEventListener("input", rebuild);
  },
});

const BUTTON_COLORS = { accent: null, blue: "--chart-2", green: "--chart-3", amber: "--chart-4", pink: "--chart-5" };
page("dither-button", {
  description: "A real button with a dithered face. It grows denser on hover and while pressed.",
  html: () => `
${preview({
  caption: "Hover and press: the dither deepens",
  stage: `<div class="row" id="dbtn-row" style="gap:16px"></div>`,
  controls: `<span class="knob-label">Fill</span><span class="dash-segmented" role="group" aria-label="Fill">${FILLS.map((f, i) => `<button type="button" data-bfill="${f}" aria-selected="${i === 0}">${f}</button>`).join("")}</span>
    <span class="knob-label">Colour</span><span class="swatches">${Object.entries(BUTTON_COLORS).map(([n, v], i) => `<button class="swatch" type="button" data-bcolor="${n}" style="--swatch:${v ? `var(${v})` : "var(--accent)"}" aria-label="${n}" aria-pressed="${i === 0}"></button>`).join("")}</span>`,
  rust: `
DitherButton::new("save", "Save changes")
DitherButton::new("deploy", "Deploy →").fill(ChartFill::Hatched).color(theme.colors.chart[2])
DitherButton::new("off", "Disabled").disabled(true)`,
})}
${api([["DitherButton::new(id, label)", "A foundation button with a dithered face, 36 px tall."], [".fill(ChartFill)", "<code>Gradient</code> (default), <code>Dotted</code>, <code>Hatched</code> or <code>Solid</code>."], [".color(hsla)", "Ink and border colour. Default: the accent."], [".on_click(fn) / .disabled(bool)", "Activation and the inert state."]])}
<p>The face is drawn in 2 px cells. On hover a denser face shows through a group hover, and the border takes the full colour. Keyboard focus, Enter and Space, and the disabled state come from the foundation button.</p>`,
  mount(root) {
    const st = { fill: "gradient", color: "accent" };
    const draw = () => {
      const row = $("#dbtn-row", root);
      for (const el of $$(".dither-layer", row)) el.__surface?.destroy();
      const v = BUTTON_COLORS[st.color];
      const css = v ? `var(${v})` : "var(--accent)";
      row.innerHTML = [["Save changes", ""], ["Deploy →", "lg"], ["Disabled", "", true]]
        .map(([label, size, disabled]) => `<button class="dbtn ${size}" type="button" style="--btn:${css}"${disabled ? " disabled" : ""}><span class="face"></span><span>${label}</span></button>`)
        .join("");
      for (const face of $$(".face", row)) {
        const rgb = v ? rgbVar(root, `${v}-rgb`) : null;
        demo.buttonFace(face, { fill: st.fill, color: rgb });
      }
    };
    for (const b of $$("[data-bfill]", root))
      b.addEventListener("click", () => {
        st.fill = b.dataset.bfill;
        for (const o of $$("[data-bfill]", root)) o.setAttribute("aria-selected", String(o === b));
        draw();
      });
    for (const b of $$("[data-bcolor]", root))
      b.addEventListener("click", () => {
        st.color = b.dataset.bcolor;
        for (const o of $$("[data-bcolor]", root)) o.setAttribute("aria-pressed", String(o === b));
        draw();
      });
    draw();
  },
});

const CHART_FOR = { area: "area", sparkline: "area", stat: "area", bar: "bar", line: "line", heatmap: "heatmap", donut: "donut", radar: "radar" };
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
const HOURS = Array.from({ length: 24 }, (_, h) => `${String(h).padStart(2, "0")}:00`);
const CHARTS = {
  area: {
    title: "Visitors",
    description: "January to June 2026, stacked",
    kind: "area",
    stacked: true,
    categories: MONTHS,
    series: [
      { name: "Desktop", values: [186, 305, 237, 273, 209, 314] },
      { name: "Mobile", values: [80, 200, 120, 190, 130, 140] },
    ],
  },
  bar: {
    title: "Deploys",
    description: "Production and preview, per month",
    kind: "bar",
    stacked: true,
    categories: MONTHS,
    series: [
      { name: "Production", values: [42, 58, 51, 66, 72, 80] },
      { name: "Preview", values: [120, 140, 98, 160, 172, 190] },
    ],
  },
  line: {
    title: "Response time",
    description: "Median and 95th percentile, last 24 hours",
    kind: "line",
    categories: HOURS,
    format: (v) => `${Math.round(v)} ms`,
    series: [
      { name: "p50", values: [62, 60, 58, 57, 57, 59, 66, 78, 86, 90, 92, 91, 88, 90, 93, 95, 94, 88, 82, 76, 72, 68, 65, 63] },
      { name: "p95", values: [140, 132, 128, 126, 127, 134, 158, 196, 222, 236, 241, 238, 230, 236, 244, 252, 248, 230, 210, 192, 176, 162, 150, 144] },
    ],
  },
  radar: {
    title: "Character stats",
    description: "Two builds compared on six attributes",
    kind: "radar",
    height: 280,
    categories: ["Speed", "Power", "Range", "Defense", "Magic", "Luck"],
    series: [
      { name: "Knight", values: [70, 90, 40, 95, 20, 50] },
      { name: "Mage", values: [55, 40, 85, 30, 98, 65] },
    ],
  },
  donut: {
    title: "Requests by region",
    description: "Today",
    kind: "donut",
    categories: ["North America", "Europe", "Asia Pacific", "South America", "Africa"],
    series: [{ name: "Requests", values: [48210, 31870, 17430, 6340, 2150] }],
  },
  heatmap: {
    title: "Deploys by hour",
    description: "Last 30 days, UTC",
    kind: "heatmap",
    categories: ["00", "02", "04", "06", "08", "10", "12", "14", "16", "18", "20", "22"],
    series: [
      ["Mon", [0, 0, 0, 1, 4, 9, 7, 11, 12, 8, 3, 1]],
      ["Tue", [0, 0, 0, 2, 6, 12, 9, 14, 15, 9, 4, 1]],
      ["Wed", [1, 0, 0, 1, 5, 11, 10, 13, 16, 10, 3, 2]],
      ["Thu", [0, 0, 1, 2, 7, 13, 8, 12, 14, 11, 5, 1]],
      ["Fri", [0, 0, 0, 1, 4, 8, 6, 7, 5, 3, 1, 0]],
      ["Sat", [0, 0, 0, 0, 0, 1, 2, 1, 1, 0, 0, 0]],
      ["Sun", [0, 0, 0, 0, 0, 0, 1, 2, 1, 1, 0, 0]],
    ].map(([name, values]) => ({ name, values })),
  },
};
const chartKnobs = (id, spec) =>
  `<span class="knob-label">Fill</span><span class="dash-segmented" role="group" aria-label="Fill">${["auto", "gradient", "hatched", "dotted", "solid"].map((f) => `<button type="button" data-knob="${id}" data-fill="${f}" aria-selected="${f === "auto"}">${f}</button>`).join("")}</span>${
    id === "area" || id === "bar" ? `<label><input type="checkbox" data-stack="${id}" ${spec.stacked ? "checked" : ""}/> Stacked</label>` : ""
  }`;
const rustChart = (id, spec) => {
  const kind = spec.kind[0].toUpperCase() + spec.kind.slice(1);
  const cats = spec.categories.length > 8 ? `(0..${spec.categories.length}).map(|h| format!("{h:02}:00"))` : `[${spec.categories.map((c) => `"${c}"`).join(", ")}]`;
  return `DitherChart::new("${id}", "${spec.title}")
    .description("${spec.description}")
    .kind(ChartKind::${kind})${spec.stacked ? "\n    .stacked(true)" : ""}
    .categories(${cats})
${spec.series.map((s) => `    .series("${s.name}", [${s.values.map((v) => `${v}.`).join(", ")}])`).join("\n")}${spec.format ? `\n    .format(|v| format!("{v:.0} ms"))` : ""}`;
};

page("dither-chart", {
  description: "Charts driven by your data, with axes, labels and a legend. Hover a chart to read its values.",
  html: () => `
${Object.entries(CHARTS)
  .map(([id, spec]) => `<h2 id="${id}">${{ area: "Area", bar: "Bar", line: "Line", radar: "Radar", donut: "Donut", heatmap: "Heatmap" }[id]}</h2>
${preview({ caption: { area: "Stacked areas: gradient under hatched. Hover the legend to spotlight a series; click to lock it.", bar: "Stacked bars with a total in the tooltip", line: "Lines with a custom value format", radar: "One polygon per series, categories as spokes", donut: "Share of a total, with a legend table", heatmap: "One row per series, categories as columns" }[id], stage: `<div style="width:100%" data-chart="${id}"></div>`, controls: ["area", "bar", "line", "radar"].includes(id) ? chartKnobs(id, spec) : "", rust: rustChart(id, spec) })}`)
  .join("")}
<h2 id="api">API</h2>
${api([
  ["DitherChart::new(id, title)", "The title heads the chart and names it for assistive technology, which also hears every value."],
  [".kind(ChartKind)", "<code>Area</code>, <code>Line</code>, <code>Bar</code>, <code>Radar</code>, <code>Donut</code> or <code>Heatmap</code>."],
  [".stacked(bool)", "Stack areas and bars instead of overlaying or grouping them."],
  [".fill(ChartFill)", "Fill of the last added series: <code>Gradient</code>, <code>Hatched</code>, <code>Dotted</code> or <code>Solid</code>. Defaults run in that order, so neighbouring series stay distinct."],
  [".categories(labels)", "The x axis, donut segments or heatmap columns."],
  [".series(name, values)", "A named series. Series take the theme's five chart colours in order; the first is the accent."],
  [".description(text)", "Supporting text under the title."],
  [".format(fn)", "How values read on the axis and legend. Default: compact numbers such as <code>1.2k</code>."],
  [".legend(bool) / .plot_height(px)", "Hide the legend; size the plot (default 200 px)."],
])}
<p>The y axis uses round ticks (steps of 1, 2, 2.5 or 5 × 10ⁿ) from zero. Negative and non-finite values count as zero. Hover tooltips are part of this site; the native component carries the full text summary instead.</p>`,
  mount(root) {
    const state = {};
    const draw = (id) => {
      const { fill = "auto", stacked = CHARTS[id].stacked } = state[id] ?? {};
      const spec = { ...CHARTS[id], stacked, series: CHARTS[id].series.map((s) => ({ ...s, fill: fill === "auto" ? undefined : fill })) };
      chart($(`[data-chart="${id}"]`, root), spec);
    };
    for (const el of $$("[data-chart]", root)) draw(el.dataset.chart);
    for (const b of $$("[data-knob]", root))
      b.addEventListener("click", () => {
        const id = b.dataset.knob;
        state[id] = { ...state[id], fill: b.dataset.fill };
        for (const o of $$(`[data-knob="${id}"]`, root)) o.setAttribute("aria-selected", String(o === b));
        draw(id);
      });
    for (const c of $$("[data-stack]", root))
      c.addEventListener("change", () => {
        state[c.dataset.stack] = { ...state[c.dataset.stack], stacked: c.checked };
        draw(c.dataset.stack);
      });
  },
});

const PEOPLE = ["Ada Lovelace", "Grace Hopper", "Alan Turing", "Katherine Johnson", "Linus Torvalds", "Barbara Liskov", "Ken Thompson", "Hedy Lamarr", "Dennis Ritchie", "Radia Perlman", "Margaret Hamilton", "Donald Knuth"];
page("dither-avatar", {
  description: "Generative avatars: an 8 × 8 symmetric pattern and a hue, both derived from a name.",
  html: () => `
${preview({
  caption: "Type a name; the avatar is deterministic",
  stage: `<div class="avatar-lab"><div id="avatar-big"></div><div class="stack" style="gap:12px;flex:1;min-width:220px"><input class="z-input" id="avatar-name" value="Ada Lovelace" aria-label="Name" /><div class="row" style="gap:10px"><button class="z-button sm" type="button" id="avatar-auto" aria-pressed="true">Hue from name</button><input type="range" min="0" max="358" step="2" id="avatar-hue" aria-label="Hue" style="flex:1;accent-color:var(--accent)" /></div><p class="muted" style="margin:0;font-size:var(--text-sm)">32 mirrored pattern bits × 2 fold axes × 180 hues: about 1.5 trillion avatars. Half fold left/right, half fold top/bottom.</p></div></div>`,
  rust: `DitherAvatar::new("ada", "Ada Lovelace").size(px(96.))
dither_avatar("ada", "Ada Lovelace") // 40 px`,
})}
${preview({ caption: "A team", stage: `<div class="avatar-grid" id="avatar-grid"></div>`, rust: `h_stack().gap(px(12.)).children(team.iter().map(|m| DitherAvatar::new(m.id, &m.name)))` })}
${api([["DitherAvatar::new(id, name)", "Pattern and hue from an FNV-1a hash of the name, stable across platforms. An image labelled \"Avatar for <em>name</em>\"."], [".size(px)", "Edge length; default 40 px. Blocks keep whole dither cells."], ["dither_avatar(id, name)", "The 40 px default."]])}
<p>The pattern is <code>zkit_dither::Avatar</code>, the same code on native and on this page. Lightness follows the appearance so avatars hold contrast in light and dark.</p>`,
  mount(root) {
    let auto = true;
    let big;
    const draw = () => {
      big?.destroy();
      const name = $("#avatar-name", root).value || " ";
      if (auto) $("#avatar-hue", root).value = demo.avatarHueOf(name);
      big = demo.rustAvatar($("#avatar-big", root), name, 96, auto ? undefined : +$("#avatar-hue", root).value);
    };
    $("#avatar-name", root).addEventListener("input", draw);
    $("#avatar-hue", root).addEventListener("input", () => {
      auto = false;
      $("#avatar-auto", root).setAttribute("aria-pressed", "false");
      draw();
    });
    $("#avatar-auto", root).addEventListener("click", (e) => {
      auto = !auto;
      e.currentTarget.setAttribute("aria-pressed", String(auto));
      draw();
    });
    draw();
    const grid = $("#avatar-grid", root);
    for (const name of PEOPLE) {
      const cell = document.createElement("div");
      cell.className = "avatar-cell";
      cell.innerHTML = `<div></div><span>${name.split(" ")[0]}</span>`;
      grid.append(cell);
      demo.rustAvatar(cell.firstElementChild, name, 40);
    }
  },
});

page("dither-image", {
  description: "Dither photos and logos with ordered or error-diffusion methods. Nothing leaves the browser.",
  html: () => `
<div class="preview">
  <div class="preview-stage surface" style="height:440px"><canvas class="dither" id="lab-canvas" role="img" aria-label="Dithered image"></canvas></div>
  <div class="preview-controls">
    <label>Method <select class="z-select" id="lab-method"><option value="atkinson">Atkinson</option><option value="floyd-steinberg">Floyd–Steinberg</option><option value="sierra-lite">Sierra Lite</option><option value="blue-noise">Blue noise</option><option value="bayer8">Bayer 8</option><option value="bayer4">Bayer 4</option></select></label>
    <label>Palette <select class="z-select" id="lab-palette"><option value="ink">Theme</option><option value="blue">Four blues</option><option value="grey">Four greys</option><option value="game">Handheld</option></select></label>
    <label>Width <input type="range" id="lab-size" min="120" max="560" step="20" value="300" /> <span class="mono" id="lab-size-out"></span></label>
    <label class="z-button sm" style="cursor:pointer">Choose image<input type="file" id="lab-upload" accept="image/*" hidden /></label>
    ${button("Save PNG", { size: "sm", leading: icon.download, attrs: 'id="lab-save"' })}
  </div>
</div>
${code(`
let raster = dither_image(width, height, &rgba, palette, ImageMethod::Atkinson)?;
let image = DitherImage::new("photo", &raster, "Buzz Aldrin on the Moon");
`)}
${api([["dither_image(w, h, rgba, palette, method)", "Ordered (Bayer, blue noise) or diffusion (Floyd–Steinberg, Atkinson, Sierra Lite). Serpentine scanning; transparent pixels are skipped."], ["DitherImage::new(id, &raster, label)", "A cached GPU texture with an accessible description. Build once, clone in views to reuse the atlas entry."]])}
<p class="muted" style="font-size:var(--text-sm)">Sample: Buzz Aldrin on the Moon, Apollo 11. NASA, public domain.</p>`,
  mount(root) {
    const palettes = {
      ink: () => [...rgbVar(document.body, "--paper-rgb"), ...rgbVar(document.body, "--ink-rgb")],
      blue: () => [10, 20, 32, 28, 72, 118, 46, 134, 204, 120, 205, 255],
      grey: () => [12, 12, 12, 80, 80, 80, 160, 160, 160, 240, 240, 240],
      game: () => [15, 56, 15, 48, 98, 48, 139, 172, 15, 155, 188, 15],
    };
    let source = null;
    const canvas = $("#lab-canvas", root);
    const render = () => {
      if (!source || !canvas.isConnected) return;
      const width = +$("#lab-size", root).value;
      $("#lab-size-out", root).textContent = `${width}px`;
      const height = Math.max(1, Math.round((width * source.height) / source.width));
      const scratch = document.createElement("canvas");
      scratch.width = width;
      scratch.height = height;
      const ctx = scratch.getContext("2d");
      ctx.drawImage(source, 0, 0, width, height);
      const pixels = convert_image(width, height, ctx.getImageData(0, 0, width, height).data, new Uint8Array(palettes[$("#lab-palette", root).value]()), $("#lab-method", root).value);
      canvas.width = width;
      canvas.height = height;
      canvas.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(pixels), width, height), 0, 0);
      const scale = Math.max(1, Math.floor(Math.min(680 / width, 400 / height)));
      canvas.style.width = `${width * scale}px`;
      canvas.style.height = `${height * scale}px`;
    };
    const load = async (src) => {
      const img = new Image();
      img.src = src;
      await img.decode();
      source = img;
      render();
    };
    load("/media/aldrin.jpg").catch(() => toast("The sample image could not load."));
    for (const id of ["#lab-method", "#lab-palette", "#lab-size"]) $(id, root).addEventListener("input", render);
    onThemeChange(render);
    $("#lab-upload", root).addEventListener("change", (e) => {
      const file = e.target.files[0];
      if (!file) return;
      const url = URL.createObjectURL(file);
      load(url)
        .catch(() => toast("That file could not be read. Choose a PNG, JPEG or WebP."))
        .finally(() => URL.revokeObjectURL(url));
    });
    $("#lab-save", root).addEventListener("click", () => canvas.toBlob((b) => b && download(b, "zkit-dither-image.png")));
  },
});

page("dither-states", {
  description: "Cards, dividers, skeletons, loaders and empty states with a dithered finish.",
  html: () => `
${preview({ stage: `<div class="stack" style="width:min(420px,100%);gap:18px"><div style="position:relative;border-radius:8px;overflow:hidden;border:1px solid var(--border)"><div data-w="card" class="stage-fill"></div><div style="position:relative;padding:16px"><strong>dither_card()</strong><div class="muted" style="font-size:var(--text-md)">A glow field at 0.25 contrast behind any content.</div></div></div><div style="position:relative;height:4px"><div data-w="divider" class="stage-fill"></div></div><div class="stack" style="gap:8px"><div style="position:relative;height:22px;border-radius:4px;overflow:hidden"><div data-w="skeleton" class="stage-fill"></div></div><div style="position:relative;height:22px;width:70%;border-radius:4px;overflow:hidden"><div data-w="skeleton" class="stage-fill"></div></div></div><div class="row" style="gap:12px;font-size:var(--text-md)" role="status"><div data-rust-loader></div>Loading workspace…</div></div>`, rust: `
dither_card().p_4().child(content)
dither_divider()
dither_skeleton()
dither_loader("ws", "Loading workspace")` })}
<h2 id="empty">Empty and not-found states</h2>
${preview({ stage: `<div class="stack" style="align-items:center;gap:14px;text-align:center"><div style="position:relative;width:72px;height:72px"><div data-w="orb" class="stage-fill"></div></div><div style="font-size:20px">No deployments yet</div><div class="muted" style="font-size:var(--text-md)">Push to main to create your first one.</div>${button("Create deployment", { variant: "primary" })}</div>`, rust: `dither_empty_state("No deployments yet", "Push to main to create your first one.", Button::new("create", "Create deployment").primary())\ndither_not_found(Button::new("home", "Go home"))` })}`,
  mount(root) {
    const fields = {
      card: { preset: "glow", contrast: 0.25, cell: 4, matrix: "bayer8", palette: presetPalette(null) },
      divider: { cell: 2, field: (u) => (1 - Math.abs(u * 2 - 1)) * 0.7 },
      skeleton: { cell: 2, animated: true, still: 0, field: (u, v, t) => 0.08 + 0.2 * Math.exp(-(((u - ((t * 0.3) % 1)) * 5) ** 2)) },
      loader: { cell: 2, animated: true, still: 0, field: (u, v, t) => { const x = u - 0.5; const y = v - 0.5; const r = Math.hypot(x, y); return r >= 0.24 && r < 0.44 ? 0.15 + 0.85 * (0.5 + 0.5 * Math.cos(Math.atan2(y, x) - t * 2)) ** 3 : 0; } },
      orb: { preset: "orb", cell: 3, matrix: "bayer8", palette: presetPalette(null) },
    };
    for (const el of $$("[data-w]", root)) surface(el, { matrix: "bayer8", ...fields[el.dataset.w] });
    demo.rustLoader($("[data-rust-loader]", root), "ring", "sm");
  },
});

page("dither-examples", {
  description: "Larger pieces built from fields: type, a globe, waves and photos.",
  html: () => `
${preview({ stageClass: "flush surface", stage: `<div style="position:relative;height:300px"><div class="text-stage" data-demo="text" role="img" aria-label="Signal through noise, in dithered type"></div></div>`, rust: `// Rasterise text into a mask, then return a tone per cell:\n// solid on the outline, a shimmering half-tone inside.` })}
${preview({ caption: "DitherGlobe", stageClass: "flush surface", stage: `<div style="position:relative;height:380px"><div class="stage-fill" data-demo="globe"></div></div>`, rust: `DitherGlobe::new("offices").markers(OFFICES).arcs(ROUTES)` })}
${preview({ stageClass: "flush", stage: `<div style="position:relative;height:320px"><div class="stage-fill" data-demo="aurora"></div></div>`, rust: `DitherSurface::new(Preset::Aurora).palette(\n    Palette::new(vec![[2, 10, 21], [20, 35, 70], [74, 66, 141], [84, 221, 125]]).unwrap(),\n)` })}
${preview({ stageClass: "flush surface", stage: `<div style="position:relative;height:300px"><div class="stage-fill" data-demo="waves"></div></div>`, rust: `DitherSurface::new(Preset::Waves)` })}
${preview({ stageClass: "surface", stage: `<div style="width:100%;height:340px"><div class="image-frame" data-demo="image"></div></div>`, rust: `dither_image(w, h, &rgba, four_blues, ImageMethod::Ordered(Matrix::BlueNoise))` })}`,
  mount(root) {
    demo.ditherText($('[data-demo="text"]', root), ["Signal", "through noise"]);
    demo.rustGlobe($('[data-demo="globe"]', root));
    demo.aurora($('[data-demo="aurora"]', root));
    demo.waves($('[data-demo="waves"]', root));
    demo.ditherImage($('[data-demo="image"]', root), "/media/aldrin.jpg").catch(() => {});
  },
});

page("dither-engine", {
  description: "zkit-dither: fields, matrices, palettes, images and export, with no GPUI dependency.",
  html: () => `
${code(`
use zkit_dither::{Dither, Matrix, Palette, Preset};

let dither = Dither { matrix: Matrix::Bayer4, palette: Palette::duo([0, 0, 0], [132, 157, 255]), contrast: 1. };
let raster = dither.render_preset(320, 180, Preset::Waves, 6.)?;
std::fs::write("waves.svg", raster.svg())?;
`)}
<h2 id="pipeline">Pipeline</h2>
<ol><li><strong>Field.</strong> A tone per cell; out-of-range values clamp, non-finite values become paper.</li><li><strong>Contrast.</strong> The tone stretches around its midpoint.</li><li><strong>Threshold.</strong> The cell's position picks a threshold from the matrix.</li><li><strong>Palette.</strong> The tone picks two neighbouring colours; the threshold picks between them.</li></ol>
<h2 id="matrices">Matrices</h2>
<div class="catalog-grid" id="matrix-grid"></div>
<table><thead><tr><th>Matrix</th><th>Levels</th><th>Character</th></tr></thead><tbody><tr><td><code>Bayer2</code></td><td>5</td><td>Chunky, a visible grid.</td></tr><tr><td><code>Bayer4</code></td><td>17</td><td>The classic cross-hatch.</td></tr><tr><td><code>Bayer8</code></td><td>65</td><td>Finer gradients. The surface default.</td></tr><tr><td><code>BlueNoise</code></td><td>256</td><td>Organic grain, no grid. A 64 × 64 tile.</td></tr></tbody></table>
<h2 id="webassembly">WebAssembly</h2>
<p>With the <code>web</code> feature the crate exports <code>render_field</code>, <code>render_tones</code>, <code>convert_image</code>, <code>export_svg</code> and <code>catalog</code>. This site is built on them.</p>
${code(`
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cd website && npm ci && npm run dev
`)}
<h2 id="limits">Limits</h2>
<p>Rasters are capped at four million cells with checked dimensions and buffers. GPUI surfaces coarsen their cells to stay near 32,000 samples, so work does not grow with display density.</p>`,
  mount(root) {
    for (const [id, label] of MATRICES) {
      const item = document.createElement("div");
      item.className = "catalog-item";
      item.innerHTML = `<div class="thumb"><div></div></div>${label}`;
      $("#matrix-grid", root).append(item);
      surface($(".thumb > div", item), { cell: 3, matrix: id, field: (u) => u });
    }
  },
});

/* Router */
const TITLES = new Map(ROUTES.flatMap(([group, items]) => items.map(([r, t]) => [r, { title: t, group }])));
const ORDER = ROUTES.flatMap(([, items]) => items.map(([r]) => r));

function sidebar(route) {
  const link = (r, title) => `<a href="#/${r}"${r === route ? ' aria-current="page"' : ""}>${title}</a>`;
  return ROUTES.map(
    ([group, items]) =>
      `<div class="side-group"><h4>${group}</h4>${items
        .map(([r, t]) => {
          if (r !== "dither-fields") return link(r, t);
          const open = route.startsWith("dither-fields");
          return `${link(r, t)}${open ? `<div class="side-sub">${FIELDS.map((f) => link(`dither-fields/${f.slug}`, f.label)).join("")}</div>` : ""}`;
        })
        .join("")}</div>`,
  ).join("");
}

function render() {
  const raw = location.hash.replace(/^#\/?/, "") || "introduction";
  const [route, anchor] = raw.split("#");
  // `#/installation.md` (or .mdx) asks for the page as Markdown.
  if (/\.mdx?$/.test(route)) return location.replace(`/docs/${route}`);
  const def = PAGES[route] ?? PAGES.introduction;
  const meta = TITLES.get(def.route) ?? { title: def.title, group: def.group };
  for (const el of $$("#doc .dither-layer")) el.__surface?.destroy();
  const at = ORDER.indexOf(def.route.startsWith("dither-fields/") ? "dither-fields" : def.route);
  const prev = ORDER[at - 1];
  const next = ORDER[at + 1];
  const doc = $("#doc");
  doc.innerHTML = `<div class="doc-head"><div class="doc-crumb">${def.crumb ?? meta.group}</div><h1>${meta.title}</h1><p class="doc-description">${def.description}</p>
  <div class="doc-actions">${button("Copy Markdown", { size: "sm", leading: icon.copy, attrs: "data-copy-markdown" })}<a class="z-button ghost sm" href="/docs/${def.route}.md">${icon.fileCode}View as Markdown</a></div></div>
  <div class="prose">${def.html()}</div>
  <nav class="doc-pager" aria-label="Pages">${prev ? `<a href="#/${prev}"><small>Previous</small><span>${TITLES.get(prev).title}</span></a>` : ""}${next ? `<a class="next" href="#/${next}"><small>Next</small><span>${TITLES.get(next).title}</span></a>` : ""}</nav>`;
  $(".docs-sidebar").innerHTML = sidebar(def.route);
  const headings = $$(".prose h2[id]", doc).map((h) => [h.id, h.textContent]);
  $(".docs-toc").innerHTML = headings.length ? `<h4>On this page</h4>${headings.map(([id, t]) => `<a href="#/${def.route}#${id}">${t}</a>`).join("")}` : "";
  setActiveNav(meta.group);
  document.title = `${meta.title} · zkit`;
  hydrate(doc);
  wirePreviews(doc);
  $("[data-copy-markdown]", doc).addEventListener("click", () =>
    fetch(`/docs/${def.route}.md`)
      .then((r) => (r.ok ? r.text() : Promise.reject()))
      .then((md) => copyText(md, "Markdown copied."), () => toast("Markdown is unavailable on this server.")),
  );
  def.mount?.(doc);
  if (anchor) document.getElementById(anchor)?.scrollIntoView();
  else window.scrollTo(0, 0);
  $(".docs-sidebar [aria-current]")?.scrollIntoView({ block: "nearest" });
}

document.addEventListener("click", (e) => {
  const a = e.target.closest('a[href^="#/"]');
  if (!a) return;
  const [route, id] = a.getAttribute("href").slice(2).split("#");
  const current = location.hash.replace(/^#\/?/, "").split("#")[0];
  if (id && route === current) {
    e.preventDefault();
    history.replaceState(null, "", a.getAttribute("href"));
    document.getElementById(id)?.scrollIntoView({ behavior: reducesMotion() ? "auto" : "smooth" });
  }
});

/** Every page, including one per engine field. The engine must be initialised. */
export function allPages() {
  for (const f of loadFields()) PAGES[`dither-fields/${f.slug}`] = fieldPage(f);
  return PAGES;
}

async function boot() {
  header();
  footer();
  await ready();
  allPages();
  setSearchExtras(() => FIELDS.map((f) => ({ group: "Dither fields", label: f.label, href: `/docs.html#/dither-fields/${f.slug}` })));
  window.addEventListener("hashchange", render);
  render();
}
// The Markdown build (scripts/build-markdown.mjs) imports this module on the server.
if (!import.meta.env.SSR) boot();
