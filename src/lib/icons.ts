// Tabler icons (https://tabler.io/icons, MIT license). Each icon is imported on
// its own, so only the ones used end up in the app.
import type { Component as IconComponent } from "svelte";
import CircleQuestionMark from "@tabler/icons-svelte-runes/icons/help-circle";
import Columns3 from "@tabler/icons-svelte-runes/icons/columns-3";
import Component from "@tabler/icons-svelte-runes/icons/components";
import Download from "@tabler/icons-svelte-runes/icons/download";
import Group from "@tabler/icons-svelte-runes/icons/section";
import Image from "@tabler/icons-svelte-runes/icons/photo";
import Images from "@tabler/icons-svelte-runes/icons/library-photo";
import List from "@tabler/icons-svelte-runes/icons/list";
import ListTree from "@tabler/icons-svelte-runes/icons/list-tree";
import SquarePlay from "@tabler/icons-svelte-runes/icons/movie";
import Megaphone from "@tabler/icons-svelte-runes/icons/speakerphone";
import Newspaper from "@tabler/icons-svelte-runes/icons/news";
import PanelTop from "@tabler/icons-svelte-runes/icons/layout-navbar";
import Type from "@tabler/icons-svelte-runes/icons/typography";
import Mic from "@tabler/icons-svelte-runes/icons/microphone";
import AudioLines from "@tabler/icons-svelte-runes/icons/wave-sine";
import Captions from "@tabler/icons-svelte-runes/icons/badge-cc";
import ListOrdered from "@tabler/icons-svelte-runes/icons/list-numbers";
import NotebookText from "@tabler/icons-svelte-runes/icons/notebook";
import Share2 from "@tabler/icons-svelte-runes/icons/share";
import Users from "@tabler/icons-svelte-runes/icons/users";
import Rss from "@tabler/icons-svelte-runes/icons/rss";
import UsersRound from "@tabler/icons-svelte-runes/icons/users-group";
import type { BlockType } from "$lib/site";

/** One icon per block type, for the Add row and the block cards. */
export const BLOCK_ICONS: Record<BlockType, IconComponent> = {
  text: Type,
  hero: PanelTop,
  image: Image,
  "text-image": Newspaper,
  gallery: Images,
  download: Download,
  faq: CircleQuestionMark,
  columns: Columns3,
  section: Group,
  "call-to-action": Megaphone,
  "post-list": List,
  "page-list": ListTree,
  video: SquarePlay,
  shared: Component,
  "latest-episode": Mic,
  subscribe: Rss,
  contributors: UsersRound,
  "episode-player": AudioLines,
  "episode-people": Users,
  "episode-shownotes": NotebookText,
  "episode-chapters": ListOrdered,
  "episode-transcript": Captions,
  "episode-share": Share2,
};
