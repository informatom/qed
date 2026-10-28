// What block lists need to know about the site's shared blocks (the files
// in `shared/`, see shared.rs). The main page provides it once; every
// BlockList below it, including those inside columns, reads it.

import { getContext, setContext } from "svelte";
import type { Block, Contributor, Page } from "$lib/site";

export interface SharedBlocks {
  /** The site's shared blocks, sorted by name. */
  readonly pieces: Page[];
  /** True while editing a shared block itself: it can't hold another one. */
  readonly insidePiece: boolean;
  /** Whether the site has a podcast: only then are the podcast blocks offered. */
  readonly podcast: boolean;
  /** The podcast's contributors, for a Post list of someone's episodes. */
  readonly contributors: Contributor[];
  /** Opens a shared block in the editor panel. */
  open(name: string): void;
  /** Makes a new shared block holding `block`; resolves to its name. */
  make(title: string, block: Block): Promise<string>;
}

const KEY = Symbol("shared-blocks");

export function provideSharedBlocks(value: SharedBlocks) {
  setContext(KEY, value);
}

export function sharedBlocks(): SharedBlocks | undefined {
  return getContext<SharedBlocks | undefined>(KEY);
}

/** Whether these blocks (columns included) show the shared block `name`. */
export function usesShared(blocks: Block[], name: string): boolean {
  return blocks.some(
    (block) =>
      (block.type === "shared" && block.name === name) ||
      (block.type === "columns" && block.items.some((column) => usesShared(column.blocks, name))),
  );
}
