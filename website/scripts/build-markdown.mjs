// Writes the docs as Markdown for agents and LLM tools:
//   dist/docs/<route>.md and .mdx   one file per page
//   dist/llms.txt                   an index of every page
//   dist/llms-full.txt              every page in one file
// The pages are the same HTML templates the site renders, converted here, so
// the Markdown never drifts from the docs. Live previews become their code.
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { parseHTML } from "linkedom";
import { createServer } from "vite";

const SITE = "https://zkit.opcodehq.com";
// Interactive and drawn elements have no Markdown form.
const SKIP = new Set(["SCRIPT", "STYLE", "SVG", "CANVAS", "BUTTON", "INPUT", "SELECT", "TEXTAREA", "LABEL"]);
const root = resolve(import.meta.dirname, "..");
const out = resolve(root, "dist");

// The page modules touch browser globals when they load; give them a DOM.
const { window, document } = parseHTML("<!doctype html><html><head></head><body></body></html>");
const noop = () => {};
Object.assign(globalThis, {
  window,
  document,
  localStorage: { getItem: () => null, setItem: noop },
  matchMedia: () => ({ matches: false, addEventListener: noop }),
  IntersectionObserver: class { observe() {} unobserve() {} disconnect() {} },
  ResizeObserver: class { observe() {} unobserve() {} disconnect() {} },
  requestAnimationFrame: () => 0,
  cancelAnimationFrame: noop,
  getComputedStyle: () => ({ getPropertyValue: () => "" }),
});

const vite = await createServer({ root, logLevel: "error", server: { middlewareMode: true, hmr: false }, appType: "custom" });
try {
  const engine = await vite.ssrLoadModule("/src/engine/dither.js");
  await engine.default({ module_or_path: await readFile(resolve(root, "src/engine/dither_bg.wasm")) });
  const { ROUTES } = await vite.ssrLoadModule("/src/shell.js");
  const { allPages } = await vite.ssrLoadModule("/src/docs.js");
  const { FIELDS } = await vite.ssrLoadModule("/src/fields.js");
  const pages = allPages();

  const titles = new Map(ROUTES.flatMap(([group, items]) => items.map(([r, t]) => [r, { title: t, group }])));
  const sections = ROUTES.map(([group, items]) => [group, items.map(([r]) => r)]);
  sections.push(["Dither fields", FIELDS.map((f) => `dither-fields/${f.slug}`)]);

  const docs = [];
  for (const [group, routes] of sections) {
    for (const route of routes) {
      const def = pages[route];
      const title = titles.get(route)?.title ?? def.title;
      const body = toMarkdown(def.html());
      const md = `# ${title}\n\n> ${def.description}\n\nSource: ${SITE}/docs#/${route}\n\n${body}\n`;
      docs.push({ group, route, title, description: def.description, md });
      for (const ext of ["md", "mdx"]) {
        const file = resolve(out, "docs", `${route}.${ext}`);
        await mkdir(dirname(file), { recursive: true });
        await writeFile(file, md);
      }
    }
  }

  const index = [
    "# zkit",
    "",
    "> A GPU interface toolkit for native and browser apps, written in Rust: GPUI foundations, styled accessible components, motion and a dither engine.",
    "",
    "Every page below is Markdown. Append `.md` (or `.mdx`) to a docs path, for example " + `${SITE}/docs/installation.md. ` + `All pages in one file: ${SITE}/llms-full.txt`,
    "",
    ...sections.flatMap(([group]) => [
      `## ${group}`,
      "",
      ...docs.filter((d) => d.group === group).map((d) => `- [${d.title}](${SITE}/docs/${d.route}.md): ${d.description}`),
      "",
    ]),
  ].join("\n");
  await writeFile(resolve(out, "llms.txt"), index);
  await writeFile(resolve(out, "llms-full.txt"), docs.map((d) => d.md).join("\n---\n\n"));
  console.log(`markdown: ${docs.length} pages, llms.txt and llms-full.txt`);
} finally {
  await vite.close();
}

/* HTML to Markdown, for the subset the docs templates use. */
function toMarkdown(html) {
  const { document: doc } = parseHTML(`<!doctype html><html><body>${html}</body></html>`);
  return blocks(doc.body).replace(/\n{3,}/g, "\n\n").trim();
}

function blocks(el) {
  let md = "";
  for (const node of el.childNodes) {
    if (node.nodeType === 3) {
      const text = node.textContent.replace(/\s+/g, " ");
      if (text.trim()) md += text;
      continue;
    }
    if (node.nodeType !== 1 || SKIP.has(node.tagName.toUpperCase())) continue;
    const tag = node.tagName;
    if (node.classList.contains("preview")) md += preview(node);
    else if (/^H[1-6]$/.test(tag)) md += `\n\n${"#".repeat(Math.max(2, +tag[1]))} ${inline(node).trim()}\n\n`;
    else if (tag === "P") md += `\n\n${inline(node).trim()}\n\n`;
    else if (tag === "PRE") md += fence(node.textContent);
    else if (tag === "TABLE") md += table(node);
    else if (tag === "UL" || tag === "OL") md += list(node, tag === "OL");
    else if (tag === "BLOCKQUOTE") md += `\n\n${blocks(node).trim().replace(/^/gm, "> ")}\n\n`;
    else if (["A", "CODE", "STRONG", "EM", "B", "I", "KBD", "SPAN", "SMALL"].includes(tag)) md += inline(node);
    else md += `\n\n${blocks(node)}\n\n`;
  }
  return md;
}

// A live preview is a rendering; agents get its code and caption.
function preview(box) {
  const caption = box.querySelector(".caption")?.textContent.trim();
  const code = box.querySelector("[data-pane='code'] code")?.textContent ?? "";
  return `${caption ? `\n\n${caption}\n` : ""}${code.trim() ? fence(code) : ""}`;
}

function fence(src) {
  const text = src.replace(/^\n+|\s+$/g, "");
  const first = text.trimStart();
  const lang = /^\[/.test(first) ? "toml" : /^(cargo|npm|cd|rustup|npx) /.test(first) ? "sh" : "rust";
  return `\n\n\`\`\`${lang}\n${text}\n\`\`\`\n\n`;
}

function inline(el) {
  let md = "";
  for (const node of el.childNodes) {
    if (node.nodeType === 3) md += node.textContent.replace(/\s+/g, " ");
    if (node.nodeType !== 1 || SKIP.has(node.tagName.toUpperCase())) continue;
    const text = () => inline(node);
    switch (node.tagName) {
      case "CODE":
      case "KBD":
        md += `\`${node.textContent}\``;
        break;
      case "STRONG":
      case "B":
        md += `**${text().trim()}**`;
        break;
      case "EM":
      case "I":
        md += `*${text().trim()}*`;
        break;
      case "A":
        md += `[${text().trim()}](${href(node.getAttribute("href") ?? "")})`;
        break;
      case "BR":
        md += "\n";
        break;
      default:
        md += text();
    }
  }
  return md;
}

// Doc links point at the Markdown version; everything else becomes absolute.
function href(h) {
  const doc = h.match(/^(?:\/docs(?:\.html)?)?#\/([^#]+)(#.*)?$/);
  if (doc) return `${SITE}/docs/${doc[1]}.md${doc[2] ?? ""}`;
  return h.startsWith("/") ? SITE + h : h;
}

function table(t) {
  const rows = [...t.querySelectorAll("tr")].map((tr) =>
    [...tr.children].map((c) => inline(c).replace(/\s+/g, " ").replace(/\|/g, "\\|").trim()),
  );
  if (!rows.length) return "";
  const width = Math.max(...rows.map((r) => r.length));
  const line = (r) => `| ${Array.from({ length: width }, (_, i) => r[i] ?? "").join(" | ")} |`;
  return `\n\n${[line(rows[0]), line(rows[0].map(() => "---")), ...rows.slice(1).map(line)].join("\n")}\n\n`;
}

function list(l, ordered) {
  const items = [...l.children].filter((c) => c.tagName === "LI");
  return `\n\n${items.map((li, i) => `${ordered ? `${i + 1}.` : "-"} ${blocks(li).replace(/\n{2,}/g, "\n").trim().replace(/\n/g, "\n   ")}`).join("\n")}\n\n`;
}
