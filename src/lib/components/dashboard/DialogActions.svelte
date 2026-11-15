<script lang="ts">
  import { Button, Row } from '@hermitk/bluenite'

  interface Props {
    confirmLabel: string
    oncancel: () => void
    onconfirm?: () => void
    submitForm?: string
    busy?: boolean
    danger?: boolean
    confirmDisabled?: boolean
    cancelLabel?: string
  }

  let {
    confirmLabel,
    oncancel,
    onconfirm,
    submitForm,
    busy = false,
    danger = false,
    confirmDisabled = false,
    cancelLabel = 'Cancel',
  }: Props = $props()
</script>

<Row justify="end" gap={0.5} style="width: 100%">
  <Button variant="ghost" color="secondary" disabled={busy} onclick={oncancel}>
    {cancelLabel}
  </Button>
  <Button
    type={submitForm ? 'submit' : 'button'}
    form={submitForm}
    loading={busy}
    disabled={busy || confirmDisabled}
    variant={danger ? 'outline' : undefined}
    color={danger ? 'danger' : undefined}
    onclick={onconfirm}
  >
    {confirmLabel}
  </Button>
</Row>
