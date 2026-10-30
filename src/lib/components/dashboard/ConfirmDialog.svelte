<script lang="ts">
  import { Modal } from "@hermitk/bluenite";
  import { TriangleAlertIcon } from "lucide-svelte";
  import DialogActions from "./DialogActions.svelte";

  interface Props {
    open: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    busy?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let {
    open = $bindable(),
    title,
    message,
    confirmLabel = "Delete",
    busy = false,
    onconfirm,
    oncancel,
  }: Props = $props();
</script>

<Modal
  bind:open
  size="sm"
  {title}
  onClose={oncancel}
  closeOnBackdropClick={!busy}
>
  <div class="body">
    <div class="icon-wrap" aria-hidden="true">
      <TriangleAlertIcon size={22} />
    </div>
    <p class="message">{message}</p>
  </div>

  {#snippet footer()}
    <DialogActions {busy} danger {confirmLabel} {onconfirm} {oncancel} />
  {/snippet}
</Modal>

<style>
  .body {
    display: flex;
    gap: 0.875rem;
    align-items: flex-start;
    padding: 0.25rem 0 0.75rem;
  }

  .icon-wrap {
    flex-shrink: 0;
    display: grid;
    place-items: center;
    width: 2.25rem;
    height: 2.25rem;
    border-radius: var(--round-md);
    background: color-mix(in srgb, var(--danger) 12%, transparent);
    color: var(--danger);
  }

  .message {
    margin: 0;
    font-size: 0.875rem;
    color: var(--secondary-text);
    line-height: 1.55;
  }
</style>
