// The page tree in the Pages tab: its order, and what dropping a dragged
// page somewhere in it means.

import type { Page } from "$lib/site";

/** The URL path a page sits below: `blog` for `blog/hello`, `""` at the top. */
export function parentOf(path: string): string {
  const slash = path.lastIndexOf("/");
  return slash < 0 ? "" : path.slice(0, slash);
}

/**
 * The pages in the order the site shows them, each followed by the pages
 * below it: the start page first; among siblings, pages by weight, then
 * title (as menus and page lists do), then posts by date. A folder without
 * a page of its own sorts like a page with weight 0 and its name as title.
 */
export function treeOrder(pages: Page[]): Page[] {
  const byPath = new Map(pages.map((page) => [page.path, page]));
  const compareNodes = (a: string, b: string): number => {
    const pageA = byPath.get(a);
    const pageB = byPath.get(b);
    const dated = (page?: Page) => (page?.front.date ? 1 : 0);
    if (dated(pageA) !== dated(pageB)) return dated(pageA) - dated(pageB);
    if (!pageA?.front.date) {
      const weight = (pageA?.front.weight ?? 0) - (pageB?.front.weight ?? 0);
      if (weight) return weight;
      const title = (pageA?.front.title ?? a.slice(a.lastIndexOf("/") + 1)).localeCompare(
        pageB?.front.title ?? b.slice(b.lastIndexOf("/") + 1),
        undefined,
        { sensitivity: "base" },
      );
      if (title) return title;
    }
    return a < b ? -1 : a > b ? 1 : 0;
  };
  return [...pages].sort((a, b) => {
    if (a.path === "" || b.path === "") return a.path === "" ? -1 : 1;
    const partsA = a.path.split("/");
    const partsB = b.path.split("/");
    // The first level where they part: compare the pages (or folders)
    // there. If one is above the other, it comes first.
    for (let i = 0; i < Math.min(partsA.length, partsB.length); i++) {
      if (partsA[i] !== partsB[i]) {
        return compareNodes(partsA.slice(0, i + 1).join("/"), partsB.slice(0, i + 1).join("/"));
      }
    }
    return partsA.length - partsB.length;
  });
}

/** Where on a row the pointer is: above it, on it, or below it. */
export type Zone = "before" | "onto" | "after";

/** What a drop does. */
export interface DropPlan {
  /** The new parent, when the page moves (its address changes). */
  moveTo: string | null;
  /**
   * The pages below the target parent in their new order (files), with
   * `null` for the dragged page; `null` for a post, whose place is its date.
   */
  order: (string | null)[] | null;
}

/**
 * What dropping `dragged` at `zone` of `row` would do, or `null` if
 * nothing (it's already there, or it can't go there). `ordered` is the tree
 * in `treeOrder`; `hasShownChildren` is true when `row` is unfolded with
 * pages below it.
 */
export function planDrop(
  ordered: Page[],
  dragged: Page,
  row: Page,
  zone: Zone,
  hasShownChildren: boolean,
): DropPlan | null {
  if (dragged.path === "") return null;
  let parent: string;
  let anchor: { path: string; after: boolean } | null;
  if (zone === "onto") {
    // Onto the start page means the top level.
    parent = row.path;
    anchor = null;
  } else if (zone === "after" && hasShownChildren) {
    // The line shows between the page and its first child.
    parent = row.path;
    anchor = { path: "", after: true };
  } else if (row.path === "") {
    // Below the start page: first at the top level.
    if (zone === "before") return null;
    parent = "";
    anchor = { path: "", after: true };
  } else {
    parent = parentOf(row.path);
    anchor = { path: row.path, after: zone === "after" };
  }
  // A page can't go below itself or below the pages under it.
  if (parent === dragged.path || parent.startsWith(`${dragged.path}/`)) return null;
  const moves = parent !== parentOf(dragged.path);

  if (dragged.front.date) return moves ? { moveTo: parent, order: null } : null;

  const siblings = ordered.filter(
    (page) => !page.front.date && page.path !== "" && page.path !== dragged.path && parentOf(page.path) === parent,
  );
  let index: number;
  if (anchor === null) index = siblings.length;
  else if (anchor.path === "") index = 0;
  else {
    const at = siblings.findIndex((page) => page.path === anchor.path);
    // Next to a post: after the pages, which come before the posts.
    index = at < 0 ? siblings.length : at + (anchor.after ? 1 : 0);
  }
  const order: (string | null)[] = siblings.map((page) => page.file);
  order.splice(index, 0, null);

  if (!moves) {
    const current = ordered
      .filter((page) => !page.front.date && page.path !== "" && parentOf(page.path) === parent)
      .map((page) => (page.path === dragged.path ? null : page.file));
    if (current.join("\n") === order.join("\n")) return null;
  }
  return { moveTo: moves ? parent : null, order };
}
