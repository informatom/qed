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
