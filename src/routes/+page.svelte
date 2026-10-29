<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronsDownUp from "@lucide/svelte/icons/chevrons-down-up";
  import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";
  import Eye from "@lucide/svelte/icons/eye";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import FilePlus from "@lucide/svelte/icons/file-plus";
  import Mic from "@lucide/svelte/icons/mic";
  import UsersRound from "@lucide/svelte/icons/users-round";
  import ContributorsPanel from "$lib/ContributorsPanel.svelte";
  import Files from "@lucide/svelte/icons/files";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import Globe from "@lucide/svelte/icons/globe";
  import Hammer from "@lucide/svelte/icons/hammer";
  import History from "@lucide/svelte/icons/history";
  import FileDown from "@lucide/svelte/icons/file-down";
  import AudioLines from "@lucide/svelte/icons/audio-lines";
  import Captions from "@lucide/svelte/icons/captions";
  import FolderTree from "@lucide/svelte/icons/folder-tree";
  import PanelBottom from "@lucide/svelte/icons/panel-bottom";
  import FooterEditor from "$lib/FooterEditor.svelte";
  import LayoutTemplate from "@lucide/svelte/icons/layout-template";
  import EpisodeLayoutEditor from "$lib/EpisodeLayoutEditor.svelte";
  import SharedEditor from "$lib/SharedEditor.svelte";
  import Component from "@lucide/svelte/icons/component";
  import Plus from "@lucide/svelte/icons/plus";
  import { provideSharedBlocks, usesShared } from "$lib/sharedBlocks";
  import Image from "@lucide/svelte/icons/image";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Settings from "@lucide/svelte/icons/settings";
  import MenuIcon from "@lucide/svelte/icons/menu";
  import Palette from "@lucide/svelte/icons/palette";
  import Upload from "@lucide/svelte/icons/upload";
  import AppSettings from "$lib/AppSettings.svelte";
  import ManualLink from "$lib/ManualLink.svelte";
  import LifeBuoy from "@lucide/svelte/icons/life-buoy";
  import { openManual } from "$lib/manual";
  import MediaLibrary from "$lib/MediaLibrary.svelte";
  import SiteSettings from "$lib/SiteSettings.svelte";
  import PodcastPanel from "$lib/PodcastPanel.svelte";
  import SiteTheme from "$lib/SiteTheme.svelte";
  import MenuEditor from "$lib/MenuEditor.svelte";
  import VersionsPanel from "$lib/VersionsPanel.svelte";
  import RestingVersionDialog from "$lib/RestingVersionDialog.svelte";
  import NewPageDialog from "$lib/NewPageDialog.svelte";
  import NewSiteDialog from "$lib/NewSiteDialog.svelte";
  import CloneSiteDialog from "$lib/CloneSiteDialog.svelte";
  import PullBar from "$lib/PullBar.svelte";
  import MovePageDialog from "$lib/MovePageDialog.svelte";
  import PublishPanel from "$lib/PublishPanel.svelte";
  import PageEditor from "$lib/PageEditor.svelte";
  import SiteSwitcher from "$lib/SiteSwitcher.svelte";
  // Vite turns an imported file into its URL.
  import logoMark from "$lib/assets/logo-mark.svg";
  import UpdateBar from "$lib/UpdateBar.svelte";
  import { checkForUpdate } from "$lib/updates.svelte";
  import X from "@lucide/svelte/icons/x";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import { edgeScroll, scrollParent } from "$lib/dragScroll";
  import { parentOf, planDrop, treeOrder, type DropPlan, type Zone } from "$lib/pageTree";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { mayDiscardUnsaved, trackUnsaved } from "$lib/unsaved.svelte";
  import { forgetSite, recentSites, rememberSite, reopenOnStart, setSiteClosed } from "$lib/recentSites";
  import {
    commands,
    BLOCK_NAMES,
    defaultEpisodeLayout,
    pageIcons,
    sharedName,
    pageLabel,
    formatLabel,
    podcastFormats,
    type AudioFormat,
    type BuildReport,
    type Page,
    type Site,
  } from "$lib/site";

  /** Site folders opened before, newest first. */
  let recent = $state(recentSites());

  let site = $state<Site | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let building = $state(false);
  // The page whose settings are open, by file (files don't change on save).
  let selectedFile = $state<string | null>(null);
  let selected = $derived(site?.pages.find((p) => p.file === selectedFile) ?? null);
  // The site footer opens like a page, by its file name.
  const FOOTER_FILE = "footer.md";
  let editingFooter = $derived(selectedFile === FOOTER_FILE);
  // And the episode layout, on podcast sites.
  const EPISODE_LAYOUT_FILE = "episode.md";
  let editingEpisodeLayout = $derived(selectedFile === EPISODE_LAYOUT_FILE && !!site?.config.podcast);
  // So do shared blocks, by their file in shared/.
  let selectedShared = $derived(site?.shared.find((p) => p.file === selectedFile) ?? null);
  let panelOpen = $derived(selected !== null || editingFooter || editingEpisodeLayout || selectedShared !== null);

  /** The pages (the footer, the episode layout) that show the shared block `name`. */
  function sharedUsers(name: string): string[] {
    if (!site) return [];
    return [...site.pages, ...(site.footer ? [site.footer] : []), ...(site.episode_layout ? [site.episode_layout] : [])]
      .filter((page) => usesShared(page.blocks, name))
      .map((page) => page.path);
  }

  /** Puts a new or saved shared block into the list, in name order. */
  function replaceShared(piece: Page) {
    if (!site) return;
    site.shared = [...site.shared.filter((p) => p.file !== piece.file), piece].sort((a, b) => a.file.localeCompare(b.file));
  }

  provideSharedBlocks({
    get pieces() {
      return site?.shared ?? [];
    },
    insidePiece: false,
    get podcast() {
      return !!site?.config.podcast;
    },
    get contributors() {
      return site?.contributors ?? [];
    },
    open: (name) => select(`shared/${name}.md`),
    async make(title, block) {
      if (!site) throw new Error("no site open");
      const piece = await commands.createShared(site.root, title, [block]);
      replaceShared(piece);
      return sharedName(piece);
    },
  });

  // The name typed for a new shared block in the page list, while open.
  let newSharedTitle = $state<string | null>(null);
  let newSharedError = $state<string | null>(null);

  async function createShared() {
    if (!site || !newSharedTitle?.trim()) return;
    newSharedError = null;
    try {
      const piece = await commands.createShared(site.root, newSharedTitle, []);
      replaceShared(piece);
      newSharedTitle = null;
      await select(piece.file);
    } catch (e) {
      newSharedError = String(e);
    }
  }
  // True while the open page has unsaved changes.
  let editorDirty = $state(false);

  /** The open page's file is gone: close it and read the site again. */
  async function pageDeleted() {
    editorDirty = false;
    selectedFile = null;
    if (site) await load(site.root);
  }

  /** Opens another page (or none), after asking if edits would be lost. */
  async function select(file: string | null) {
    if (file === selectedFile) return;
    if (editorDirty) {
      const discard = await ask("The open page has unsaved changes. Discard them?", {
        title: "Unsaved changes",
        kind: "warning",
        okLabel: "Discard",
        cancelLabel: "Keep editing",
      });
      if (!discard) return;
    }
    editorDirty = false;
    selectedFile = file;
  }

  let creatingPage = $state(false);
  let creatingEpisode = $state(false);
  let creatingSite = $state(false);
  let cloningSite = $state(false);
  let movingPage = $state(false);
  let view = $state<"pages" | "media" | "assets" | "static" | "audio" | "transcripts" | "podcast" | "contributors" | "site" | "menu" | "theme" | "publish" | "versions" | "app">("pages");

  async function pageCreated(page: Page) {
    if (!site) return;
    // Reload, so the new page lands in its place in the tree, then open it.
    await load(site.root);
    selectedFile = page.file;
  }

  async function pageMoved(page: Page) {
    if (!site) return;
    // Other pages changed too (links, redirects): read the whole site again.
    await load(site.root);
    selectedFile = page.file;
  }

  function replacePage(saved: Page) {
    if (!site) return;
    const index = site.pages.findIndex((p) => p.file === saved.file);
    if (index >= 0) site.pages[index] = saved;
  }
  let report = $state<BuildReport | null>(null);
  let previewUrl = $state<string | null>(null);

  // A build report is good news, not something to keep: it goes away by
  // itself after 10 seconds. Each new report starts its own timer; the
  // function returned here cancels the old one.
  $effect(() => {
    // One with notes stays until it's closed, so the notes can be read.
    if (!report || report.notes.length) return;
    const timer = setTimeout(() => (report = null), 10_000);
    return () => clearTimeout(timer);
  });

  // Podcasts keep their audio files in audio/, unless they're elsewhere.
  let hasAudioLibrary = $derived(!!site?.config.podcast && site.config.podcast.audio_files !== "external");
  $effect(() => {
    if (view === "audio" && !hasAudioLibrary) view = "pages";
    if ((view === "contributors" || view === "transcripts") && !site?.config.podcast) view = "pages";
  });

  // Derived values update by themselves when `site` changes.
  let draftCount = $derived(site ? site.pages.filter((p) => p.front.draft).length : 0);

  /** The podcast's formats a published episode has no audio for; Publish waits for them. */
  function missingAudio(page: Page): AudioFormat[] {
    const episode = page.front.episode;
    if (!site?.config.podcast || !episode || page.front.draft || !page.path.startsWith("episodes/")) return [];
    return podcastFormats(site.config.podcast).filter((f) => !episode.audio.some((a) => a.format === f));
  }

  async function load(path: string) {
    loading = true;
    error = null;
    try {
      site = await commands.openSite(path);
      report = null;
      const files = [FOOTER_FILE, EPISODE_LAYOUT_FILE, ...site.pages.map((p) => p.file), ...site.shared.map((p) => p.file)];
      if (selectedFile !== null && !files.includes(selectedFile)) selectedFile = null;
      recent = rememberSite(path, site.config.title);
      setSiteClosed(false);
    } catch (e) {
      site = null;
      error = String(e);
      // Every tab but App needs a site; show the error on Pages instead.
      if (view !== "pages" && view !== "app") view = "pages";
    } finally {
      loading = false;
    }
  }

  // Pages can change outside the app: an AI assistant through qed-mcp, a
  // text editor, a pull on another computer (#105). When the window comes
  // back into focus, read the site again, quietly, unless a page has
  // unsaved changes (saving then asks, see `saveChecked`). Only on Pages:
  // other tabs keep forms of their own.
  async function refreshOnFocus() {
    if (!site || loading || editorDirty || view !== "pages") return;
    const root = site.root;
    try {
      const fresh = await commands.openSite(root);
      if (!site || site.root !== root || editorDirty) return;
      if (JSON.stringify(fresh) === JSON.stringify(site)) return;
      site = fresh;
      const files = [FOOTER_FILE, EPISODE_LAYOUT_FILE, ...fresh.pages.map((p) => p.file), ...fresh.shared.map((p) => p.file)];
      if (selectedFile !== null && !files.includes(selectedFile)) selectedFile = null;
      // The open editor starts again from the page as it is now.
      reloads++;
    } catch {
      // The next save or a manual reload shows what's wrong.
    }
  }

  async function build() {
    if (!site) return;
    building = true;
    error = null;
    try {
      report = await commands.buildSite(site.root);
    } catch (e) {
      error = String(e);
    } finally {
      building = false;
    }
  }

  // What the preview opens: the page selected in the Pages tab, else the
  // start page. Drafts aren't built, so they open the start page too.
  let previewPage = $derived(
    view === "pages" && selected && !selected.front.draft ? selected : null,
  );
  let previewTip = $derived(
    previewPage ? `Preview “${previewPage.front.title}” in browser` : "Preview in browser",
  );

  async function preview() {
    if (!site) return;
    building = true;
    error = null;
    try {
      const info = await commands.previewSite(site.root);
      report = info.report;
      previewUrl = info.url;
      const path = previewPage?.path ?? "";
      await openUrl(path ? `${info.url.replace(/\/$/, "")}/${path}/` : info.url);
    } catch (e) {
      error = String(e);
    } finally {
      building = false;
    }
  }

  async function chooseFolder() {
    const path = await open({ directory: true, title: "Open a site folder" });
    if (typeof path === "string") await switchTo(path);
  }

  /** Opens another site, after asking if unsaved edits would be lost. */
  async function switchTo(path: string) {
    if (site && site.root !== path) {
      // Every editor of the old site goes away.
      if (!(await mayDiscardUnsaved("Open another site"))) return;
      editorDirty = false;
    } else if (!(await mayLeavePage())) return;
    if (site && site.root !== path) {
      selectedFile = null;
      unfolded.clear();
    }
    await load(path);
  }

  /** Asks before unsaved edits in the open page would be lost. */
  async function mayLeavePage(): Promise<boolean> {
    if (!editorDirty) return true;
    const discard = await ask("The open page has unsaved changes. Discard them?", {
      title: "Unsaved changes",
      kind: "warning",
      okLabel: "Discard",
      cancelLabel: "Keep editing",
    });
    if (discard) editorDirty = false;
    return discard;
  }

  /** Back to the start screen. The site stays in the recent list, and the
      next start shows the start screen too, until a site is opened. */
  async function closeSite() {
    if (!(await mayDiscardUnsaved("Close the site"))) return;
    editorDirty = false;
    site = null;
    selectedFile = null;
    report = null;
    error = null;
    if (view !== "app") view = "pages";
    setSiteClosed(true);
  }

  // The open editor counts as unsaved while it has changes, by its name.
  trackUnsaved(
    () =>
      selectedShared
        ? `the shared block “${selectedShared.front.title}”`
        : editingFooter
          ? "the footer"
          : editingEpisodeLayout
            ? "the episode layout"
            : `the page “${selected?.front.title ?? ""}”`,
    () => editorDirty,
  );

  // Closing the window asks first when changes would be lost. Tauri waits
  // for this handler, then closes the window unless it was prevented.
  onMount(() => {
    const stop = getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await mayDiscardUnsaved("Close q.e.d."))) event.preventDefault();
    });
    return () => {
      stop.then((unlisten) => unlisten());
    };
  });

  // Reopen the last site once, when the page first appears.
  // Page icons (SVG by name), for the page list.
  let icons = $state(new Map<string, string>());
  onMount(async () => {
    icons = await pageIcons();
  });

  // Look for a newer qed once at start; not while developing, where the
  // version in Cargo.toml says little about what's running.
  onMount(() => {
    if (!import.meta.env.DEV) checkForUpdate(true);
  });

  onMount(() => {
    const path = recent[0]?.path;
    if (path && reopenOnStart()) load(path);
  });

  function depth(path: string): number {
    return path === "" ? 0 : path.split("/").length;
  }

  // The page tree folds: pages below another page show only once it's
  // unfolded. All folded at first, since a site may have hundreds of pages.
  const unfolded = new SvelteSet<string>();

  /** How many pages lie below each page that has any (all levels). */
  let below = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const page of site?.pages ?? []) {
      for (const parent of parents(page.path)) counts.set(parent, (counts.get(parent) ?? 0) + 1);
    }
    return counts;
  });

  let paths = $derived(new Set(site?.pages.map((p) => p.path)));

  /** The paths above a page that are pages themselves: `a/b/c` → `a`, `a/b`. */
  function parents(path: string): string[] {
    const parts = path.split("/");
    return parts
      .slice(0, -1)
      .map((_, i) => parts.slice(0, i + 1).join("/"))
      .filter((parent) => paths.has(parent));
  }

  // In the order the site shows them (weight, then title; posts by date).
  let ordered = $derived(site ? treeOrder(site.pages) : []);
  let shownPages = $derived(ordered.filter((page) => parents(page.path).every((p) => unfolded.has(p))));

  // Dragging a page by its handle, as blocks are dragged (BlockList).
  // Dropped between pages, it changes the order (weights); dropped onto a
  // page, it moves below it, which changes its address.
  let rows = $state<HTMLElement[]>([]);
  let pageDrag = $state<{ page: Page; row: number; zone: Zone; plan: DropPlan | null } | null>(null);
  let treeY = 0;
  let treeScroller: HTMLElement | null = null;
  // Re-creates the open page's editor after its order changed on disk.
  let reorders = $state(0);
  /** Counts quiet reloads after changes outside the app (`refreshOnFocus`). */
  let reloads = $state(0);

  function treeDragStart(event: PointerEvent, page: Page) {
    if (event.button !== 0) return;
    event.preventDefault();
    const handle = event.currentTarget as HTMLElement;
    handle.setPointerCapture(event.pointerId);
    pageDrag = { page, row: -1, zone: "onto", plan: null };
    treeY = event.clientY;
    treeScroller = scrollParent(handle);
    requestAnimationFrame(treeAutoScroll);
  }

  function treeDragMove(event: PointerEvent) {
    if (!pageDrag) return;
    treeY = event.clientY;
    findDrop();
  }

  function treeAutoScroll() {
    if (!pageDrag) return;
    if (edgeScroll(treeScroller, treeY)) findDrop();
    requestAnimationFrame(treeAutoScroll);
  }

  // The row under the pointer; its top and bottom third mean the gap
  // above or below it, the middle the page itself.
  function findDrop() {
    if (!pageDrag) return;
    const boxes = rows.slice(0, shownPages.length).map((row) => row.getBoundingClientRect());
    if (!boxes.length) return;
    let row = boxes.findIndex((box) => treeY >= box.top && treeY < box.bottom);
    let zone: Zone;
    if (row < 0) {
      row = treeY < boxes[0].top ? 0 : boxes.length - 1;
      zone = row === 0 ? "before" : "after";
    } else {
      const share = (treeY - boxes[row].top) / boxes[row].height;
      zone = share < 0.3 ? "before" : share > 0.7 ? "after" : "onto";
    }
    const target = shownPages[row];
    const childrenShown = unfolded.has(target.path) && (below.get(target.path) ?? 0) > 0;
    pageDrag.row = row;
    pageDrag.zone = zone;
    pageDrag.plan = planDrop(ordered, pageDrag.page, target, zone, childrenShown);
  }

  async function treeDragEnd() {
    const dropped = pageDrag;
    pageDrag = null;
    if (dropped?.plan) await applyDrop(dropped.page, dropped.plan);
  }

  async function applyDrop(page: Page, plan: DropPlan) {
    if (!site || !(await mayLeavePage())) return;
    const root = site.root;
    let file = page.file;
    try {
      if (plan.moveTo !== null) {
        const slug = page.path.slice(page.path.lastIndexOf("/") + 1);
        const to = plan.moveTo ? `${plan.moveTo}/${slug}` : slug;
        const along = below.get(page.path) ?? 0;
        const sure = await ask(
          `Move “${page.front.title}” from ${pageLabel(page.path)} to /${to}?` +
            (along ? ` The ${along} ${along === 1 ? "page" : "pages"} below it move along.` : "") +
            " Links to it are updated, and the old address redirects to the new one.",
          { title: "Move page", kind: "info", okLabel: "Move", cancelLabel: "Cancel" },
        );
        if (!sure) return;
        file = (await commands.movePage(root, page.file, plan.moveTo, slug)).file;
      }
      if (plan.order) await commands.setPageOrder(root, plan.order.map((f) => f ?? file));
      const wasOpen = selectedFile === page.file;
      editorDirty = false;
      await load(root);
      if (wasOpen) selectedFile = file;
      if (plan.moveTo) [...parents(plan.moveTo), plan.moveTo].forEach((p) => unfolded.add(p));
      reorders += 1;
    } catch (e) {
      error = String(e);
      await load(root);
    }
  }

  /** How far a line for the gap among the pages below `parent` is indented. */
  function gapIndent(parent: string): string {
    return `${(parent === "" ? 0 : depth(parent)) * 1.25 + 1.5}rem`;
  }

  function toggle(path: string) {
    if (unfolded.has(path)) unfolded.delete(path);
    else unfolded.add(path);
  }

  // A page opened some other way (new, moved) shows with its parents unfolded.
  // Only a new selection does this; folding a parent afterwards stays allowed.
  $effect(() => {
    const path = selected?.path;
    if (path !== undefined) untrack(() => parents(path).forEach((parent) => unfolded.add(parent)));
  });

  /** Counts versions closed outside the Versions tab, so it reloads. */
  let historyChanges = $state(0);
</script>

<svelte:window onkeydown={(e) => pageDrag && e.key === "Escape" && (pageDrag = null)} onfocus={refreshOnFocus} />

<main class="flex min-h-screen flex-col gap-8 p-8">
  <UpdateBar />
  {#if site}
    <PullBar siteRoot={site.root} editing={editorDirty} onpulled={() => site && load(site.root)} />
    <RestingVersionDialog siteRoot={site.root} onclosed={() => historyChanges++} />
  {/if}
  <header class="flex flex-wrap items-start justify-between gap-4">
    {#if site}
      <div class="flex min-w-0 items-center gap-4">
        <img id="app-logo" src={logoMark} alt="" class="size-14 shrink-0" />
        <div id="site-summary" class="flex min-w-0 flex-col gap-1">
          <div class="flex flex-wrap items-baseline gap-x-3 text-sm text-base-content/60">
            <span class="text-xs font-semibold tracking-widest text-primary uppercase">q.e.d.</span>
            <span>
              <span id="site-root" class="font-mono">{site.root}</span> · {site.pages.length} pages{draftCount
                ? `, ${draftCount} drafts`
                : ""} · language {site.config.language} · theme {site.config.theme}
            </span>
          </div>
          <div class="flex flex-wrap items-baseline gap-x-4">
            <SiteSwitcher
                title={site.config.title}
                root={site.root}
                {recent}
                onopen={switchTo}
                onchoose={chooseFolder}
                onnew={() => (creatingSite = true)}
                onclone={() => (cloningSite = true)}
                onforget={(path) => (recent = forgetSite(path))}
              onclose={closeSite}
              />
            {#if site.config.description}
              <p class="text-base-content/70">{site.config.description}</p>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <h1 class="flex items-center gap-3 text-3xl font-bold tracking-tight">
        <img id="app-logo" src={logoMark} alt="" class="size-12" /> q.e.d.
      </h1>
    {/if}
    <!-- Keeps the room the preview button had in the header; the button
         itself floats in the window's corner, so it stays in reach while
         scrolling. -->
    <div class="h-10 w-22 shrink-0" aria-hidden="true"></div>
    <div class="fixed top-8 right-8 z-40 flex gap-2">
      <div class="tooltip tooltip-left" data-tip="Manual">
        <button id="open-manual" class="btn btn-circle shadow-lg" aria-label="Manual"
          onclick={() => openManual().catch((e) => (error = String(e)))}>
          <LifeBuoy class="size-5" />
        </button>
      </div>
      {#if site}
        <!-- An icon is enough here; the tooltip and aria-label say what it does. -->
        <div class="tooltip tooltip-left" data-tip={previewTip}>
          <button id="preview-site" class="btn btn-circle btn-secondary shadow-lg" onclick={preview} disabled={loading || building}
            aria-label={previewTip}>
            {#if building}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              <Eye class="size-5" />
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </header>

  {#if error}
    <div id="site-error" role="alert" class="alert alert-error alert-soft">
      <span class="font-mono text-sm whitespace-pre-wrap">{error}</span>
    </div>
  {/if}

  {#if report}
    <!-- A light green box with text in the normal color: green on green is hard to read. -->
    <div id="build-report" role="status" class="alert items-start border-success/40 bg-success/10 text-base-content">
      <span class="flex flex-1 flex-col gap-0.5">
        <span>
          Built {report.pages} pages{report.drafts_skipped ? ` (${report.drafts_skipped} drafts skipped)` : ""},
          {report.redirects} redirects, {report.media_files} media files, {report.resized_images} resized images{report.static_files
            ? `, ${report.static_files} static files`
            : ""}{report.feeds
            ? ` and ${report.feeds === 1 ? "1 feed" : `${report.feeds} feeds`}`
            : ""}.{report.searchable_pages
            ? ` ${report.searchable_pages === 1 ? "1 page" : `${report.searchable_pages} pages`} can be searched.`
            : ""}
        </span>
        <span>
          Into <code class="font-mono text-sm">{report.out_dir}</code> ·
          <button class="link" onclick={() => report && revealItemInDir(report.out_dir)}>Show in file manager</button>
        </span>
        {#if previewUrl}
          <span>Preview: <button class="link" onclick={() => previewUrl && openUrl(previewUrl)}>{previewUrl}</button></span>
        {/if}
        {#if report.notes.length}
          <ul id="build-notes" class="mt-2 flex flex-col gap-1">
            {#each report.notes as note (note)}
              <li class="flex items-start gap-2"><TriangleAlert class="mt-0.5 size-4 shrink-0 text-warning" />{note}</li>
            {/each}
          </ul>
        {/if}
      </span>
      <button type="button" class="btn btn-ghost btn-xs btn-square" aria-label="Close" onclick={() => (report = null)}>✕</button>
    </div>
  {/if}

  <div role="tablist" class="tabs tabs-border -mb-4">
    <button type="button" role="tab" id="view-pages" class={["tab", view === "pages" && "tab-active"]}
      onclick={() => (view = "pages")}><Files class="me-1.5 size-4" />Pages</button>
    {#if site}
      <button type="button" role="tab" id="view-media" class={["tab", view === "media" && "tab-active"]}
        onclick={() => (view = "media")}><Image class="me-1.5 size-4" />Media</button>
      <button type="button" role="tab" id="view-assets" class={["tab", view === "assets" && "tab-active"]}
        onclick={() => (view = "assets")}><FileDown class="me-1.5 size-4" />Downloads</button>
      <button type="button" role="tab" id="view-static" class={["tab", view === "static" && "tab-active"]}
        onclick={() => (view = "static")}><FolderTree class="me-1.5 size-4" />Static</button>
      {#if hasAudioLibrary}
        <button type="button" role="tab" id="view-audio" class={["tab", view === "audio" && "tab-active"]}
          onclick={() => (view = "audio")}><AudioLines class="me-1.5 size-4" />Audio</button>
      {/if}
      {#if site.config.podcast}
        <button type="button" role="tab" id="view-transcripts" class={["tab", view === "transcripts" && "tab-active"]}
          onclick={() => (view = "transcripts")}><Captions class="me-1.5 size-4" />Transcripts</button>
      {/if}
      <button type="button" role="tab" id="view-site" class={["tab", view === "site" && "tab-active"]}
        onclick={() => (view = "site")}><Globe class="me-1.5 size-4" />Site</button>
      <button type="button" role="tab" id="view-podcast" class={["tab", view === "podcast" && "tab-active"]}
        onclick={() => (view = "podcast")}><Mic class="me-1.5 size-4" />Podcast</button>
      {#if site.config.podcast}
        <button type="button" role="tab" id="view-contributors" class={["tab", view === "contributors" && "tab-active"]}
          onclick={() => (view = "contributors")}><UsersRound class="me-1.5 size-4" />Contributors</button>
      {/if}
      <button type="button" role="tab" id="view-menu" class={["tab", view === "menu" && "tab-active"]}
        onclick={() => (view = "menu")}><MenuIcon class="me-1.5 size-4" />Menu</button>
      <button type="button" role="tab" id="view-theme" class={["tab", view === "theme" && "tab-active"]}
        onclick={() => (view = "theme")}><Palette class="me-1.5 size-4" />Theme</button>
      <button type="button" role="tab" id="view-publish" class={["tab", view === "publish" && "tab-active"]}
        onclick={() => (view = "publish")}><Upload class="me-1.5 size-4" />Publish</button>
      <button type="button" role="tab" id="view-versions" class={["tab", view === "versions" && "tab-active"]}
        onclick={() => (view = "versions")}><History class="me-1.5 size-4" />Versions</button>
    {/if}
    <button type="button" role="tab" id="view-app" class={["tab", view === "app" && "tab-active"]}
      onclick={() => (view = "app")}><Settings class="me-1.5 size-4" />App</button>
  </div>

  {#if view === "app"}
    <AppSettings />
  {/if}

  {#if site}
    <!-- Stays mounted while hidden, so unsaved settings survive a tab switch;
         re-created for another site. -->
    {#key site.root}
      <div class={["flex flex-col gap-8", view !== "site" && "hidden"]}>
        <div class="flex flex-wrap items-center gap-3 rounded-box bg-base-200 px-4 py-3">
          <span class="text-sm text-base-content/60">Folder</span>
          <code class="min-w-0 flex-1 truncate font-mono text-sm">{site.root}</code>
          <button id="reload-site" class="btn btn-ghost btn-sm" onclick={() => site && load(site.root)} disabled={loading}>
            <RefreshCw class="size-4" /> Reload
          </button>
          <button id="open-other-site" class="btn btn-sm" onclick={chooseFolder} disabled={loading}>
            <FolderOpen class="size-4" /> Open another site folder…
          </button>
          <button id="new-site-other" class="btn btn-sm" onclick={() => (creatingSite = true)} disabled={loading}>
            <FolderPlus class="size-4" /> New site…
          </button>
        </div>
        <div class="max-w-4xl">
          <section class="flex flex-col gap-3">
            <h2 class="text-lg font-semibold">Settings</h2>
            <SiteSettings {site} onsaved={(config) => {
              if (!site) return;
              site.config = config;
              recent = rememberSite(site.root, config.title);
            }} />
          </section>
        </div>
      </div>
      <!-- Podcast, Menu, Theme and Publish have tabs of their own, also kept mounted. -->
      <div id="site-publish" class={["flex max-w-4xl flex-col gap-8", view !== "publish" && "hidden"]}>
        <section class="flex flex-col gap-3">
          <div class="flex items-center justify-between gap-2">
            <h2 class="text-lg font-semibold">Build</h2>
            <button id="build-site" class="btn btn-sm" onclick={build} disabled={loading || building}>
              {#if building}<span class="loading loading-spinner loading-sm"></span>{:else}<Hammer class="size-4" />{/if}
              Build
            </button>
          </div>
          <p class="text-sm text-base-content/60">
            Writes the website into <code>_site/</code> in the site folder, without publishing it. Preview and
            Publish do this by themselves; this is for looking at the files or uploading them yourself.
          </p>
        </section>
        <PublishPanel {site} ontargets={(targets) => site && (site.targets = targets)} />
      </div>
      <section id="site-podcast" class={["flex max-w-4xl flex-col gap-3", view !== "podcast" && "hidden"]}>
        <h2 class="text-lg font-semibold">Podcast</h2>
        <PodcastPanel {site} onsaved={(config) => site && (site.config = config)} />
      </section>
      <section id="site-contributors" class={["flex flex-col gap-3", view !== "contributors" && "hidden"]}>
        <h2 class="text-lg font-semibold">Contributors</h2>
        <ContributorsPanel {site} onchanged={(people) => site && (site.contributors = people)} />
      </section>
      <section id="site-menu" class={["flex max-w-4xl flex-col gap-3", view !== "menu" && "hidden"]}>
        <h2 class="text-lg font-semibold">Menu</h2>
        <MenuEditor {site} onsaved={(config) => site && (site.config = config)} />
      </section>
      <section id="site-theme-tab" class={["flex flex-col gap-3", view !== "theme" && "hidden"]}>
        <h2 class="text-lg font-semibold">Theme</h2>
        <SiteTheme {site} onsaved={(config) => site && (site.config = config)} />
      </section>
    {/key}

    <!-- Mounted on demand, so they list the folder fresh each time, and
         made again for another site (they load only when they appear). -->
    {#key site.root}
      {#if view === "media"}
        <MediaLibrary siteRoot={site.root} editing={editorDirty} onmoved={() => site && load(site.root)} />
      {:else if view === "assets"}
        <MediaLibrary siteRoot={site.root} library="assets" />
      {:else if view === "audio"}
        <MediaLibrary siteRoot={site.root} library="audio" />
      {:else if view === "transcripts"}
        <MediaLibrary siteRoot={site.root} library="transcripts" />
      {:else if view === "static"}
        <MediaLibrary siteRoot={site.root} library="static" />
      {:else if view === "versions"}
        {#key historyChanges}
          <VersionsPanel siteRoot={site.root} editing={editorDirty} onrestored={() => site && load(site.root)} />
        {/key}
      {/if}
    {/key}

    <div class={["-mb-3 flex items-center gap-3", view !== "pages" && "hidden"]}>
      <button id="new-page" class="btn btn-sm btn-primary" onclick={() => (creatingPage = true)} disabled={loading}>
        <FilePlus class="size-4" /> New page
      </button>
      {#if site.config.podcast}
        <button id="new-episode" class="btn btn-sm" onclick={() => (creatingEpisode = true)} disabled={loading}>
          <Mic class="size-4" /> New episode
        </button>
      {/if}
      <span class="text-sm text-base-content/60">
        {site.pages.length} pages{draftCount ? `, ${draftCount} drafts` : ""}
      </span>
      <ManualLink page="pages-and-blocks/" topic="Pages and blocks" />
      {#if below.size}
        <button id="unfold-all" class="btn btn-ghost btn-xs" onclick={() => below.forEach((_, path) => unfolded.add(path))}>
          <ChevronsUpDown class="size-3.5" /> Expand all
        </button>
        <button id="fold-all" class="btn btn-ghost btn-xs" onclick={() => unfolded.clear()}>
          <ChevronsDownUp class="size-3.5" /> Collapse all
        </button>
      {/if}
    </div>

    <!-- The pages view stays mounted while hidden, so unsaved edits survive. -->
    <div
      class={[
        "grid items-start gap-6",
        view !== "pages" && "hidden",
        panelOpen && "lg:grid-cols-[22rem_minmax(0,1fr)] 2xl:grid-cols-[28rem_minmax(0,1fr)]",
      ]}
    >
      <ul
        id="pages"
        class={[
          "flex flex-col gap-0.5 rounded-box bg-base-100 p-2 shadow-sm",
          // While a page is open, the list stays in view and scrolls by itself.
          panelOpen && "lg:sticky lg:top-8 lg:max-h-[calc(100vh-4rem)] lg:overflow-y-auto",
        ]}
      >
        <li id="pages-heading" class="px-1 pb-1">
          <span class="text-xs font-semibold tracking-wide text-base-content/60 uppercase">Pages</span>
        </li>
        {#each shownPages as page, row (page.path)}
          {@const count = below.get(page.path)}
          {@const drop = pageDrag?.plan && pageDrag.row === row ? pageDrag : null}
          <li id="page-{page.path || 'home'}" bind:this={rows[row]}
            class={["group relative flex items-center gap-1", pageDrag?.page.path === page.path && "opacity-50"]}
            style:padding-left="{Math.max(depth(page.path) - 1, 0) * 1.25}rem">
            <!-- Where the dragged page would land. -->
            {#if drop && drop.zone !== "onto"}
              {@const parent = drop.zone === "after" && unfolded.has(page.path) && count ? page.path : page.path === "" ? "" : parentOf(page.path)}
              <div class={["pointer-events-none absolute right-0 h-0.5 rounded-full bg-primary", drop.zone === "before" ? "-top-px" : "-bottom-px"]}
                style:left={gapIndent(parent)}></div>
            {/if}
            {#if page.path === ""}
              <span class="w-5 shrink-0"></span>
            {:else}
              <button type="button" class={["flex shrink-0 touch-none items-center rounded-field py-0.5 text-base-content/25 select-none group-hover:text-base-content/60 hover:bg-base-200", pageDrag ? "cursor-grabbing" : "cursor-grab"]}
                title="Drag to reorder, or onto another page to move it there" aria-label="Drag {page.front.title}"
                onpointerdown={(e) => treeDragStart(e, page)} onpointermove={treeDragMove} onpointerup={treeDragEnd}
                onpointercancel={() => (pageDrag = null)}><GripVertical class="size-4" /></button>
            {/if}
            {#if count}
              <button type="button" id="fold-{page.path}" class="btn btn-ghost btn-xs btn-square shrink-0"
                aria-expanded={unfolded.has(page.path)}
                aria-label="{unfolded.has(page.path) ? 'Fold' : 'Unfold'} {page.front.title}"
                onclick={() => toggle(page.path)}>
                <ChevronRight class={["size-4 transition-transform", unfolded.has(page.path) && "rotate-90"]} />
              </button>
            {:else}
              <span class="w-6 shrink-0"></span>
            {/if}
            <button
              type="button"
              class={[
                "flex min-w-0 flex-1 items-center gap-4 rounded-field px-3 py-1 text-left transition-colors",
                page.file === selectedFile ? "bg-primary/10" : "hover:bg-base-200",
                drop?.zone === "onto" && "ring-2 ring-primary",
              ]}
              onclick={() => select(page.file)}
            >
              <div class="flex min-w-0 flex-1 flex-col leading-snug">
                <span class="font-medium">
                  {#if page.front.icon && icons.has(page.front.icon)}
                    <!-- qed's own built-in SVG, safe to insert. -->
                    <span class="me-1 inline-flex align-[-2px] text-base-content/70">{@html icons.get(page.front.icon)}</span>
                  {/if}
                  {page.front.title}
                  {#if page.front.draft}<span class="badge badge-ghost badge-sm">draft</span>{/if}
                  {#if missingAudio(page).length}
                    <span class="badge badge-soft badge-warning badge-sm" title="Publish waits for this episode's audio">
                      no {missingAudio(page).map(formatLabel).join(", ")}
                    </span>
                  {/if}
                  {#if page.front.menu !== "none"}
                    <span class="badge badge-outline badge-sm">{page.front.menu} menu</span>
                  {/if}
                </span>
                <span class="font-mono text-xs text-base-content/60">
                  /{page.path}{#if page.front.date}<span class="ms-2">· {page.front.date}</span>{/if}
                  {#if count && !unfolded.has(page.path)}<span class="ms-2">· {count} below</span>{/if}
                </span>
              </div>
              {#if !panelOpen}
                <div class="flex flex-wrap justify-end gap-1">
                  {#if page.front.episode && site.config.podcast}
                    <!-- The episode layout makes the page; its blocks are the shownotes. -->
                    <span class="badge badge-soft badge-sm" title="Built with the episode layout (Templates); its own blocks are the shownotes">
                      <LayoutTemplate class="size-3" />Episode
                    </span>
                  {:else}
                    {#each page.blocks as block, i (i)}
                      <span class="badge badge-soft badge-sm">{BLOCK_NAMES[block.type]}</span>
                    {/each}
                  {/if}
                </div>
              {/if}
            </button>
          </li>
        {/each}
        <!-- Shared blocks show on several pages, so they come after them. -->
        <li id="shared-blocks" class="mt-1 flex items-center gap-1 border-t border-base-300 px-1 pt-2">
          <span class="flex-1 text-xs font-semibold tracking-wide text-base-content/60 uppercase">Shared blocks</span>
          <button type="button" id="new-shared" class="btn btn-ghost btn-xs" title="A new shared block: the same blocks on several pages"
            onclick={() => {
              newSharedTitle = "";
              newSharedError = null;
            }}><Plus class="size-3.5" /> New</button>
        </li>
        {#if newSharedTitle !== null}
          <li class="flex flex-col gap-1 px-1">
            <form class="flex items-center gap-1"
              onsubmit={(e) => {
                e.preventDefault();
                createShared();
              }}>
              <!-- svelte-ignore a11y_autofocus -->
              <input id="new-shared-title" class="input input-sm min-w-0 flex-1" placeholder="Name, e.g. Opening hours"
                aria-label="Name of the new shared block" bind:value={newSharedTitle} autofocus />
              <button type="submit" class="btn btn-primary btn-sm" disabled={!newSharedTitle.trim()}>Create</button>
              <button type="button" class="btn btn-ghost btn-sm btn-square" aria-label="Cancel"
                onclick={() => (newSharedTitle = null)}><X class="size-4" /></button>
            </form>
            {#if newSharedError}<p role="alert" class="text-xs text-error">{newSharedError}</p>{/if}
          </li>
        {/if}
        {#each site.shared as piece (piece.file)}
          {@const users = sharedUsers(sharedName(piece))}
          <li id="shared-{sharedName(piece)}" class="flex items-center gap-1">
            <span class="w-6 shrink-0"></span>
            <button
              type="button"
              class={[
                "flex min-w-0 flex-1 items-center gap-4 rounded-field px-3 py-1 text-left transition-colors",
                piece.file === selectedFile ? "bg-primary/10" : "hover:bg-base-200",
              ]}
              onclick={() => select(piece.file)}
            >
              <div class="flex min-w-0 flex-1 flex-col leading-snug">
                <span class="font-medium"><Component class="me-1 inline size-4 align-[-2px] text-base-content/70" />{piece.front.title}</span>
                <span class="text-xs text-base-content/60">
                  {users.length ? `on ${users.length} ${users.length === 1 ? "page" : "pages"}` : "not used yet"}
                  · {piece.blocks.length} {piece.blocks.length === 1 ? "block" : "blocks"}
                </span>
              </div>
            </button>
          </li>
        {:else}
          {#if newSharedTitle === null}
            <li class="px-3 pb-1 text-xs text-base-content/50">None yet. “Make shared” on a block makes one.</li>
          {/if}
        {/each}
        <!-- Templates: blocks on every page (the footer) or every episode. -->
        <li id="templates" class="mt-1 flex items-center gap-1 border-t border-base-300 px-1 pt-2">
          <span class="flex-1 text-xs font-semibold tracking-wide text-base-content/60 uppercase">Templates</span>
        </li>
        <li id="page-footer" class="flex items-center gap-1">
          <span class="w-6 shrink-0"></span>
          <button
            type="button"
            class={[
              "flex min-w-0 flex-1 items-center gap-4 rounded-field px-3 py-1 text-left transition-colors",
              editingFooter ? "bg-primary/10" : "hover:bg-base-200",
            ]}
            onclick={() => select(FOOTER_FILE)}
          >
            <div class="flex min-w-0 flex-1 flex-col leading-snug">
              <span class="font-medium"><PanelBottom class="me-1 inline size-4 align-[-2px] text-base-content/70" />Footer</span>
              <span class="text-xs text-base-content/60">
                {site.footer ? `on every page · ${site.footer.blocks.length} ${site.footer.blocks.length === 1 ? "block" : "blocks"}` : "none yet"}
              </span>
            </div>
          </button>
        </li>
        {#if site.config.podcast}
          {@const layout = site.episode_layout?.blocks}
          <li id="page-episode-layout" class="flex items-center gap-1">
            <span class="w-6 shrink-0"></span>
            <button
              type="button"
              class={[
                "flex min-w-0 flex-1 items-center gap-4 rounded-field px-3 py-1 text-left transition-colors",
                editingEpisodeLayout ? "bg-primary/10" : "hover:bg-base-200",
              ]}
              onclick={() => select(EPISODE_LAYOUT_FILE)}
            >
              <div class="flex min-w-0 flex-1 flex-col leading-snug">
                <span class="font-medium"><LayoutTemplate class="me-1 inline size-4 align-[-2px] text-base-content/70" />Episode layout</span>
                <span class="text-xs text-base-content/60">
                  {layout ? `on every episode · ${layout.length} ${layout.length === 1 ? "block" : "blocks"}` : "on every episode · the default"}
                </span>
              </div>
            </button>
          </li>
        {/if}
      </ul>

      {#if editingEpisodeLayout}
        <EpisodeLayoutEditor siteRoot={site.root} saved={site.episode_layout?.blocks ?? defaultEpisodeLayout()}
          onclose={() => select(null)}
          bind:dirty={editorDirty}
          onsaved={() => site && load(site.root)} />
      {/if}

      {#if editingFooter}
        <FooterEditor siteRoot={site.root} saved={site.footer?.blocks ?? []} onclose={() => select(null)}
          bind:dirty={editorDirty}
          onsaved={() => site && load(site.root)} />
      {/if}

      {#if selectedShared}
        {#key selectedShared.file}
          <SharedEditor siteRoot={site.root} piece={selectedShared} usedOn={sharedUsers(sharedName(selectedShared))}
            bind:dirty={editorDirty}
            onsaved={replaceShared}
            ondeleted={pageDeleted}
            onclose={() => select(null)} />
        {/key}
      {/if}

      {#if selected}
        <!-- Re-create the editor when another page is picked, so it starts fresh. -->
        {#key `${selected.file} ${reorders} ${reloads}`}
          <PageEditor
            page={selected}
            siteRoot={site.root}
            onsaved={replacePage}
            onclose={() => select(null)}
            ondeleted={pageDeleted}
            onmove={() => (movingPage = true)}
            podcast={site.config.podcast}
            contributors={site.contributors}
            bind:dirty={editorDirty}
          />
        {/key}
      {/if}
    </div>
  {:else if !loading && view === "pages"}
    <section id="empty-state" class="flex flex-1 flex-col items-center justify-center gap-3 text-center">
      <img id="empty-state-logo" src={logoMark} alt="" class="mb-2 size-40" />
      <p class="text-lg">Open a site folder to see its pages, or make a new site.</p>
      <button id="open-site" class="btn btn-primary" onclick={chooseFolder} disabled={loading}>
        {#if loading}<span class="loading loading-spinner loading-sm"></span>{:else}<FolderOpen class="size-4" />{/if}
        Open site folder
      </button>
      <button id="new-site" class="btn" onclick={() => (creatingSite = true)} disabled={loading}>
        <FolderPlus class="size-4" /> New site…
      </button>
      <button id="clone-site-start" class="btn btn-ghost" onclick={() => (cloningSite = true)} disabled={loading}>
        <GitBranch class="size-4" /> Get a site from git…
      </button>
      <p class="text-sm text-base-content/60">
        A site folder has a <code>site.toml</code> and a <code>pages/</code> folder. Try
        <code>examples/demo-site</code> in the q.e.d. repository.
      </p>
      {#if recent.length}
        <ul id="recent-sites" class="mt-4 flex w-full max-w-2xl flex-col gap-1 text-start">
          <li class="px-3 text-sm font-semibold text-base-content/60">Recent sites</li>
          {#each recent as recentSite (recentSite.path)}
            <li class="flex items-center gap-2 rounded-box hover:bg-base-200">
              <button type="button" class="flex min-w-0 flex-1 flex-col px-3 py-2 text-start" disabled={loading}
                onclick={() => switchTo(recentSite.path)}>
                <span class="font-medium">{recentSite.title}</span>
                <span class="truncate font-mono text-xs text-base-content/60">{recentSite.path}</span>
              </button>
              <button type="button" class="btn btn-ghost btn-sm btn-square me-2" title="Remove from the list (the folder stays)"
                aria-label="Remove from the list" onclick={() => (recent = forgetSite(recentSite.path))}>
                <X class="size-4" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</main>

{#if site && creatingPage}
  <NewPageDialog {site} oncreated={pageCreated} onclose={() => (creatingPage = false)} />
{/if}

{#if site && creatingEpisode}
  <NewPageDialog {site} episode oncreated={pageCreated} onclose={() => (creatingEpisode = false)} />
{/if}

{#if site && selected && movingPage}
  <MovePageDialog {site} page={selected} onmoved={pageMoved} onclose={() => (movingPage = false)} />
{/if}

{#if cloningSite}
  <CloneSiteDialog
    oncreated={(path) => {
      view = "pages";
      switchTo(path);
    }}
    onclose={() => (cloningSite = false)}
  />
{/if}

{#if creatingSite}
  <NewSiteDialog
    oncreated={(path) => {
      view = "pages";
      switchTo(path);
    }}
    onclose={() => (creatingSite = false)}
  />
{/if}
