// Writes q.e.d.'s copy of the Tabler icons (#149) from the npm package
// @tabler/icons into crates/qed-core/assets/tabler/: one line per icon,
// its name, a tab and the SVG's inner markup. Run with
// `npm run build:icons` after updating the package; see the README there.
import { readFileSync, writeFileSync, copyFileSync, mkdirSync } from "node:fs";

const from = "node_modules/@tabler/icons";
const to = "crates/qed-core/assets/tabler";
mkdirSync(to, { recursive: true });

const escape = (value) => String(value).replaceAll("&", "&amp;").replaceAll('"', "&quot;").replaceAll("<", "&lt;");

function write(style) {
  const nodes = JSON.parse(readFileSync(`${from}/tabler-nodes-${style}.json`, "utf8"));
  const lines = Object.keys(nodes)
    .sort()
    .map((name) => {
      const inner = nodes[name]
        .map(([tag, attrs]) => {
          const list = Object.entries(attrs).map(([key, value]) => ` ${key}="${escape(value)}"`);
          return `<${tag}${list.join("")}/>`;
        })
        .join("");
      return `${name}\t${inner}`;
    });
  writeFileSync(`${to}/${style}.txt`, lines.join("\n") + "\n");
  return lines.length;
}

const outline = write("outline");
const filled = write("filled");

// Search words: the category and Tabler's tags, so "envelope" finds mail.
const meta = JSON.parse(readFileSync(`${from}/icons.json`, "utf8"));
const tags = Object.keys(meta)
  .sort()
  .map((name) => {
    const words = [meta[name].category, ...(meta[name].tags ?? [])]
      .filter(Boolean)
      .map((word) => String(word).toLowerCase().replaceAll(/\s+/g, "-"));
    return `${name}\t${[...new Set(words)].join(" ")}`;
  });
writeFileSync(`${to}/tags.txt`, tags.join("\n") + "\n");
copyFileSync(`${from}/LICENSE`, `${to}/LICENSE`);

const version = JSON.parse(readFileSync(`${from}/package.json`, "utf8")).version;
console.log(`Tabler ${version}: ${outline} outline and ${filled} filled icons → ${to}`);
