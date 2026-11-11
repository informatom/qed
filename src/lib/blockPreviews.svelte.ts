// Blocks shown as previews (#155): the generator's HTML for each block,
// drawn in the app's colors (app.css takes in the site templates' classes).
// A BlockEditor provides one of these; every BlockList below it, including
// those inside columns and sections, reads it.

import { getContext, setContext } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import { convertFileSrc } from "@tauri-apps/api/core";
import { commands, normalizeBlock, type Block } from "$lib/site";

/** A block's preview: its HTML, or why there is none. */
export type Preview = { html: string } | { error: string };

const KEY = Symbol("block-previews");
const FORMS_SETTING = "qed.blockForms";

export class BlockPreviews {
  /** True: every block shows its form, as before previews. */
  forms = $state(readForms());
  /** Previews by the block's content (see `key`), so equal blocks share one. */
  #cache = new SvelteMap<string, Preview>();
  #loading = new Set<string>();
  #root: string;
  #file: string;

  /**
   * @param root The site folder.
   * @param file The file the blocks are in, relative to it: their language,
   *   and in the footer compact lists.
   */
  constructor(root: string, file: string) {
    this.#root = root;
    this.#file = file;
  }

  setForms(forms: boolean) {
    this.forms = forms;
    try {
      localStorage.setItem(FORMS_SETTING, forms ? "1" : "");
    } catch {
      // Only a convenience: without storage, previews it is next time.
    }
  }

  /** The preview of `block`, `undefined` until it has arrived (see `load`). */
  get(block: Block): Preview | undefined {
    return this.#cache.get(key(block));
  }

  /** Asks the generator for the previews of these blocks it hasn't made yet. */
  async load(blocks: Block[]) {
    const wanted = new Map<string, Block>();
    for (const block of blocks) {
      const k = key(block);
      if (!this.#cache.has(k) && !this.#loading.has(k)) wanted.set(k, $state.snapshot(block));
    }
    if (!wanted.size) return;
    const keys = [...wanted.keys()];
    keys.forEach((k) => this.#loading.add(k));
    try {
      const html = await commands.previewBlocks(this.#root, this.#file, [...wanted.values()]);
      keys.forEach((k, i) => this.#cache.set(k, { html: pointAtFolder(html[i], this.#root) }));
    } catch (e) {
      keys.forEach((k) => this.#cache.set(k, { error: String(e) }));
    } finally {
      keys.forEach((k) => this.#loading.delete(k));
    }
  }
}

export function provideBlockPreviews(value: BlockPreviews) {
  setContext(KEY, value);
}

export function blockPreviews(): BlockPreviews | undefined {
  return getContext<BlockPreviews | undefined>(KEY);
}

function readForms(): boolean {
  try {
    return localStorage.getItem(FORMS_SETTING) === "1";
  } catch {
    return false;
  }
}

function key(block: Block): string {
  return JSON.stringify(normalizeBlock($state.snapshot(block) as Block));
}

/**
 * The generator's links start at `/` (`/media/photo.jpg`); this points
 * images, video and audio at the files in the site folder. Only originals
 * are there, so `srcset` (the copies a build makes) goes. Scripts (the
 * Podlove player) don't run in a preview.
 */
function pointAtFolder(html: string, root: string): string {
  const template = document.createElement("template");
  template.innerHTML = html;
  const content = template.content;
  content.querySelectorAll("script").forEach((script) => script.remove());
  for (const element of content.querySelectorAll("[src], [srcset], [poster]")) {
    element.removeAttribute("srcset");
    element.removeAttribute("sizes");
    for (const attribute of ["src", "poster"]) {
      const value = element.getAttribute(attribute);
      if (value?.startsWith("/")) element.setAttribute(attribute, fileUrl(root, value));
    }
  }
  return template.innerHTML;
}

/** Where a site address (`/media/a.jpg`) comes from in the site folder. */
function fileUrl(root: string, address: string): string {
  let path = address.split(/[?#]/)[0];
  try {
    path = decodeURIComponent(path);
  } catch {
    // Not percent-encoded after all.
  }
  path = path.replace(/^\/+/, "");
  // Media and downloads keep their folder; local episode audio lives in
  // audio/ (served at /episodes/); everything else comes from static/.
  const [first] = path.split("/");
  const inFolder =
    first === "media" || first === "assets"
      ? path
      : first === "episodes"
        ? `audio/${path.slice("episodes/".length)}`
        : `static/${path}`;
  return convertFileSrc(`${root}/${inFolder}`);
}
