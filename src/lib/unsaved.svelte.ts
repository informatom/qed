// Which parts of the app hold unsaved changes, so closing the window,
// opening another site or restarting for an update can ask first. Each
// part reports itself while it's on screen:
//
//   trackUnsaved(() => "Site settings", () => changed);

import { ask } from "@tauri-apps/plugin-dialog";
import { SvelteSet } from "svelte/reactivity";

const places = new SvelteSet<string>();

/**
 * Keeps `place` reported while `dirty()` is true; call it while a component
 * starts. When the component goes away, so does its report.
 */
export function trackUnsaved(place: () => string, dirty: () => boolean) {
  $effect(() => {
    const name = place();
    reportUnsaved(name, dirty());
    return () => reportUnsaved(name, false);
  });
}

/** Notes that `place` (shown to people, e.g. "Site settings") has unsaved changes, or not. */
export function reportUnsaved(place: string, dirty: boolean) {
  if (dirty) places.add(place);
  else places.delete(place);
}

/** The places with unsaved changes. */
export function unsavedPlaces(): string[] {
  return [...places];
}

/**
 * Asks whether to go ahead with `action` ("Close q.e.d.", "Install the
 * update and restart") although changes would be lost; true right away if
 * there are none.
 */
export async function mayDiscardUnsaved(action: string): Promise<boolean> {
  const list = unsavedPlaces();
  if (list.length === 0) return true;
  return ask(`There are unsaved changes in ${list.join(", ")}. ${action} anyway? The changes are lost.`, {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: "Discard changes",
    cancelLabel: "Keep editing",
  });
}
