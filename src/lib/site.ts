// Helpers around the generated bindings. The Rust types and the commands
// come from bindings.ts, which tauri-specta writes from src-tauri (see
// `commands()` there); this file adds what Rust doesn't provide.

import { Channel, convertFileSrc } from "@tauri-apps/api/core";
import { commands as rustCommands } from "./bindings";
import type * as Rust from "./bindings";

/**
 * A Rust type as Rust sends it. Fields with `#[serde(default)]` may be left
 * out when the UI sends a value to Rust, so bindings.ts marks them optional
 * (`draft?: boolean`); but Rust always fills them in when it sends one.
 * `-?` removes the `?` from every field, all the way down. Lists, unions and
 * `null` stay as they are.
 */
export type Sent<T> = T extends object ? { [K in keyof T]-?: Sent<T[K]> } : T;

type RustCommands = typeof rustCommands;

/** The generated commands, resolving to `Sent` values. Only the types change. */
export const commands = rustCommands as {
  [K in keyof RustCommands]: (
    ...args: Parameters<RustCommands[K]>
  ) => Promise<Sent<Awaited<ReturnType<RustCommands[K]>>>>;
};

export type { Library, Menu, MediaKind, Navigation, PageDate, SftpAuth, Side, Stage } from "./bindings";
export type Block = Sent<Rust.Block>;
export type BuildReport = Sent<Rust.BuildReport>;
export type DeletedPage = Sent<Rust.DeletedPage>;
export type DiffLine = Sent<Rust.DiffLine>;
export type HistoryStatus = Sent<Rust.HistoryStatus>;
export type PageContent = Sent<Rust.PageContent>;
export type Version = Sent<Rust.Version>;
export type FrontMatter = Sent<Rust.FrontMatter>;
export type Gallery = Sent<Rust.Gallery>;
export type MediaFile = Sent<Rust.MediaFile>;
export type Page = Sent<Rust.Page>;
export type PreviewInfo = Sent<Rust.PreviewInfo>;
export type Progress = Sent<Rust.Progress>;
export type PublishReport = Sent<Rust.PublishReport>;
export type SecretStatus = Sent<Rust.SecretStatus>;
export type Site = Sent<Rust.Site>;
export type SiteConfig = Sent<Rust.SiteConfig>;
export type Target = Sent<Rust.Target>;
export type TestOutcome = Sent<Rust.TestOutcome>;

/** Today's date as YYYY-MM-DD, in local time. */
export function today(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
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

/** The blocks a column can hold (see COLUMN_TYPES in block.rs). */
export const COLUMN_BLOCK_TYPES: BlockType[] = ["text", "image", "text-image", "download", "faq", "call-to-action"];

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
      return {
        type,
        items: [
          { title: null, blocks: [{ type: "text", markdown: "" }] },
          { title: null, blocks: [{ type: "text", markdown: "" }] },
        ],
      };
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
  // Blocks inside columns have the same fields as those on the page.
  const isBlock = topLevel || "type" in value;
  const entries = Object.entries(value).map(([key, field]) => {
    // Inside lists, only alt texts and column titles are optional.
    const optional = isBlock ? OPTIONAL.has(key) : key === "alt" || key === "title";
    if (optional && typeof field === "string" && field.trim() === "") return [key, null];
    return [key, normalize(field, false)];
  });
  return Object.fromEntries(entries);
}

/** A URL the app window can load a media file from (for thumbnails). */
export function mediaUrl(root: string, name: string): string {
  return convertFileSrc(`${root}/media/${name}`);
}

/** A version's time (seconds since 1970) in the computer's own format. */
export function formatTime(seconds: number): string {
  return new Date(seconds * 1000).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

/** "1.4 MB", "820 KB", "12 bytes". */
export function formatSize(bytes: number): string {
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${bytes} bytes`;
}

export function newSftpTarget(): Target {
  return { kind: "sftp", name: "", host: "", port: 22, user: "", path: "", auth: "password", host_key: null, delete_stale: false };
}

/**
 * Whether publishing needs a note: there's an open version to close or a
 * change since the last version (the same rule as `HistoryStatus::needs_note`
 * in Rust).
 */
export function needsNote(status: HistoryStatus): boolean {
  return status.tracked && (status.open || status.changes.length > 0);
}

/**
 * Builds and publishes; `onProgress` is called while it runs. `note` says
 * what's new, required when `needsNote`.
 */
export function publishSite(
  root: string,
  targetName: string,
  note: string | null,
  onProgress: (p: Progress) => void,
): Promise<PublishReport> {
  const channel = new Channel<Progress>();
  channel.onmessage = onProgress;
  return commands.publishSite(root, targetName, note, channel);
}

let iconCache: Promise<Map<string, string>> | null = null;

/** Page icons by name (SVG markup), loaded once from Rust. */
export function pageIcons(): Promise<Map<string, string>> {
  iconCache ??= commands.listIcons().then((list) => new Map(list));
  return iconCache;
}
