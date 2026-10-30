<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "@hermitk/bluenite";
  import { CheckIcon, CopyIcon } from "lucide-svelte";

  interface Props {
    icon: Snippet;
    label: string;
    value: string | null;
    mono?: boolean;
    multiline?: boolean;
    copied?: boolean;
    oncopy?: () => void;
  }

  let {
    icon,
    label,
    value,
    mono = false,
    multiline = false,
    copied = false,
    oncopy,
  }: Props = $props();
</script>

<div class="meta-row" class:muted={!value}>
  <span class="meta-icon" aria-hidden="true">{@render icon()}</span>
  <div class="meta-content">
    <span class="meta-label">{label}</span>
    <span
      class="meta-value"
      class:muted-text={!value}
      class:mono={mono && value}
      class:multiline={multiline}
    >
      {value ?? "Not set"}
    </span>
  </div>
  {#if value && oncopy}
    <Button
      variant="ghost"
      color="secondary"
      size="sm"
      class="copy-btn"
      aria-label="Copy {label}"
      title="Copy {label}"
      onclick={oncopy}
    >
      {#snippet icon()}
        {#if copied}
          <CheckIcon size={14} />
        {:else}
          <CopyIcon size={14} />
        {/if}
      {/snippet}
    </Button>
  {/if}
</div>

<style>
  .meta-row {
    display: flex;
    align-items: flex-start;
    gap: 0.625rem;
    padding: 0.45rem 0.5rem;
    border-radius: var(--round-sm);
    transition: background 120ms ease;
    min-height: 2.25rem;
  }

  .meta-row:not(.muted):hover {
    background: var(--surface-hover);
  }

  .meta-icon {
    display: flex;
    align-items: center;
    padding-top: 0.2rem;
    color: var(--muted-text);
    flex-shrink: 0;
  }

  .meta-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .meta-label {
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-text);
  }

  .meta-value {
    font-size: 0.875rem;
    color: var(--text);
    word-break: break-word;
    line-height: 1.45;
  }

  .meta-value.muted-text {
    color: var(--muted-text);
    font-style: italic;
  }

  .meta-value.mono {
    font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
    font-size: 0.82rem;
    word-break: break-all;
  }

  .meta-value.multiline {
    white-space: pre-wrap;
  }

  :global(.copy-btn) {
    width: 1.75rem !important;
    height: 1.75rem !important;
    padding: 0 !important;
    flex-shrink: 0;
    align-self: flex-start;
    margin-top: 0.2rem;
  }
</style>
