<script lang="ts">
  import { X } from 'lucide-svelte'
  import { Button, Input } from '../..'
  import Modal from '../ui/Modal.svelte'
  import './AddVaultModal.css'
  import { refreshVault } from '../../../global.svelte'

  let { open = $bindable(false) } = $props()
  let vaultName = $state('')
</script>

<Modal bind:isOpen={open} class="modal">
  <div class="modal-content">
    <header class="modal-header-vault">
      <h2 class="modal-title">Add Vault</h2>
      <Button variant="ghost" class="close-button" onclick={() => (open = false)} iconLeft={X} />
    </header>

    <form
      onsubmit={() => {
        window.api.vault.create(vaultName)
        refreshVault(vaultName)
      }}
      class="modal-form"
    >
      <div class="form-fields">
        <div class="field">
          <label for="vaultTitle" class="field-label">Vault Name</label>
          <Input
            id="vaultTitle"
            required
            placeholder="Enter a name for the vault"
            class="modal-input"
            bind:value={vaultName}
          />
        </div>
      </div>

      <footer class="modal-footer-vault">
        <Button variant="filled" type="submit">Submit</Button>
      </footer>
    </form>
  </div>
</Modal>
