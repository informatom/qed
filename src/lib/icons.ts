// Lucide icons (https://lucide.dev, ISC license). Each icon is imported on
// its own, so only the ones used end up in the app.
import type { Component as IconComponent } from "svelte";
import CircleQuestionMark from "@lucide/svelte/icons/circle-question-mark";
import Columns3 from "@lucide/svelte/icons/columns-3";
import Component from "@lucide/svelte/icons/component";
import Download from "@lucide/svelte/icons/download";
import Image from "@lucide/svelte/icons/image";
import Images from "@lucide/svelte/icons/images";
import List from "@lucide/svelte/icons/list";
import ListTree from "@lucide/svelte/icons/list-tree";
import SquarePlay from "@lucide/svelte/icons/square-play";
import Megaphone from "@lucide/svelte/icons/megaphone";
import Newspaper from "@lucide/svelte/icons/newspaper";
import PanelTop from "@lucide/svelte/icons/panel-top";
import Type from "@lucide/svelte/icons/type";
import Mic from "@lucide/svelte/icons/mic";
import AudioLines from "@lucide/svelte/icons/audio-lines";
import Captions from "@lucide/svelte/icons/captions";
import ListOrdered from "@lucide/svelte/icons/list-ordered";
import NotebookText from "@lucide/svelte/icons/notebook-text";
import Share2 from "@lucide/svelte/icons/share-2";
import Users from "@lucide/svelte/icons/users";
import Rss from "@lucide/svelte/icons/rss";
import UsersRound from "@lucide/svelte/icons/users-round";
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
