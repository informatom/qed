/**
 * Updates of the qed app itself, from GitHub releases (Tauri's updater
 * plugin). One shared state, so the bar at the top and the App tab show
 * the same thing. Runes work outside components in a `.svelte.ts` file.
 */

import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

export const updates = $state({
  /** This app's version, once known. */
  version: "",
  /** A newer version that can be installed. */
  available: null as Update | null,
  checking: false,
  /** When the last check finished, in milliseconds since 1970. */
  checkedAt: null as number | null,
  installing: false,
  /** Downloaded bytes and the total (0 if the server didn't say). */
  done: 0,
  total: 0,
  error: null as string | null,
  /** The bar was closed for this version. */
  dismissed: false,
});

/**
 * Asks GitHub whether there's a newer version. `quiet` keeps errors to
 * itself: at start, being offline isn't worth a message.
 */
export async function checkForUpdate(quiet = false) {
  updates.checking = true;
  updates.error = null;
  try {
    updates.version ||= await getVersion();
    const found = await check();
    if (found?.version !== updates.available?.version) updates.dismissed = false;
    updates.available = found;
    updates.checkedAt = Date.now();
  } catch (e) {
    if (!quiet) updates.error = String(e);
  } finally {
    updates.checking = false;
  }
}

/** Downloads and installs the available version, then restarts qed. */
export async function installUpdate() {
  const update = updates.available;
  if (!update) return;
  updates.installing = true;
  updates.error = null;
  updates.done = 0;
  updates.total = 0;
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") updates.total = event.data.contentLength ?? 0;
      if (event.event === "Progress") updates.done += event.data.chunkLength;
    });
    // On Windows the installer has closed qed already; elsewhere, restart
    // into the new version.
    await relaunch();
  } catch (e) {
    updates.error = String(e);
    updates.installing = false;
  }
}
