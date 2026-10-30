<script lang="ts">
  import { Button } from "@hermitk/bluenite";
  import { CheckIcon, CopyIcon, EyeIcon, EyeOffIcon } from "lucide-svelte";

  interface Props {
    value: string;
    revealed: boolean;
    copied: boolean;
    label: string;
    compact?: boolean;
    onreveal: () => void;
    oncopy: () => void;
  }

  let {
    value,
    revealed,
    copied,
    label,
    compact = false,
    onreveal,
    oncopy,
  }: Props = $props();
</script>

<div class="secret-field" class:compact>
  <span
    class="secret-value"
    aria-label={revealed ? `${label} revealed` : `${label} hidden`}
  >
    {revealed ? value : "••••••••••••"}
  </span>
  <div class="secret-actions">
    <Button
      variant="ghost"
      color="secondary"
      size="sm"
      class="secret-btn"
      aria-label={revealed ? `Hide ${label.toLowerCase()}` : `Reveal ${label.toLowerCase()}`}
      title={revealed ? "Hide" : "Reveal"}
      aria-pressed={revealed}
      onclick={onreveal}
    >
      {#snippet icon()}
        {#if revealed}
          <EyeOffIcon size={compact ? 14 : 15} />
        {:else}
          <EyeIcon size={compact ? 14 : 15} />
        {/if}
      {/snippet}
    </Button>
    <Button
      variant="ghost"
      color="secondary"
      size="sm"
      class="secret-btn"
      aria-label="Copy {label.toLowerCase()}"
      title="Copy"
      onclick={oncopy}
    >
      {#snippet icon()}
        {#if copied}
          <CheckIcon size={compact ? 14 : 15} />
        {:else}
          <CopyIcon size={compact ? 14 : 15} />
        {/if}
      {/snippet}
    </Button>
  </div>
</div>

<style>
  .secret-field {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-width: 0;
  }

  .secret-value {
    flex: 1;
    font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
    font-size: 0.875rem;
    letter-spacing: 0.05em;
    word-break: break-all;
    color: var(--text);
    user-select: none;
  }

  .compact .secret-value {
    font-size: 0.8rem;
  }

  .secret-actions {
    display: flex;
    gap: 0.125rem;
    flex-shrink: 0;
  }

  :global(.secret-btn) {
    width: 2rem !important;
    height: 2rem !important;
    padding: 0 !important;
  }
</style>
