/**
 * The site folders opened on this computer, newest first. Kept in the
 * window's local storage, like the app theme: it's a convenience per
 * computer, not part of any site.
 */

export interface RecentSite {
  path: string;
  title: string;
  /** Milliseconds since 1970, when it was last opened. */
  opened: number;
}

const KEY = "qed:recent-sites";
/** Before the list, only the last site was kept. */
const OLD_KEY = "qed:last-site";
const MAX = 10;

export function recentSites(): RecentSite[] {
  try {
    const stored = localStorage.getItem(KEY);
    if (stored) {
      const list: unknown = JSON.parse(stored);
      if (Array.isArray(list)) return list.filter(isRecentSite);
    }
    const last = localStorage.getItem(OLD_KEY);
    return last ? [{ path: last, title: folderName(last), opened: Date.now() }] : [];
  } catch {
    // Storage can be unavailable or hold something unreadable; start empty.
    return [];
  }
}

/** Puts the site first in the list and returns the new list. */
export function rememberSite(path: string, title: string): RecentSite[] {
  const list = [{ path, title, opened: Date.now() }, ...recentSites().filter((s) => s.path !== path)].slice(0, MAX);
  save(list);
  return list;
}

/** Takes a site off the list (the folder stays) and returns the new list. */
export function forgetSite(path: string): RecentSite[] {
  const list = recentSites().filter((s) => s.path !== path);
  save(list);
  return list;
}

/** The last part of a folder path, as a title for sites not opened yet. */
export function folderName(path: string): string {
  return path.split(/[/\\]/).filter(Boolean).pop() ?? path;
}

function save(list: RecentSite[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(list));
    localStorage.removeItem(OLD_KEY);
  } catch {
    // Remembering is a convenience; without storage the app still works.
  }
}

function isRecentSite(value: unknown): value is RecentSite {
  if (typeof value !== "object" || value === null) return false;
  const site = value as Record<string, unknown>;
  return typeof site.path === "string" && typeof site.title === "string" && typeof site.opened === "number";
}
