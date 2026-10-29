// Opens the user manual (#110) in its own window, built into the app
// (see `open_manual` in src-tauri): the same pages as on the docs site,
// offline, in the app's theme.

import { loadAppTheme } from "$lib/appTheme";
import { commands } from "$lib/site";

/** The app's theme as shown: the chosen one, or light or dark as the system has it. */
function shownTheme(): string {
  return loadAppTheme() ?? (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
}

/** Opens the manual at `page` (`versions/`), or at its start. */
export async function openManual(page?: string) {
  await commands.openManual(shownTheme(), page ?? null);
}
