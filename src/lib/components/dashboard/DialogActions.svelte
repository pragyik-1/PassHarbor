<script lang="ts">
  import { Button } from "@hermitk/bluenite";

  interface Props {
    confirmLabel: string;
    oncancel: () => void;
    onconfirm?: () => void;
    submitForm?: string;
    busy?: boolean;
    danger?: boolean;
  }

  let {
    confirmLabel,
    oncancel,
    onconfirm,
    submitForm,
    busy = false,
    danger = false,
  }: Props = $props();
</script>

<div class="actions">
  <Button
    variant="ghost"
    color="secondary"
    size="sm"
    disabled={busy}
    onclick={oncancel}
  >
    Cancel
  </Button>
  <Button
    type={submitForm ? "submit" : "button"}
    form={submitForm}
    size="sm"
    loading={busy}
    variant={danger ? "outline" : undefined}
    color={danger ? "danger" : undefined}
    onclick={onconfirm}
  >
    {confirmLabel}
  </Button>
</div>

<style>
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
