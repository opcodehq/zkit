// The engine's built-in fields, as the docs present them. Names and groups
// come from the Rust crate; descriptions are the site's.
import { catalog } from "./engine.js";

const DESCRIPTIONS = {
  aurora: "Curtains of light that fold and drift along a horizon.",
  gradient: "A diagonal ramp from paper to ink, the simplest field there is.",
  glow: "A soft light source, off centre, falling away to paper.",
  mesh: "Four wandering colour points blended into a mesh.",
  noise: "Layered value noise, drifting slowly.",
  waves: "Stacked swells that roll across the surface.",
  plasma: "Interfering sine fields, the classic demo effect.",
  beams: "Rays fanning down from a source above the frame.",
  silk: "Folded ribbons of light with a little grain.",
  smoke: "Warped noise rising from the bottom of the frame.",
  contours: "Topographic lines over a slow height field.",
  ripples: "Two drops on still water, interfering.",
  ridgelines: "Stacked mountain silhouettes, one ridge in front of the next.",
  metaballs: "Four blobs that merge as they pass.",
  tunnel: "Concentric rings rushing towards the viewer.",
  "retro-grid": "A perspective floor under a striped sun.",
  eclipse: "A dark disc in front of a bright corona.",
  horizon: "A planet's lit rim under a dark sky.",
  starfield: "Stars streaming out from the centre.",
  halftone: "Newsprint dots, larger where the tone is darker.",
  globe: "A rotating sphere with procedural land.",
  "dot-map": "A flat world map drawn in dots.",
  orb: "A lit sphere with a soft terminator.",
  fade: "A vertical fade for the edge of a section.",
  area: "A filled area under a sample trend.",
  bar: "Sample bars, densest at the top.",
  line: "A single trend line on paper.",
  sparkline: "A compact trend for a table cell or stat.",
  heatmap: "A grid of sample activity levels.",
  donut: "A ring split into sample segments.",
  radar: "A filled polygon over concentric rings.",
  gauge: "A half-circle meter at a sample value.",
  progress: "A track with a dithered fill.",
  uptime: "Daily status bars across a period.",
  stat: "A trend under a headline number.",
};

export const FIELDS = [];
export const GROUPS = [
  { id: "backgrounds", name: "Backgrounds", description: "Full-bleed dithered surfaces for heroes, sections and cards." },
  { id: "charts", name: "Charts", description: "Sample-data fields for charts and status displays. Supply your own field for real values." },
];

export function loadFields() {
  if (FIELDS.length) return FIELDS;
  for (const [index, line] of catalog().split("\n").entries()) {
    const [slug, label, group] = line.split("|");
    FIELDS.push({ slug, label, group, index, description: DESCRIPTIONS[slug] ?? "" });
  }
  return FIELDS;
}

/** Rust identifier of a preset, for code samples. */
export const variant = (slug) =>
  slug
    .split("-")
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("");
