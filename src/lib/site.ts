// TypeScript mirrors of the Rust types in crates/qed-core, as they arrive
// from `invoke("open_site")`. Keep them in sync by hand for now.

import { Channel, convertFileSrc, invoke } from "@tauri-apps/api/core";

export type Menu = "none" | "main" | "footer";

export interface FrontMatter {
  title: string;
  description: string | null;
  draft: boolean;
  menu: Menu;
  /** A page icon name from qed's set, shown in menus. */
  icon: string | null;
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

export interface ImageBlock {
  image: string | null;
  alt: string | null;
  caption: string | null;
}

export type Side = "left" | "right";

export interface TextImageBlock {
  text: string | null;
  image: string | null;
  alt: string | null;
  image_side: Side;
}

export interface GalleryImage {
  image: string;
  alt: string | null;
}

export interface GalleryBlock {
  images: GalleryImage[];
}

export interface DownloadBlock {
  file: string | null;
  label: string | null;
}

export interface FaqItem {
  question: string;
  answer: string;
}

export interface FaqBlock {
  heading: string | null;
  items: FaqItem[];
}

export interface Column {
  title: string | null;
  /** Markdown; images with ![alt](/media/file.jpg). */
  text: string;
}

export interface ColumnsBlock {
  items: Column[];
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
  | ({ type: "image" } & ImageBlock)
  | ({ type: "text-image" } & TextImageBlock)
  | ({ type: "gallery" } & GalleryBlock)
  | ({ type: "download" } & DownloadBlock)
  | ({ type: "faq" } & FaqBlock)
  | ({ type: "columns" } & ColumnsBlock)
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
  /** Where the main menu goes on the generated site. */
  navigation: "top" | "sidebar";
}

export interface Site {
  root: string;
  config: SiteConfig;
  pages: Page[];
  /** Publish targets, from targets.toml. */
  targets: Target[];
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

/** Saves a page's blocks; resolves to the page as its file now reads. */
export function savePageBlocks(root: string, file: string, blocks: Block[]): Promise<Page> {
  return invoke<Page>("save_page_blocks", { root, file, blocks });
}

export type BlockType = Block["type"];

export const BLOCK_NAMES: Record<BlockType, string> = {
  text: "Text",
  hero: "Hero",
  image: "Image",
  "text-image": "Text with image",
  gallery: "Gallery",
  download: "Download",
  faq: "FAQ",
  columns: "Columns",
  "call-to-action": "Call to action",
};

/** A new, empty block of the given type. */
export function newBlock(type: BlockType): Block {
  switch (type) {
    case "text":
      return { type, markdown: "" };
    case "hero":
      return { type, image: null, headline: null, text: null, button_label: null, button_link: null };
    case "image":
      return { type, image: null, alt: null, caption: null };
    case "text-image":
      return { type, text: null, image: null, alt: null, image_side: "right" };
    case "gallery":
      return { type, images: [] };
    case "download":
      return { type, file: null, label: null };
    case "faq":
      return { type, heading: null, items: [{ question: "", answer: "" }] };
    case "columns":
      return { type, items: [{ title: null, text: "" }, { title: null, text: "" }] };
    case "call-to-action":
      return { type, headline: null, text: null, button_label: null, button_link: null };
  }
}

/** Fields that may be empty in a file. Everything else is always text. */
const OPTIONAL = new Set(["image", "alt", "headline", "text", "caption", "button_label", "button_link", "file", "label", "heading", "title"]);

/** Empty optional fields become null, so "" and "nothing" count as the same. */
export function normalizeBlock(block: Block): Block {
  return normalize(block, true) as Block;
}

function normalize(value: unknown, topLevel: boolean): unknown {
  if (Array.isArray(value)) return value.map((item) => normalize(item, false));
  if (value === null || typeof value !== "object") return value;
  const entries = Object.entries(value).map(([key, field]) => {
    // Inside lists, only alt texts and column titles are optional.
    const optional = topLevel ? OPTIONAL.has(key) : key === "alt" || key === "title";
    if (optional && typeof field === "string" && field.trim() === "") return [key, null];
    return [key, normalize(field, false)];
  });
  return Object.fromEntries(entries);
}

export type MediaKind = "image" | "document" | "other";

export interface MediaFile {
  /** Path inside media/, e.g. "team.jpg" or "logos/a.png". */
  name: string;
  size: number;
  kind: MediaKind;
  width: number | null;
  height: number | null;
  /** URL paths of the pages that use the file. */
  used_by: string[];
}

export function listMedia(root: string): Promise<MediaFile[]> {
  return invoke<MediaFile[]>("list_media", { root });
}

/** Copies files into the media folder; resolves to their new names. */
export function importMedia(root: string, paths: string[]): Promise<string[]> {
  return invoke<string[]>("import_media", { root, paths });
}

export function deleteMedia(root: string, name: string): Promise<void> {
  return invoke<void>("delete_media", { root, name });
}

/** A URL the app window can load a media file from (for thumbnails). */
export function mediaUrl(root: string, name: string): string {
  return convertFileSrc(`${root}/media/${name}`);
}

/** "1.4 MB", "820 KB", "12 bytes". */
export function formatSize(bytes: number): string {
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${bytes} bytes`;
}

export type SftpAuth = "password" | "key";

export interface SftpTarget {
  kind: "sftp";
  name: string;
  host: string;
  port: number;
  user: string;
  /** The folder on the server, e.g. /var/www/html. */
  path: string;
  auth: SftpAuth;
  /** The trusted server key fingerprint, set on first connection. */
  host_key: string | null;
  delete_stale: boolean;
}

/** Only SFTP for now; GitHub Pages will join this union. */
export type Target = SftpTarget;

export function newSftpTarget(): SftpTarget {
  return { kind: "sftp", name: "", host: "", port: 22, user: "", path: "", auth: "password", host_key: null, delete_stale: false };
}

export function saveTargets(root: string, targets: Target[]): Promise<void> {
  return invoke<void>("save_targets", { root, targets });
}

export function setTargetPassword(target: SftpTarget, password: string): Promise<void> {
  return invoke<void>("set_target_password", { target, password });
}

export interface SecretStatus {
  has_password: boolean;
  public_key: string | null;
}

export function targetSecrets(target: SftpTarget): Promise<SecretStatus> {
  return invoke<SecretStatus>("target_secrets", { target });
}

/** Generates a key pair; resolves to the public key line for the server. */
export function generateTargetKey(target: SftpTarget): Promise<string> {
  return invoke<string>("generate_target_key", { target });
}

export type TestOutcome =
  | { outcome: "connected"; message: string }
  | { outcome: "untrusted"; fingerprint: string };

export function testTarget(target: SftpTarget): Promise<TestOutcome> {
  return invoke<TestOutcome>("test_target", { target });
}

export type Stage = "connecting" | "comparing" | "uploading" | "deleting" | "done";

export interface Progress {
  stage: Stage;
  done_files: number;
  total_files: number;
  done_bytes: number;
  total_bytes: number;
  current: string | null;
}

export interface PublishReport {
  uploaded: number;
  deleted: number;
  unchanged: number;
  bytes: number;
}

/** Builds and publishes; `onProgress` is called while it runs. */
export function publishSite(root: string, targetName: string, onProgress: (p: Progress) => void): Promise<PublishReport> {
  const channel = new Channel<Progress>();
  channel.onmessage = onProgress;
  return invoke<PublishReport>("publish_site", { root, targetName, onProgress: channel });
}

/** The daisyUI theme names, from Rust (the list lives in qed-core). */
export function listThemes(): Promise<string[]> {
  return invoke<string[]>("list_themes");
}

/** Saves the site's settings into site.toml. */
export function saveSiteConfig(root: string, config: SiteConfig): Promise<void> {
  return invoke<void>("save_site_config", { root, config });
}

let iconCache: Promise<Map<string, string>> | null = null;

/** Page icons by name (SVG markup), loaded once from Rust. */
export function pageIcons(): Promise<Map<string, string>> {
  iconCache ??= invoke<[string, string][]>("list_icons").then((list) => new Map(list));
  return iconCache;
}
