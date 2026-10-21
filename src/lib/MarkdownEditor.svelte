<script lang="ts">
  import { onMount } from "svelte";
  import OverType, { type OverTypeInstance, type Theme } from "overtype";

  interface Props {
    value: string;
    placeholder?: string;
    minHeight?: string;
  }

  // `$bindable` lets the parent write `bind:value={…}`, like on an <input>.
  let { value = $bindable(), placeholder = "", minHeight = "160px" }: Props = $props();

  let element: HTMLDivElement;
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

  onMount(() => {
    [editor] = new OverType(element, {
      value,
      placeholder,
      toolbar: true,
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
