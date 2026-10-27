<script lang="ts">
  // Nothing to set up: the sketch is markup only.
</script>

<!--
  A wireframe of a generated page, drawn with the theme's colors (it takes
  them from the element it sits in). The numbers match the legend: where
  primary (1), secondary (2), accent (3) and neutral (4) show up.
-->
{#snippet mark(n: number, place = "-top-1.5 -right-1.5")}
  <span class={["absolute z-10 flex size-4 items-center justify-center rounded-full bg-base-content text-[0.55rem] font-bold text-base-100 ring-2 ring-base-100", place]}>{n}</span>
{/snippet}

<div id="site-sketch" aria-hidden="true"
  class="flex flex-col overflow-hidden rounded-box border border-base-300 bg-base-100 text-base-content shadow-sm">
  <!-- Accent stripe and header with the menu; the current page in primary. -->
  <div class="relative h-1 bg-accent">{@render mark(3, "top-0.5 left-2")}</div>
  <div class="flex items-center justify-between border-b border-base-300 px-3 py-2">
    <div class="flex items-center gap-1.5">
      <span class="size-3 rounded-field bg-base-300"></span>
      <span class="h-2 w-14 rounded-full bg-base-content/80"></span>
    </div>
    <div class="flex items-center gap-2">
      <span class="relative h-1.5 w-6 rounded-full bg-primary">{@render mark(1, "-top-4 -right-1")}</span>
      <span class="h-1.5 w-6 rounded-full bg-base-content/30"></span>
      <span class="h-1.5 w-6 rounded-full bg-base-content/30"></span>
    </div>
  </div>

  <div class="flex flex-col gap-3 px-3 py-3">
    <!-- Hero without an image: secondary, its button primary. -->
    <div class="relative flex flex-col items-center gap-1.5 rounded-box bg-secondary px-4 py-4 text-secondary-content">
      {@render mark(2)}
      <span class="h-2.5 w-2/3 rounded-full bg-current opacity-80"></span>
      <span class="h-1.5 w-1/2 rounded-full bg-current opacity-50"></span>
      <span class="relative mt-1.5 h-4 w-14 rounded-field bg-primary">{@render mark(1)}</span>
    </div>

    <!-- Text with a link (primary underline) and a quote (accent bar). -->
    <div class="flex flex-col gap-1">
      <span class="h-1.5 w-full rounded-full bg-base-content/25"></span>
      <span class="flex items-end gap-1">
        <span class="h-1.5 w-1/4 rounded-full bg-base-content/25"></span>
        <span class="relative h-2 w-1/4 border-b-2 border-primary">
          <span class="absolute inset-x-0 top-0 h-1.5 rounded-full bg-base-content/40"></span>
        </span>
        <span class="h-1.5 flex-1 rounded-full bg-base-content/25"></span>
      </span>
    </div>
    <div class="relative flex flex-col gap-1 border-l-2 border-accent pl-2">
      {@render mark(3, "-top-1.5 -left-2")}
      <span class="h-1.5 w-4/5 rounded-full bg-base-content/25"></span>
      <span class="h-1.5 w-3/5 rounded-full bg-base-content/25"></span>
    </div>

    <!-- A post with its date (secondary). -->
    <div class="flex flex-col gap-1">
      <span class="h-2 w-1/3 rounded-full bg-base-content/70"></span>
      <span class="relative h-2.5 w-10 rounded-field bg-secondary">{@render mark(2, "-top-1 -right-5")}</span>
      <span class="h-1.5 w-2/3 rounded-full bg-base-content/25"></span>
    </div>

    <!-- A download with its file type (neutral). -->
    <div class="flex items-center gap-2 rounded-box border border-base-300 p-1.5">
      <span class="relative h-3 w-7 rounded-field bg-neutral">{@render mark(4)}</span>
      <span class="h-1.5 w-1/3 rounded-full bg-base-content/50"></span>
    </div>

    <!-- Call to action: a primary box. -->
    <div class="relative flex flex-col items-center gap-1.5 rounded-box bg-primary px-4 py-3 text-primary-content">
      {@render mark(1)}
      <span class="h-2 w-1/3 rounded-full bg-current opacity-80"></span>
      <span class="h-3.5 w-12 rounded-field bg-base-100"></span>
    </div>
  </div>

  <!-- Footer in accent. -->
  <div class="relative flex items-center justify-between bg-accent px-3 py-2.5 text-accent-content">
    {@render mark(3, "-top-1.5 left-2")}
    <span class="h-1.5 w-12 rounded-full bg-current opacity-60"></span>
    <span class="h-1.5 w-16 rounded-full bg-current opacity-60"></span>
  </div>
</div>
