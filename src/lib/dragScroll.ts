// Scrolling while something is dragged: near the top or bottom edge of
// the list (or the window), it scrolls by itself, faster the closer the
// pointer gets. Used by the block list and the page tree.

/** The nearest ancestor that scrolls, or null for the window. */
export function scrollParent(element: HTMLElement): HTMLElement | null {
  for (let parent = element.parentElement; parent; parent = parent.parentElement) {
    const { overflowY } = getComputedStyle(parent);
    if ((overflowY === "auto" || overflowY === "scroll") && parent.scrollHeight > parent.clientHeight) return parent;
  }
  return null;
}

/** Scrolls when `y` is near an edge of `scroller`; returns whether it did. */
export function edgeScroll(scroller: HTMLElement | null, y: number): boolean {
  const edge = 60;
  const box = scroller?.getBoundingClientRect();
  const top = box ? box.top : 0;
  const bottom = box ? box.bottom : window.innerHeight;
  const by = y < top + edge ? -(top + edge - y) / 3 : y > bottom - edge ? (y - bottom + edge) / 3 : 0;
  if (!by) return false;
  (scroller ?? document.scrollingElement)?.scrollBy(0, by);
  return true;
}
