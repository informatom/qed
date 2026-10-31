// Opens the user manual (#110) in its own window, built into the app
// (see `open_manual` in src-tauri): the same pages as on the docs site,
// offline, in the app's theme and in the chosen language (#145).

import { loadAppTheme } from "$lib/appTheme";
import { commands } from "$lib/site";

const LANGUAGE_KEY = "qed:manual-language";

/** The app's theme as shown: the chosen one, or light or dark as the system has it. */
function shownTheme(): string {
  return loadAppTheme() ?? (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
}

/** The manual's language picked in the App tab, or null for the computer's own. */
export function loadManualLanguage(): string | null {
  try {
    return localStorage.getItem(LANGUAGE_KEY);
  } catch {
    return null;
  }
}

export function saveManualLanguage(language: string | null) {
  try {
    if (language) localStorage.setItem(LANGUAGE_KEY, language);
    else localStorage.removeItem(LANGUAGE_KEY);
  } catch {
    // Without storage, the choice lasts until the app closes.
  }
}

/** The language the manual opens in: the picked one, else the computer's if the manual has it, else English. */
export async function manualLanguage(): Promise<string> {
  const picked = loadManualLanguage();
  if (picked) return picked;
  const languages = await commands.manualLanguages();
  const own = navigator.language.split("-")[0].toLowerCase();
  return languages.includes(own) ? own : "en";
}

/** Opens the manual at `page` (`versions/`), or at its start. */
export async function openManual(page?: string) {
  await commands.openManual(shownTheme(), await manualLanguage(), page ?? null);
}
