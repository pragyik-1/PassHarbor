<script lang="ts">
  import { Button, Row } from '@hermitk/bluenite'

  interface Props {
    confirmLabel: string
    oncancel: () => void
    onconfirm?: () => void
    submitForm?: string
    busy?: boolean
    danger?: boolean
  }

  let {
    confirmLabel,
    oncancel,
    onconfirm,
    submitForm,
    busy = false,
    danger = false,
  }: Props = $props()
</script>

<Row justify="end" gap={0.5}>
  <Button variant="ghost" color="secondary" size="sm" disabled={busy} onclick={oncancel}>
    Cancel
  </Button>
  <Button
    type={submitForm ? 'submit' : 'button'}
    form={submitForm}
    size="sm"
    loading={busy}
    variant={danger ? 'outline' : undefined}
    color={danger ? 'danger' : undefined}
    onclick={onconfirm}
  >
    {confirmLabel}
  </Button>
</Row>
