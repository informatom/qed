<script lang="ts">
  import { formatSize, type Transfer } from "$lib/site";

  interface Props {
    transfer: Transfer | null;
  }

  let { transfer }: Props = $props();

  const LABELS = {
    packing: "Packing versions",
    sending: "Sending",
    receiving: "Receiving",
    resolving: "Unpacking",
  } as const;
</script>

{#if transfer}
  <div id="transfer-progress" class="flex flex-col gap-1">
    <progress class="progress progress-primary w-full" value={transfer.done} max={Math.max(transfer.total, 1)}></progress>
    <span class="text-sm text-base-content/70">
      {LABELS[transfer.stage]}: {transfer.done} of {transfer.total}
      {#if transfer.bytes > 0}· {formatSize(transfer.bytes)}{/if}
    </span>
  </div>
{/if}
