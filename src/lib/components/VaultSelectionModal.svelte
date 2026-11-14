<script lang="ts">
  import { onMount } from 'svelte'
  import { Badge, Button, Modal, Radio, Row, toast } from '@hermitk/bluenite'
  import { FolderIcon } from 'lucide-svelte'
  import { listVaults, setDefaultVault, type VaultSummary } from '#lib/api.ts'
  import { timeAgo } from '#lib/time.ts'

  type VaultSelectionModalProps = {
    open?: boolean
    selectedVaultId?: string | null
    vaults?: VaultSummary[]
    onSelect?: (vault: VaultSummary) => void
    onVaultsChanged?: (vaults: VaultSummary[]) => void
  }

  let {
    open = $bindable(false),
    selectedVaultId = $bindable(null),
    vaults = $bindable([]),
    onSelect,
    onVaultsChanged,
  }: VaultSelectionModalProps = $props()

  let defaultBusy = $state(false)

  onMount(async () => {
    if (vaults.length === 0) {
      try {
        vaults = await listVaults()
        if (!selectedVaultId) {
          const defaultVault = vaults.find((v) => v.is_default) ?? vaults[0] ?? null
          if (defaultVault) selectedVaultId = defaultVault.id
        }
      } catch (err) {
        toast.show({
          variant: 'danger',
          message: 'Failed to list vaults',
          description: `${err}`,
        })
      }
    }
  })

  function handleClose() {
    open = false
  }

  function handleProceed() {
    const vault = vaults.find((v) => v.id === selectedVaultId)
    if (vault) onSelect?.(vault)
    open = false
  }

  async function handleSetDefault(vaultId: string) {
    if (defaultBusy) return
    defaultBusy = true
    try {
      await setDefaultVault(vaultId)
      vaults = await listVaults()
      onVaultsChanged?.(vaults)
    } catch (err) {
      toast.show({
        variant: 'danger',
        message: 'Failed to set default vault',
        description: `${err}`,
      })
    } finally {
      defaultBusy = false
    }
  }
</script>

<Modal bind:open title="Select a vault" size="md" onClose={handleClose} closeOnBackdropClick={true}>
  <div class="body">
    <p class="subtitle">Select a vault to unlock, or designate one as your default.</p>

    <div class="vault-list" role="radiogroup" aria-label="Available vaults">
      {#each vaults as v (v.id)}
        {@const isSelected = selectedVaultId === v.id}
        <div class="vault-item" class:selected={isSelected}>
          <Radio
            bind:group={selectedVaultId}
            value={v.id}
            id="vault-radio-{v.id}"
            name="vault-selection"
            class="vault-radio"
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault()
                handleProceed()
              }
            }}
          >
            <div class="vault-info">
              <div class="vault-title-row">
                <FolderIcon size={14} />
                <span class="vault-name">{v.name}</span>
                {#if v.is_default}
                  <Badge variant="primary" size="sm">Default</Badge>
                {/if}
              </div>
              <span class="vault-meta">Created {timeAgo(v.created_at)}</span>
            </div>
          </Radio>

          <div class="vault-actions">
            {#if !v.is_default}
              <Button
                type="button"
                variant="ghost"
                color="secondary"
                size="sm"
                disabled={defaultBusy}
                onclick={() => void handleSetDefault(v.id)}
              >
                Set default
              </Button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>

  {#snippet footer()}
    <Row justify="end" gap={0.75} style="width: 100%">
      <Button type="button" variant="ghost" color="secondary" size="md" onclick={handleClose}>
        Cancel
      </Button>
      <Button
        type="button"
        variant="fill"
        color="primary"
        size="md"
        disabled={!selectedVaultId}
        onclick={handleProceed}
      >
        Proceed to unlock
      </Button>
    </Row>
  {/snippet}
</Modal>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .subtitle {
    margin: 0;
    font-size: 0.875rem;
    color: var(--secondary-text);
    line-height: 1.4;
  }

  .vault-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 320px;
    overflow-y: auto;
  }

  .vault-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: var(--surface-hover);
    border: 1px solid var(--border);
    border-radius: var(--round-md);
    transition:
      border-color 140ms ease,
      background-color 140ms ease;
  }

  .vault-item:hover {
    border-color: color-mix(in srgb, var(--primary) 40%, var(--border));
    background: color-mix(in srgb, var(--primary) 4%, var(--surface-hover));
  }

  .vault-item.selected {
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 8%, var(--surface-hover));
  }

  :global(.vault-radio) {
    flex: 1;
    min-width: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  :global(.vault-radio .radio-label) {
    flex: 1;
    min-width: 0;
    margin-left: 0.25rem;
  }

  .vault-info {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .vault-title-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .vault-name {
    font-weight: 600;
    font-size: 0.9375rem;
    color: var(--text);
  }

  .vault-meta {
    font-size: 0.75rem;
    color: var(--muted-text);
  }

  .vault-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }
</style>
