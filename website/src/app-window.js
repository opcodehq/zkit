// The example dashboard on the home page and the introduction, composed
// from the components: sidebar, page header, stat cards, a dithered chart
// and a deployments list.
import { surface, clamp } from "./engine.js";
import { $, $$ } from "./shell.js";
import { icon } from "./icons.js";
import { button, badge, menu, openDialog } from "./components.js";
import { chart } from "./charts.js";
import { rustAvatar } from "./demos.js";

const STATS = [
  { label: "Requests", value: "1.28M", delta: "+12.4%", up: true, series: [3, 4, 3.6, 5, 4.6, 5.8, 6.2, 5.9, 7, 7.4, 8.1, 8.6] },
  { label: "Error rate", value: "0.42%", delta: "−0.08%", up: true, series: [6, 5.5, 6.2, 5, 4.6, 4.9, 4.1, 3.8, 4, 3.4, 3.2, 3] },
  { label: "P95 latency", value: "182 ms", delta: "+6 ms", up: false, series: [4, 4.2, 4.1, 4.6, 4.4, 4.8, 5.1, 4.9, 5.3, 5.2, 5.5, 5.6] },
];
const hours = Array.from({ length: 24 }, (_, h) => `${String(h).padStart(2, "0")}:00`);
const days = (n) => Array.from({ length: n }, (_, i) => {
  const d = new Date(Date.UTC(2026, 9, 27 - (n - 1 - i)));
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric", timeZone: "UTC" });
});
const RANGES = {
  "24h": { categories: hours, requests: [22, 18, 15, 14, 16, 24, 38, 52, 61, 66, 70, 68, 64, 66, 71, 74, 72, 65, 58, 50, 44, 38, 31, 26].map((v) => v * 1000) },
  "7d": { categories: days(7), requests: [912, 1040, 1188, 1102, 1264, 744, 690].map((v) => v * 1000) },
  "30d": { categories: days(30), requests: Array.from({ length: 30 }, (_, i) => Math.round((820 + i * 14 + 160 * Math.sin(i * 0.9) - (i % 7 >= 5 ? 280 : 0)) * 1000)) },
};
const trafficSpec = (range) => {
  const { categories, requests } = RANGES[range];
  return {
    title: "",
    header: false,
    kind: "area",
    height: 170,
    categories,
    series: [
      { name: "Requests", values: requests },
      { name: "Cached", values: requests.map((v, i) => Math.round(v * (0.58 + 0.08 * Math.sin(i)))) },
    ],
  };
};
const DEPLOYS = [
  { status: "building", message: "Restyle dialogs and cards", author: "Ada Lovelace", branch: "main", sha: "4f2a1c9", time: "Just now" },
  { status: "ready", message: "Port the globe to Rust", author: "Grace Hopper", branch: "main", sha: "9b7e3d2", time: "12m ago" },
  { status: "ready", message: "Add dither loaders", author: "Alan Turing", branch: "dither", sha: "c41a08f", time: "1h ago" },
  { status: "failed", message: "Bump wgpu to 26", author: "Radia Perlman", branch: "deps", sha: "2e9f6b1", time: "3h ago" },
];

/** Linear interpolation through a series at u in 0..1, normalised to 0..1. */
const sampler = (series) => {
  const lo = Math.min(...series);
  const hi = Math.max(...series);
  return (u) => {
    const x = clamp(u) * (series.length - 1);
    const i = Math.min(series.length - 2, Math.floor(x));
    const f = x - i;
    const k = f * f * (3 - 2 * f);
    return ((series[i] + (series[i + 1] - series[i]) * k) - lo) / (hi - lo || 1);
  };
};

export function appWindow() {
  const statusDot = { building: "var(--warning)", ready: "var(--success)", failed: "var(--danger)" };
  const statusLabel = { building: "Building", ready: "Ready", failed: "Failed" };
  return `<div class="dash-wrap"><div class="dash" id="app">
    <aside class="dash-side" aria-label="Workspace">
      <div class="z-popover-anchor dash-team" data-popover>
        <button class="dash-team-button" type="button"><span class="dash-logo">A</span><span><b>Acme</b><small>Pro plan</small></span>${icon.chevronsUpDown}</button>
        <div class="z-floating z-card" role="menu" aria-label="Switch team" style="width:200px">${menu([{ heading: "Teams" }, { label: "Acme", group: "team", checked: true }, { label: "Personal", group: "team", checked: false }, "-", { label: "Create team…" }])}</div>
      </div>
      <nav class="z-section" role="list" aria-label="Pages">
        ${[["Overview", "overview", true], ["Deployments", "deploy"], ["Analytics", "chart"], ["Logs", "logs"], ["Settings", "settings"]]
          .map(([label, ic, on]) => `<button class="z-item" role="listitem" type="button" aria-current="${!!on}">${icon[ic]}${label}${label === "Deployments" ? badge("1", true) : ""}</button>`)
          .join("")}
      </nav>
      <div class="dash-usage">
        <div class="row" style="justify-content:space-between"><span>Bandwidth</span><span class="mono">68%</span></div>
        <div class="z-progress" role="progressbar" aria-valuenow="68" aria-valuemin="0" aria-valuemax="100" aria-label="Bandwidth used"><i style="width:68%"></i></div>
        <small>68 of 100 GB this month</small>
      </div>
    </aside>
    <main class="dash-main">
      <header class="dash-header">
        <div><h3>Overview</h3><p>acme/web · Production</p></div>
        <div class="row">${button("Share", { leading: icon.share })}${button("Deploy", { variant: "primary", attrs: 'id="dash-deploy"' })}</div>
      </header>
      <section class="dash-stats" aria-label="Key metrics">
        ${STATS.map(
          (s, i) => `<div class="dash-card dash-stat"><span class="dash-label">${s.label}</span><div class="dash-stat-row"><b>${s.value}</b><span class="dash-delta ${s.up ? "good" : "bad"}">${s.delta}</span></div><div class="dash-spark" data-spark="${i}"></div></div>`,
        ).join("")}
      </section>
      <section class="dash-card dash-chart-card" aria-label="Traffic">
        <div class="dash-card-head">
          <div><b>Traffic</b><span class="dash-label">Requests, with the share served from cache</span></div>
          <div class="dash-segmented" role="tablist" aria-label="Range">${Object.keys(RANGES).map((r, i) => `<button type="button" role="tab" aria-selected="${i === 0}" data-range="${r}">${r}</button>`).join("")}</div>
        </div>
        <div class="dash-chart-slot" id="dash-chart"></div>
      </section>
      <section class="dash-card dash-list" aria-label="Recent deployments">
        <div class="dash-card-head"><b>Recent deployments</b><a class="dash-link" href="#" onclick="return false">View all</a></div>
        <ul>
          ${DEPLOYS.map(
            (d) => `<li><span class="dash-status"><i style="background:${statusDot[d.status]}"></i>${statusLabel[d.status]}</span><span class="dash-msg"><span class="dash-avatar" data-author="${d.author}" title="${d.author}"></span>${d.message}</span><span class="dash-branch mono">${d.branch}</span><span class="dash-meta mono">${d.sha}</span><span class="dash-meta">${d.time}</span>
              <div class="z-popover-anchor" data-popover>${button("", { variant: "ghost", size: "sm", iconOnly: true, label: `Actions for ${d.sha}`, leading: icon.more })}<div class="z-floating z-card" role="menu" aria-label="Deployment actions" style="left:auto;right:0;width:180px">${menu([{ label: "Inspect" }, { label: "Copy URL", shortcut: "secondary-c" }, "-", { label: "Redeploy", value: "redeploy" }, { label: "Delete", danger: true }])}</div></div></li>`,
          ).join("")}
        </ul>
      </section>
    </main>
  </div></div>`;
}

/** Wire the dashboard's behaviour. Call after `hydrate` on an ancestor. */
export function wireWindow(root) {
  for (const el of $$("[data-author]", root)) rustAvatar(el, el.dataset.author, 20);
  const redeploy = () =>
    openDialog({
      host: root,
      title: "Deploy to production?",
      description: "This builds main at 4f2a1c9 and promotes it to acme.dev once checks pass.",
      actions: button("Cancel", { variant: "ghost", attrs: 'data-action="cancel"' }) + button("Deploy", { variant: "primary", attrs: 'data-action="deploy"' }),
    });
  $("#dash-deploy", root).addEventListener("click", redeploy);
  for (const anchor of $$(".dash-list [data-popover]", root)) anchor.addEventListener("select", (e) => e.detail === "redeploy" && redeploy());

  for (const el of $$("[data-spark]", root)) {
    const at = sampler(STATS[+el.dataset.spark].series);
    surface(el, {
      cell: 2,
      matrix: "bayer4",
      intro: 1,
      still: 1,
      field(u, v, t, s) {
        if (u > Math.min(1, t * 1.2)) return 0;
        const top = 0.92 - 0.8 * at(u);
        const dy = (v - top) * s.h;
        if (Math.abs(dy) < 0.8) return 1;
        return dy > 0 ? 0.42 * Math.exp(-dy / 6) : 0;
      },
    });
  }

  const slot = $("#dash-chart", root);
  chart(slot, trafficSpec("24h"));
  for (const tab of $$("[data-range]", root))
    tab.addEventListener("click", () => {
      for (const t of $$("[data-range]", root)) t.setAttribute("aria-selected", String(t === tab));
      for (const el of $$(".dither-layer", slot)) el.__surface?.destroy();
      chart(slot, trafficSpec(tab.dataset.range));
    });
}
