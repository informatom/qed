// Helpers around the generated bindings. The Rust types and the commands
// come from bindings.ts, which tauri-specta writes from src-tauri (see
// `commands()` there); this file adds what Rust doesn't provide.

import { Channel, convertFileSrc } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
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

export type { AudioFiles, AudioFormat, Role, EpisodeType, PodcastType, Corners, HeaderStyle, Library, Menu, MediaKind, Navigation, PageDate, SftpAuth, Side, Stage, Surface, ThemeColor } from "./bindings";
export type Colors = Sent<Rust.Colors>;
export type Block = Sent<Rust.Block>;
export type CustomTheme = Sent<Rust.CustomTheme>;
export type FontInfo = Sent<Rust.FontInfo>;
export type FontListing = Sent<Rust.FontListing>;
export type Fonts = Sent<Rust.Fonts>;
export type ThemeVariable = Sent<Rust.ThemeVariable>;
export type ThemeChoice = Sent<Rust.ThemeChoice>;
export type BuildReport = Sent<Rust.BuildReport>;
export type DeletedPage = Sent<Rust.DeletedPage>;
export type DiffLine = Sent<Rust.DiffLine>;
export type HistoryStatus = Sent<Rust.HistoryStatus>;
export type SincePublish = Sent<Rust.SincePublish>;
export type RemoteInfo = Sent<Rust.RemoteInfo>;
export type RemoteReport = Sent<Rust.RemoteReport>;
export type RemoteOutcome = Sent<Rust.RemoteOutcome>;
export type Transfer = Sent<Rust.Transfer>;
export type PageContent = Sent<Rust.PageContent>;
export type Version = Sent<Rust.Version>;
export type RestingVersion = Sent<Rust.RestingVersion>;
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
export type Podcast = Sent<Rust.Podcast>;
export type Episode = Sent<Rust.Episode>;
export type Chapter = Sent<Rust.Chapter>;
export type Funding = Sent<Rust.Funding>;
export type AudioInfo = Sent<Rust.AudioInfo>;
export type EpisodeAudio = Sent<Rust.EpisodeAudio>;
export type Contributor = Sent<Rust.Contributor>;
export type ContributorLink = Sent<Rust.ContributorLink>;
export type EpisodePerson = Sent<Rust.EpisodePerson>;
export type Target = Sent<Rust.Target>;
export type TestOutcome = Sent<Rust.TestOutcome>;

/** The address of the page a translation translates (`en/about` → `about`); a page's own otherwise (as `Page::main_path`). */
export function mainPath(page: Page): string {
  if (!page.language) return page.path;
  return page.path.slice(page.language.length).replace(/^\//, "");
}

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
  section: "Section",
  "call-to-action": "Call to action",
  "post-list": "Post list",
  "page-list": "Page list",
  video: "Video",
  shared: "Shared block",
  "latest-episode": "Latest episode",
  subscribe: "Subscribe",
  contributors: "Contributors",
  "episode-player": "Player",
  "episode-people": "People",
  "episode-shownotes": "Shownotes",
  "episode-chapters": "Chapters",
  "episode-transcript": "Transcript",
  "episode-share": "Share links",
};

/** Blocks only podcast sites offer. */
export const PODCAST_BLOCK_TYPES: BlockType[] = ["latest-episode", "subscribe", "contributors"];

/** The parts of an episode page, only for the episode layout (`episode.md`). */
export const EPISODE_PART_TYPES: BlockType[] = [
  "episode-player",
  "episode-people",
  "episode-shownotes",
  "episode-chapters",
  "episode-transcript",
  "episode-share",
];

/** What each part of an episode page shows. */
export const EPISODE_PART_NOTES: Partial<Record<BlockType, string>> = {
  "episode-player": "The Podlove Web Player with chapters and transcript; without JavaScript, the browser's own player. Below it, the duration and download links.",
  "episode-people": "Everyone in the episode (Settings → People), with photo and roles, each linking to their page.",
  "episode-shownotes": "The episode page's own blocks.",
  "episode-chapters": "The chapter list, with times. Hidden while the Podlove Web Player runs, which lists them itself.",
  "episode-transcript": "The transcript as text, folded away until a visitor opens it. Search engines read it too.",
  "episode-share": "Links for sharing the episode: Mastodon, Bluesky, email, copy link. Needs the site's web address.",
};

/** The episode layout without `episode.md` (DEFAULT_EPISODE_LAYOUT in block.rs). */
export function defaultEpisodeLayout(): Block[] {
  return (["episode-player", "episode-people", "episode-shownotes", "episode-chapters"] as const).map(
    (type) => newBlock(type),
  );
}

/** The blocks every page offers: all but the episode parts. */
export const PAGE_BLOCK_TYPES: BlockType[] = (Object.keys(BLOCK_NAMES) as BlockType[]).filter(
  (type) => !EPISODE_PART_TYPES.includes(type),
);

/** The blocks a column can hold (see COLUMN_TYPES in block.rs). */
export const COLUMN_BLOCK_TYPES: BlockType[] = [
  "text",
  "image",
  "text-image",
  "download",
  "faq",
  "call-to-action",
  "post-list",
  "page-list",
  "video",
  "shared",
  "latest-episode",
  "subscribe",
  "contributors",
];

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
          { title: null, width: null, boxed: false, blocks: [{ type: "text", markdown: "" }] },
          { title: null, width: null, boxed: false, blocks: [{ type: "text", markdown: "" }] },
        ],
      };
    case "section":
      return { type, title: null, background: null, image: null, spacing: null, blocks: [{ type: "text", markdown: "" }] };
    case "call-to-action":
      return { type, headline: null, text: null, button_label: null, button_link: null };
    case "post-list":
      return { type, from: null, limit: null, layout: null, contributor: null };
    case "page-list":
      return { type, from: null, layout: null };
    case "video":
      return { type, url: null, file: null, title: null, image: null };
    case "shared":
      return { type, name: null };
    case "latest-episode":
    case "contributors":
    case "episode-player":
    case "episode-people":
    case "episode-shownotes":
    case "episode-chapters":
    case "episode-transcript":
    case "episode-share":
      // A block without settings. bindings.ts adds `markdown?: never` to
      // it, which `Sent` turns into a required field no value can fill;
      // the cast says the block is complete as it is.
      return { type } as Block;
    case "subscribe":
      // Centered: it usually stands on its own.
      return { type, align: "center" };
  }
}

/** Fields that may be empty in a file. Everything else is always text. */
const OPTIONAL = new Set(["image", "alt", "headline", "text", "caption", "button_label", "button_link", "file", "label", "heading", "title", "from", "url", "layout", "name", "align", "contributor", "background", "spacing"]);

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

/** A shared block's name: its file in `shared/` without `.md`. */
export function sharedName(piece: Page): string {
  return piece.file.replace(/^shared\//, "").replace(/\.md$/, "");
}

/** A URL the app window can load a media file from (for thumbnails). */
export function mediaUrl(root: string, name: string): string {
  return libraryUrl(root, "media", name);
}

/** A page's URL path for people: `/about`, or "the footer" for the site footer. */
export function pageLabel(path: string): string {
  if (path === "(footer)") return "the footer";
  if (path === "(episode layout)") return "the episode layout";
  if (path === "(menu)") return "the menu";
  if (path.startsWith("(shared) ")) return `the shared block “${path.slice(9)}”`;
  if (path.startsWith("(contributor) ")) return `${path.slice(14)}'s photo`;
  return `/${path}`;
}

/** A file in one of the site's folders (`media/`, `assets/`, `static/`), for the webview. */
export function libraryUrl(root: string, library: Rust.Library, name: string): string {
  return convertFileSrc(`${root}/${library}/${name}`);
}

export type SaveOutcome = Sent<Rust.SaveOutcome>;

const KEEP_MINE = "Keep mine";
const TAKE_FILES = "Take the file's";

/**
 * Saves part of a page (`what`: "content" or "settings") only if nobody
 * changed that part outside the app since it was opened: an AI assistant,
 * a text editor, another computer (#105). `save(true)` saves with that
 * check, `save(false)` without. If it did change, asks which to keep.
 * Returns the page as saved, or as the file has it ("Take the file's"),
 * with `replaced` true then; `null` if the person cancelled.
 */
export async function saveChecked(
  save: (check: boolean) => Promise<SaveOutcome>,
  what: string,
): Promise<{ page: Page; replaced: boolean } | null> {
  const outcome = await save(true);
  if (outcome.outcome === "saved") return { page: outcome.page, replaced: false };
  const choice = await message(
    `The page's ${what} changed outside q.e.d. since you opened it, by an AI assistant, another program or by hand.\n\n` +
      `“${KEEP_MINE}” saves your version over those changes. “${TAKE_FILES}” shows the page as the file has it now; ` +
      "your unsaved changes here are lost.",
    { title: "Changed outside q.e.d.", kind: "warning", buttons: { yes: KEEP_MINE, no: TAKE_FILES, cancel: "Cancel" } },
  );
  if (choice === KEEP_MINE || choice === "Yes") {
    const forced = await save(false);
    return { page: forced.page, replaced: false };
  }
  if (choice === TAKE_FILES || choice === "No") return { page: outcome.page, replaced: true };
  return null;
}

/** How a version is listed: an open one with when it started, and its last save. */
export function versionLabel(version: Version): { title: string; time: string } {
  return version.open
    ? { title: `${version.summary} since ${formatTime(version.started)}`, time: `last saved ${formatTime(version.time)}` }
    : { title: version.summary, time: formatTime(version.time) };
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

/**
 * Whether two values hold the same data, whatever order their keys are in
 * (`{ a, b }` and `{ b, a }` are the same); keys set to `undefined` count
 * as missing. For "unsaved changes" checks against what Rust sent.
 */
export function sameData(a: unknown, b: unknown): boolean {
  const sorted = (_key: string, value: unknown) =>
    value && typeof value === "object" && !Array.isArray(value)
      ? Object.fromEntries(Object.entries(value).sort(([x], [y]) => (x < y ? -1 : x > y ? 1 : 0)))
      : value;
  return JSON.stringify(a, sorted) === JSON.stringify(b, sorted);
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

/** A channel for push, pull and "get a site" progress; `onProgress` gets each step. */
export function transferChannel(onProgress: (t: Transfer) => void = () => {}): Channel<Transfer> {
  const channel = new Channel<Transfer>();
  channel.onmessage = onProgress;
  return channel;
}

const iconCache = new Map<string, string | null>();

/** The SVG markup of these icons by name, asking Rust once for each. */
export async function iconSvgs(names: string[]): Promise<Map<string, string>> {
  const missing = [...new Set(names)].filter((name) => !iconCache.has(name));
  if (missing.length > 0) {
    const found = new Map(await commands.iconSvgs(missing));
    for (const name of missing) iconCache.set(name, found.get(name) ?? null);
  }
  const result = new Map<string, string>();
  for (const name of names) {
    const svg = iconCache.get(name);
    if (svg) result.set(name, svg);
  }
  return result;
}

/** Icons for a search, best first: names and SVG markup. */
export function searchIcons(query: string, limit = 120): Promise<[string, string][]> {
  return commands.searchIcons(query, limit);
}

/** How people call an audio format, in the order the Podcast tab offers them. */
export const AUDIO_FORMATS: { value: Rust.AudioFormat; label: string; everywhere: boolean }[] = [
  { value: "mp3", label: "MP3", everywhere: true },
  { value: "m4a", label: "M4A", everywhere: true },
  { value: "ogg", label: "Ogg", everywhere: false },
  { value: "opus", label: "Opus", everywhere: false },
  { value: "flac", label: "FLAC", everywhere: false },
];

export function formatLabel(format: Rust.AudioFormat): string {
  return AUDIO_FORMATS.find((f) => f.value === format)?.label ?? format;
}

/** The podcast's formats, the main one first; MP3 if none are set (as Rust has it). */
export function podcastFormats(podcast: { formats?: Rust.AudioFormat[] } | null): Rust.AudioFormat[] {
  return podcast?.formats?.length ? podcast.formats : ["mp3"];
}

/** The roles someone can have, as the Podcasting 2.0 taxonomy names them (see contributors.rs). */
export const ROLES: { value: Rust.Role; label: string }[] = [
  { value: "host", label: "Host" },
  { value: "co-host", label: "Co-host" },
  { value: "guest-host", label: "Guest host" },
  { value: "guest", label: "Guest" },
  { value: "narrator", label: "Narrator" },
  { value: "producer", label: "Producer" },
  { value: "audio-editor", label: "Audio editor" },
  { value: "sound-designer", label: "Sound designer" },
  { value: "composer", label: "Composer" },
  { value: "transcriber", label: "Transcriber" },
  { value: "cover-art-designer", label: "Cover art designer" },
];
