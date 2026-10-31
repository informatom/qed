<script lang="ts">
  import { onMount } from "svelte";
  import OverType, { toolbarButtons, type OverTypeInstance, type Theme, type ToolbarButton } from "overtype";
  import IconSearch from "$lib/IconSearch.svelte";

  interface Props {
    value: string;
    placeholder?: string;
    minHeight?: string;
  }

  // `$bindable` lets the parent write `bind:value={…}`, like on an <input>.
  let { value = $bindable(), placeholder = "", minHeight = "160px" }: Props = $props();
  // Several editors can be on one page; this keeps their ids apart.
  const uid = $props.id();

  let element: HTMLDivElement;

  // The icons that `:icon-name:` can put into text, searched in the hint
  // when it's open; a click copies the code to paste into the text.
  let iconsOpen = $state(false);
  let copied = $state<string | null>(null);
  function showIcons(event: Event) {
    iconsOpen = (event.currentTarget as HTMLDetailsElement).open;
  }
  async function copy(name: string) {
    const code = `:icon-${name}:`;
    try {
      await navigator.clipboard.writeText(code);
      copied = code;
    } catch {
      copied = null;
    }
  }
  let editor: OverTypeInstance | undefined;

  // Colours from the active daisyUI theme, so the editor matches the app.
  // Taken over from the Mercator project.
  const theme: Theme = {
    name: "qed",
    colors: {
      bgPrimary: "var(--color-base-100)",
      bgSecondary: "var(--color-base-200)",
      text: "var(--color-base-content)",
      textPrimary: "var(--color-base-content)",
      textSecondary: "var(--color-neutral)",
      h1: "var(--color-primary)",
      h2: "var(--color-primary)",
      h3: "var(--color-secondary)",
      strong: "var(--color-base-content)",
      em: "var(--color-base-content)",
      del: "var(--color-neutral)",
      link: "var(--color-primary)",
      code: "var(--color-base-content)",
      codeBg: "var(--color-base-200)",
      blockquote: "var(--color-neutral)",
      hr: "var(--color-base-300)",
      syntaxMarker: "color-mix(in oklab, var(--color-base-content) 45%, transparent)",
      syntax: "color-mix(in oklab, var(--color-base-content) 45%, transparent)",
      cursor: "var(--color-primary)",
      selection: "color-mix(in oklab, var(--color-primary) 25%, transparent)",
      listMarker: "var(--color-primary)",
      toolbarBg: "var(--color-base-200)",
      toolbarIcon: "var(--color-base-content)",
      toolbarHover: "var(--color-base-300)",
      toolbarActive: "var(--color-base-300)",
      border: "var(--color-base-300)",
    },
  };

  /** A Tabler icon (https://tabler.io/icons) as OverType wants it: SVG markup. OverType's
   *  stylesheet fills toolbar icons, which turns outlines into solid shapes; the inline
   *  style wins over it. */
  function outline(...shapes: string[]): string {
    return `<svg viewBox="0 0 24 24" style="fill: none" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${shapes.join("")}</svg>`;
  }

  /** Puts `before` and `after` around the selection (or the cursor), so it can be undone like typing. */
  function wrap(textarea: HTMLTextAreaElement, before: string, after = "") {
    const { selectionStart: start, selectionEnd: end, value: text } = textarea;
    const selected = text.slice(start, end);
    textarea.focus();
    // execCommand keeps the undo history, as OverType's own buttons do; setRangeText if it's refused.
    if (!document.execCommand("insertText", false, before + selected + after)) {
      textarea.setRangeText(before + selected + after, start, end, "end");
      textarea.dispatchEvent(new Event("input", { bubbles: true }));
    }
    // With nothing selected, the cursor lands between the tags.
    const caret = start + before.length + selected.length;
    textarea.setSelectionRange(selected ? start + before.length : caret, caret);
  }

  /** Puts a ``` block around the selected lines, on lines of its own; empty, the cursor goes inside. */
  function codeBlock(textarea: HTMLTextAreaElement) {
    const { selectionStart: start, selectionEnd: end, value: text } = textarea;
    const before = start > 0 && text[start - 1] !== "\n" ? "\n```\n" : "```\n";
    const after = end < text.length && text[end] !== "\n" ? "\n```\n" : "\n```";
    wrap(textarea, before, after);
  }

  // A fenced code block, next to OverType's inline code.
  const codeBlockButton: ToolbarButton = {
    name: "codeBlock",
    title: "Code block (```)",
    icon: outline('<path d="M14.5 4h2.5a3 3 0 0 1 3 3v10a3 3 0 0 1 -3 3h-10a3 3 0 0 1 -3 -3v-5"/><path d="M6 5l-2 2l2 2"/><path d="M10 9l2 -2l-2 -2"/>'),
    action: ({ editor }) => codeBlock(editor.textarea),
  };

  // The tags text may use besides Markdown (markdown::allowed_tag in qed-core).
  const tagButtons: ToolbarButton[] = [
    {
      name: "kbd",
      title: "Key, shown as a keycap (<kbd>)",
      icon: outline('<path d="M2 8a2 2 0 0 1 2 -2h16a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-16a2 2 0 0 1 -2 -2l0 -8"/><path d="M6 10l0 .01"/><path d="M10 10l0 .01"/><path d="M14 10l0 .01"/><path d="M18 10l0 .01"/><path d="M6 14l0 .01"/><path d="M18 14l0 .01"/><path d="M10 14l4 .01"/>'),
      action: ({ editor }) => wrap(editor.textarea, "<kbd>", "</kbd>"),
    },
    {
      name: "sup",
      title: "Superscript, as in m² (<sup>)",
      icon: outline('<path d="M5 7l8 10m-8 0l8 -10"/><path d="M21 11h-4l3.5 -4a1.73 1.73 0 0 0 -3.5 -2"/>'),
      action: ({ editor }) => wrap(editor.textarea, "<sup>", "</sup>"),
    },
    {
      name: "sub",
      title: "Subscript, as in CO₂ (<sub>)",
      icon: outline('<path d="M5 7l8 10m-8 0l8 -10"/><path d="M21 20h-4l3.5 -4a1.73 1.73 0 0 0 -3.5 -2"/>'),
      action: ({ editor }) => wrap(editor.textarea, "<sub>", "</sub>"),
    },
    {
      name: "br",
      title: "Line break, also in a table cell (<br>)",
      icon: outline('<path d="M18 6v6a3 3 0 0 1 -3 3h-10l4 -4m0 8l-4 -4"/>'),
      action: ({ editor }) => wrap(editor.textarea, "<br>"),
    },
  ];

  // OverType's own buttons, with ours after Code.
  const buttons: ToolbarButton[] = [
    toolbarButtons.bold,
    toolbarButtons.italic,
    toolbarButtons.code,
    codeBlockButton,
    ...tagButtons,
    toolbarButtons.separator,
    toolbarButtons.link,
    toolbarButtons.separator,
    toolbarButtons.h1,
    toolbarButtons.h2,
    toolbarButtons.h3,
    toolbarButtons.separator,
    toolbarButtons.bulletList,
    toolbarButtons.orderedList,
    toolbarButtons.taskList,
    toolbarButtons.separator,
    toolbarButtons.quote,
    toolbarButtons.separator,
    toolbarButtons.viewMode,
  ];

  onMount(() => {
    [editor] = new OverType(element, {
      value,
      placeholder,
      toolbar: true,
      toolbarButtons: buttons,
      autoResize: true,
      minHeight,
      theme,
      onChange: (text: string) => {
        value = text;
      },
    });
    // Returning a function from onMount runs it when the component goes away.
    return () => editor?.destroy();
  });

  // When the value changes from outside (e.g. "Discard changes"), show it.
  $effect(() => {
    if (editor && value !== editor.getValue()) editor.setValue(value);
  });
</script>

<div bind:this={element} class="overflow-hidden rounded-field border border-base-300"></div>
<details class="mt-1 text-xs text-base-content/60" ontoggle={showIcons}>
  <summary class="cursor-pointer select-none">Icons in text: <code>:icon-phone:</code> and so on</summary>
  <div class="mt-2 flex flex-col gap-2">
    <p>Write the code where the icon should go; on the website it shows as tall as the text. Click an icon to copy its code{#if copied}: <span class="text-success">copied {copied}</span>{/if}.</p>
    {#if iconsOpen}
      <IconSearch id="markdown-icon-{uid}" names onpick={copy} />
    {/if}
  </div>
</details>
