<script lang="ts">
  import { Button, Input, Modal, Row } from '@hermitk/bluenite'
  import { createVault, type VaultSummary } from '#lib/api.ts'

  type CreateVaultModalProps = {
    open?: boolean
    onCreated?: (vault: VaultSummary) => void
  }

  let { open = $bindable(false), onCreated }: CreateVaultModalProps = $props()

  let name = $state('')
  let passphrase = $state('')
  let confirm = $state('')
  let busy = $state(false)
  let error = $state('')

  const canSubmit = $derived(
    name.trim().length > 0 && passphrase.length >= 12 && passphrase === confirm,
  )

  function reset() {
    name = ''
    passphrase = ''
    confirm = ''
    error = ''
    busy = false
  }

  function handleClose() {
    open = false
    reset()
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()
    error = ''
    const trimmed = name.trim()
    if (!trimmed) {
      error = 'Vault name cannot be empty.'
      return
    }
    if (trimmed.length > 64) {
      error = 'Vault name cannot exceed 64 characters.'
      return
    }
    if (passphrase.length < 12) {
      error = 'Passphrase must be at least 12 characters.'
      return
    }
    if (passphrase !== confirm) {
      error = 'Passphrases do not match.'
      return
    }

    busy = true
    try {
      const created = await createVault(trimmed, passphrase)
      open = false
      reset()
      onCreated?.(created)
    } catch (err) {
      error = err instanceof Error ? err.message : String(err)
    } finally {
      busy = false
    }
  }
</script>

<Modal bind:open title="Create new vault" size="sm" onClose={handleClose}>
  <form id="create-vault-form" class="fields" onsubmit={handleSubmit}>
    <Input
      placeholder="Vault name (e.g. Work, Personal)"
      bind:value={name}
      maxlength={64}
      disabled={busy}
      label="Name"
    />
    <Input
      type="password"
      placeholder="At least 12 characters"
      bind:value={passphrase}
      disabled={busy}
      label="Master passphrase"
    />
    <Input
      type="password"
      placeholder="Re-enter passphrase"
      bind:value={confirm}
      disabled={busy}
      label="Confirm passphrase"
    />
    {#if error}
      <p class="form-error" role="alert">{error}</p>
    {/if}
  </form>

  {#snippet footer()}
    <Row justify="end" gap={0.75} style="width: 100%">
      <Button
        type="button"
        variant="ghost"
        color="secondary"
        size="md"
        disabled={busy}
        onclick={handleClose}
      >
        Cancel
      </Button>
      <Button
        type="submit"
        form="create-vault-form"
        size="md"
        color="primary"
        loading={busy}
        disabled={!canSubmit}
      >
        Create Vault
      </Button>
    </Row>
  {/snippet}
</Modal>

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .form-error {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--danger);
  }
</style>
