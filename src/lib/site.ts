// TypeScript mirrors of the Rust types in crates/qed-core, as they arrive
// from `invoke("open_site")`. Keep them in sync by hand for now.

import { invoke } from "@tauri-apps/api/core";

export type Menu = "none" | "main" | "footer";

export interface FrontMatter {
  title: string;
  description: string | null;
  draft: boolean;
  menu: Menu;
  weight: number;
  redirect_from: string[];
  /** ISO date, e.g. "2026-10-21". */
  date: string | null;
  authors: string[];
  tags: string[];
  summary: string | null;
}

export interface Hero {
  image: string | null;
  headline: string | null;
  text: string | null;
  button_label: string | null;
  button_link: string | null;
}

export interface CallToAction {
  headline: string | null;
  text: string | null;
  button_label: string | null;
  button_link: string | null;
}

// A "discriminated union": the `type` field tells TypeScript which of the
// shapes a block has, like a tagged tuple in Elixir.
export type Block =
  | { type: "text"; markdown: string }
  | ({ type: "hero" } & Hero)
  | ({ type: "call-to-action" } & CallToAction);

export interface Page {
  path: string;
  file: string;
  front: FrontMatter;
  blocks: Block[];
}

export interface SiteConfig {
  title: string;
  language: string;
  description: string | null;
  theme: string;
  base_url: string | null;
}

export interface Site {
  root: string;
  config: SiteConfig;
  pages: Page[];
}

/** Reads a site folder. Rejects with a readable message if it can't. */
export function openSite(path: string): Promise<Site> {
  return invoke<Site>("open_site", { path });
}

export interface BuildReport {
  out_dir: string;
  pages: number;
  drafts_skipped: number;
  redirects: number;
  media_files: number;
}

/** Builds the site into its `_site` folder. */
export function buildSite(path: string): Promise<BuildReport> {
  return invoke<BuildReport>("build_site", { path });
}

export interface PreviewInfo {
  url: string;
  report: BuildReport;
}

/** Builds the site and serves it locally. */
export function previewSite(path: string): Promise<PreviewInfo> {
  return invoke<PreviewInfo>("preview_site", { path });
}

/** Saves a page's settings into its file; resolves to the page as saved. */
export function savePageSettings(root: string, file: string, front: FrontMatter): Promise<Page> {
  return invoke<Page>("save_page_settings", { root, file, front });
}

/** Creates an empty page below `parent` ("" for the top level). */
export function createPage(root: string, parent: string, title: string, date: string | null): Promise<Page> {
  return invoke<Page>("create_page", { root, parent, title, date });
}

/** The URL slug for a title, computed in Rust. */
export function slugify(title: string): Promise<string> {
  return invoke<string>("slugify", { title });
}

/** Today's date as YYYY-MM-DD, in local time. */
export function today(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}
