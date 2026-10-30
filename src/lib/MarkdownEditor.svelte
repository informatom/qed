<script lang="ts">
  import { onMount } from "svelte";
  import OverType, { toolbarButtons, type OverTypeInstance, type Theme, type ToolbarButton } from "overtype";
  import { pageIcons } from "$lib/site";

  interface Props {
    value: string;
    placeholder?: string;
    minHeight?: string;
  }

  // `$bindable` lets the parent write `bind:value={…}`, like on an <input>.
  let { value = $bindable(), placeholder = "", minHeight = "160px" }: Props = $props();

  let element: HTMLDivElement;

  // The icons that `:icon-name:` can put into text, loaded when the hint
  // is opened; a click copies the code to paste into the text.
  let icons = $state<[string, string][]>([]);
  let copied = $state<string | null>(null);
  async function showIcons(event: Event) {
    if ((event.currentTarget as HTMLDetailsElement).open && icons.length === 0) {
      icons = [...(await pageIcons())];
    }
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

  /** A Lucide icon (https://lucide.dev) as OverType wants it: SVG markup. OverType's
   *  stylesheet fills toolbar icons, which turns Lucide's outlines into solid shapes;
   *  the inline style wins over it. */
  function lucide(...shapes: string[]): string {
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
    icon: lucide('<rect x="3" y="3" width="18" height="18" rx="2"/><path d="m10 9-3 3 3 3"/><path d="m14 15 3-3-3-3"/>'),
    action: ({ editor }) => codeBlock(editor.textarea),
  };

  // The tags text may use besides Markdown (markdown::allowed_tag in qed-core).
  const tagButtons: ToolbarButton[] = [
    {
      name: "kbd",
      title: "Key, shown as a keycap (<kbd>)",
      icon: lucide(
        '<rect width="20" height="16" x="2" y="4" rx="2"/>',
        '<path d="M10 8h.01"/><path d="M12 12h.01"/><path d="M14 8h.01"/><path d="M16 12h.01"/>',
        '<path d="M18 8h.01"/><path d="M6 8h.01"/><path d="M7 16h10"/><path d="M8 12h.01"/>',
      ),
      action: ({ editor }) => wrap(editor.textarea, "<kbd>", "</kbd>"),
    },
    {
      name: "sup",
      title: "Superscript, as in m² (<sup>)",
      icon: lucide(
        '<path d="m4 19 8-8"/><path d="m12 19-8-8"/>',
        '<path d="M20 12h-4c0-1.5.442-2 1.5-2.5S20 8.334 20 7.002c0-.472-.17-.93-.484-1.29a2.105 2.105 0 0 0-2.617-.436c-.42.239-.738.614-.899 1.06"/>',
      ),
      action: ({ editor }) => wrap(editor.textarea, "<sup>", "</sup>"),
    },
    {
      name: "sub",
      title: "Subscript, as in CO₂ (<sub>)",
      icon: lucide(
        '<path d="m4 5 8 8"/><path d="m12 5-8 8"/>',
        '<path d="M20 19h-4c0-1.5.44-2 1.5-2.5S20 15.33 20 14c0-.47-.17-.93-.48-1.29a2.11 2.11 0 0 0-2.62-.44c-.42.24-.74.62-.9 1.07"/>',
      ),
      action: ({ editor }) => wrap(editor.textarea, "<sub>", "</sub>"),
    },
    {
      name: "br",
      title: "Line break, also in a table cell (<br>)",
      icon: lucide('<path d="M20 4v7a4 4 0 0 1-4 4H4"/><path d="m9 10-5 5 5 5"/>'),
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
    <div class="flex flex-wrap gap-1">
      {#each icons as [name, svg] (name)}
        <button type="button" class="btn btn-ghost btn-xs gap-1 font-normal" title=":icon-{name}:" onclick={() => copy(name)}>
          <!-- q.e.d.'s own built-in SVG, safe to insert. -->
          {@html svg}{name}
        </button>
      {/each}
    </div>
  </div>
</details>
