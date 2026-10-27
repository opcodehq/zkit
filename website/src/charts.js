// Data charts for the site, mirroring zkit's DitherChart (chart.rs). The
// marks come from the Rust engine (`zkit_dither::Plot`); axes, labels,
// legends and the hover tooltip are HTML.
//
// Credits: chart design follows dither-kit by ripgrim (https://tripwire.sh/dither-kit).
import { surface, rgbVar, render_chart } from "./engine.js";

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
export const FILLS = ["gradient", "hatched", "dotted", "solid"];

/** `1234 → 1.2k`, `2500000 → 2.5M`. */
export function compact(v) {
  const trim = (x, suffix) => `${Math.round(x * 10) / 10}${suffix}`;
  const a = Math.abs(v);
  if (a >= 1e9) return trim(v / 1e9, "B");
  if (a >= 1e6) return trim(v / 1e6, "M");
  if (a >= 1e3) return trim(v / 1e3, "k");
  return trim(v, "");
}

/** Round ticks from zero covering `max`: steps of 1, 2, 2.5 or 5 × 10ⁿ. */
export function niceTicks(max, count = 4) {
  if (!(max > 0)) max = 1;
  const rough = max / count;
  const magnitude = 10 ** Math.floor(Math.log10(rough));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * magnitude).find((s) => s >= rough) ?? 10 * magnitude;
  const steps = Math.max(1, Math.ceil(max / step - 1e-4));
  return Array.from({ length: steps + 1 }, (_, i) => +(i * step).toPrecision(12));
}

const swatch = (i) => `<i style="background:var(--chart-${(i % 5) + 1})"></i>`;

function dataMax({ kind, series, stacked }) {
  const count = Math.max(0, ...series.map((s) => s.values.length));
  if (stacked && (kind === "area" || kind === "bar"))
    return Math.max(0, ...Array.from({ length: count }, (_, i) => series.reduce((a, s) => a + (s.values[i] ?? 0), 0)));
  return Math.max(0, ...series.flatMap((s) => s.values));
}

/** Markup for a chart. Mount it with `mountChart(element, spec)`. */
export function chartHtml(spec) {
  const { title, description = "", kind = "area", categories = [], series = [], format = compact, height = 220 } = spec;
  const legend =
    kind === "donut" || kind === "heatmap" || spec.legend === false
      ? ""
      : `<div class="zc-legend">${series.map((s, i) => `<button type="button" data-series="${i}">${swatch(i)}${esc(s.name)}</button>`).join("")}</div>`;
  const head = `<div class="zc-head"><div>${title && spec.header !== false ? `<b>${esc(title)}</b>` : ""}${description && spec.header !== false ? `<span>${esc(description)}</span>` : ""}</div>${legend}</div>`;
  const label = `${title}. ${series.map((s) => `${s.name}: ${s.values.map((v, i) => `${categories[i] ?? ""} ${format(v)}`.trim()).join(", ")}`).join("; ")}`;

  if (kind === "donut") {
    const values = series[0]?.values ?? [];
    const total = values.reduce((a, b) => a + b, 0);
    return `<figure class="zc" role="img" aria-label="${esc(label)}">${head}
      <div class="zc-donut"><div class="zc-ring"><div class="zc-canvas"></div><div class="zc-center"><b>${format(total)}</b><span>Total</span></div></div>
      <ul class="zc-table">${categories.map((c, i) => `<li data-i="${i}">${swatch(i)}<span>${esc(c)}</span><b>${format(values[i] ?? 0)}</b><em>${total ? Math.round(((values[i] ?? 0) / total) * 100) : 0}%</em></li>`).join("")}</ul></div></figure>`;
  }
  if (kind === "radar") {
    const n = Math.max(3, categories.length);
    const point = (i, r) => {
      const a = (i / n) * Math.PI * 2;
      return [50 + 50 * r * Math.sin(a), 50 - 50 * r * Math.cos(a)];
    };
    const rings = [0.25, 0.5, 0.75, 1].map((r) => `<polygon points="${Array.from({ length: n }, (_, i) => point(i, r * 0.9).join(",")).join(" ")}"/>`).join("");
    const spokes = Array.from({ length: n }, (_, i) => `<line x1="50" y1="50" x2="${point(i, 0.9)[0]}" y2="${point(i, 0.9)[1]}"/>`).join("");
    const labels = categories.map((c, i) => {
      const [x, y] = point(i, 1.02);
      return `<span style="left:${x}%;top:${y}%">${esc(c)}</span>`;
    });
    return `<figure class="zc" role="img" aria-label="${esc(label)}">${head}
      <div class="zc-radar" style="height:${Math.max(height, 240)}px"><div class="zc-radar-box"><svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">${rings}${spokes}</svg><div class="zc-canvas"></div>${labels.join("")}</div></div></figure>`;
  }
  if (kind === "heatmap") {
    const stride = Math.max(1, Math.ceil(categories.length / 8));
    return `<figure class="zc" role="img" aria-label="${esc(label)}">${head}
      <div class="zc-heatmap">
        <div class="zc-rows">${series.map((s) => `<span>${esc(s.name)}</span>`).join("")}</div>
        <div class="zc-heat-plot" style="height:${series.length * 20}px"><div class="zc-canvas"></div><div class="zc-cross band" hidden></div><div class="zc-tip" hidden></div></div>
      </div>
      <div class="zc-x bands" style="--axis:44px">${categories.map((c, i) => `<span>${i % stride ? "" : esc(c)}</span>`).join("")}</div>
      <div class="zc-scale">Less${[0, 0.25, 0.5, 0.75, 1].map((l) => `<span class="zc-step" data-level="${l}"></span>`).join("")}More</div></figure>`;
  }
  const ticks = niceTicks(dataMax(spec), 4);
  const top = ticks[ticks.length - 1];
  const stride = Math.max(1, Math.ceil(categories.length / 8));
  const axis = Math.ceil(14 + 7 * Math.max(...ticks.map((t) => format(t).length)));
  return `<figure class="zc" role="img" aria-label="${esc(label)}" style="--axis:${axis}px">${head}
    <div class="zc-body">
      <div class="zc-y" style="height:${height}px">${ticks.map((t) => `<span style="top:${(1 - t / top) * 100}%">${format(t)}</span>`).join("")}</div>
      <div class="zc-plot" style="height:${height}px">
        <div class="zc-canvas"></div><div class="zc-cross" hidden></div><div class="zc-tip" hidden></div>
      </div>
    </div>
    <div class="zc-x ${kind === "bar" ? "bands" : ""}">${categories.map((c, i) => `<span>${i % stride ? "" : esc(c)}</span>`).join("")}</div></figure>`;
}

/** Draw the marks with the Rust engine and wire the legend and tooltip. */
export function mountChart(root, spec) {
  const { kind = "area", categories = [], series = [], format = compact, stacked = false } = spec;
  const count = Math.max(categories.length, ...series.map((s) => s.values.length));
  const values = new Float32Array(series.flatMap((s) => Array.from({ length: count }, (_, i) => s.values[i] ?? 0)));
  const fills = series.map((s, i) => s.fill ?? FILLS[i % 4]).join(",");
  const top = kind === "donut" ? 1 : kind === "heatmap" ? Math.max(1e-9, dataMax(spec)) : niceTicks(dataMax(spec), 4).at(-1);
  let focus = -1;
  const colors = (host) => {
    const paper = rgbVar(host, "--background-rgb");
    const n = kind === "donut" ? count : kind === "heatmap" ? 1 : series.length;
    return new Uint8Array(
      Array.from({ length: n }, (_, i) => {
        const c = rgbVar(host, `--chart-${(i % 5) + 1}-rgb`);
        // Spotlight: other series fade towards the background.
        return focus >= 0 && i !== focus ? c.map((v, k) => Math.round(v * 0.3 + (paper[k] ?? 0) * 0.7)) : c;
      }).flat(),
    );
  };
  const host = root.querySelector(".zc-canvas");
  const s = surface(host, {
    cell: 2,
    intro: 0.9,
    still: 0.9,
    raster: (w, h, t, st) => render_chart(kind, w, h, values, series.length, fills, stacked, top, Math.min(1, t / 0.8), colors(st.host)),
  });

  // Legend: hover to spotlight a series, click to lock it.
  let locked = -1;
  for (const b of root.querySelectorAll(".zc-legend [data-series], .zc-table [data-i]")) {
    const i = +(b.dataset.series ?? b.dataset.i);
    b.addEventListener("pointerenter", () => locked < 0 && ((focus = i), s.set({})));
    b.addEventListener("pointerleave", () => locked < 0 && ((focus = -1), s.set({})));
    b.addEventListener("click", () => {
      locked = locked === i ? -1 : i;
      focus = locked;
      for (const o of root.querySelectorAll("[data-series]")) o.setAttribute("aria-pressed", String(+o.dataset.series === locked));
      s.set({});
    });
  }

  // Heatmap: dithered legend steps, and a tooltip for the cell under the pointer.
  if (kind === "heatmap") {
    for (const step of root.querySelectorAll(".zc-step")) {
      const level = +step.dataset.level;
      surface(step, {
        cell: 2,
        raster: (w, h, t, st) => render_chart("heatmap", w, h, new Float32Array([level]), 1, "", false, 1, 1, colors(st.host)),
      });
    }
    const grid = root.querySelector(".zc-heat-plot");
    const tip = grid.querySelector(".zc-tip");
    grid.addEventListener("pointermove", (e) => {
      const rect = grid.getBoundingClientRect();
      const i = Math.min(count - 1, Math.floor(((e.clientX - rect.left) / rect.width) * count));
      const r = Math.min(series.length - 1, Math.floor(((e.clientY - rect.top) / rect.height) * series.length));
      tip.hidden = false;
      tip.innerHTML = `<b>${esc(series[r].name)} ${esc(categories[i] ?? "")}</b><span>${swatch(0)}Value<em>${format(series[r].values[i] ?? 0)}</em></span>`;
      const tw = tip.offsetWidth;
      const x = ((i + 1) / count) * rect.width;
      tip.style.left = `${x + 8 + tw > rect.width ? x - tw - 8 - rect.width / count : x + 8}px`;
      tip.style.top = `${(r / series.length) * rect.height}px`;
    });
    grid.addEventListener("pointerleave", () => (tip.hidden = true));
    return s;
  }

  // Hover: a crosshair (or band) and a tooltip with every series' value.
  const plot = root.querySelector(".zc-plot");
  if (!plot) return s;
  const cross = root.querySelector(".zc-cross");
  const tip = root.querySelector(".zc-tip");
  plot.addEventListener("pointermove", (e) => {
    const rect = plot.getBoundingClientRect();
    const u = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
    const i = kind === "bar" ? Math.min(count - 1, Math.floor(u * count)) : Math.round(u * (count - 1));
    const x = kind === "bar" ? ((i + 0.5) / count) * rect.width : (i / Math.max(1, count - 1)) * rect.width;
    cross.hidden = false;
    cross.classList.toggle("band", kind === "bar");
    cross.style.left = `${x}px`;
    if (kind === "bar") cross.style.width = `${(rect.width / count) * 0.8}px`;
    tip.hidden = false;
    const total = series.reduce((a, s) => a + (s.values[i] ?? 0), 0);
    tip.innerHTML = `<b>${esc(categories[i] ?? "")}</b>${series.map((s, j) => `<span>${swatch(j)}${esc(s.name)}<em>${format(s.values[i] ?? 0)}</em></span>`).join("")}${stacked && series.length > 1 ? `<span class="total">Total<em>${format(total)}</em></span>` : ""}`;
    const tw = tip.offsetWidth;
    tip.style.left = `${Math.min(rect.width - tw, Math.max(0, x + 14 + tw > rect.width ? x - tw - 14 : x + 14))}px`;
  });
  plot.addEventListener("pointerleave", () => {
    cross.hidden = true;
    tip.hidden = true;
  });
  return s;
}

/** Render a chart into `slot` and mount it. */
export function chart(slot, spec) {
  for (const el of slot.querySelectorAll(".dither-layer")) el.__surface?.destroy();
  slot.innerHTML = chartHtml(spec);
  return mountChart(slot.firstElementChild, spec);
}
