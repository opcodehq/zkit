// Icons from Lucide (ISC licence), imported one by one so only what the site
// uses is bundled. GitHub is the one brand mark Lucide no longer ships.
import {
  Check, ChevronDown, ChevronRight, ChevronsUpDown, ChartColumn, Copy, Download, Ellipsis, File, GitBranch,
  Info, Layers, LayoutGrid, Menu, Moon, PanelLeft, Plus, RefreshCw, RotateCcw, ScrollText, Search, Settings,
  Share, Sun, X, History, Diff, FileCode, Archive, Pause, SkipForward, CircleAlert, TriangleAlert,
} from "lucide";

const attrs = (o) => Object.entries(o).map(([k, v]) => `${k}="${v}"`).join(" ");
/** Render a Lucide icon node as an inline SVG string. */
export const lucide = (node, size = 16) =>
  `<svg class="icon" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${node.map(([tag, a]) => `<${tag} ${attrs(a)}/>`).join("")}</svg>`;

const GITHUB = `<svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>`;

export const icon = {
  search: lucide(Search),
  github: GITHUB,
  sun: lucide(Sun),
  moon: lucide(Moon),
  chevronRight: lucide(ChevronRight),
  chevronDown: lucide(ChevronDown),
  chevronsUpDown: lucide(ChevronsUpDown),
  copy: lucide(Copy),
  download: lucide(Download),
  replay: lucide(RotateCcw),
  refresh: lucide(RefreshCw),
  menu: lucide(Menu),
  close: lucide(X),
  sidebar: lucide(PanelLeft),
  file: lucide(File),
  fileCode: lucide(FileCode),
  branch: lucide(GitBranch),
  plus: lucide(Plus),
  check: lucide(Check),
  info: lucide(Info),
  overview: lucide(LayoutGrid),
  deploy: lucide(Layers),
  chart: lucide(ChartColumn),
  logs: lucide(ScrollText),
  settings: lucide(Settings),
  more: lucide(Ellipsis),
  share: lucide(Share),
  history: lucide(History),
  diff: lucide(Diff),
  archive: lucide(Archive),
  pause: lucide(Pause),
  next: lucide(SkipForward),
  error: lucide(CircleAlert),
  warning: lucide(TriangleAlert),
};
