// The app's own theme: a per-computer preference, like the window size, so
// it lives in the browser storage of the app window, not in any site folder.

const KEY = "qed:app-theme";

/** The saved theme, or null to follow the system's light/dark mode. */
export function loadAppTheme(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}

export function saveAppTheme(theme: string | null) {
  try {
    if (theme) localStorage.setItem(KEY, theme);
    else localStorage.removeItem(KEY);
  } catch {
    // Without storage, the choice lasts until the app closes.
  }
}

/** Shows the app in a theme; null follows the system. */
export function applyAppTheme(theme: string | null) {
  if (theme) document.documentElement.dataset.theme = theme;
  else delete document.documentElement.dataset.theme;
}
