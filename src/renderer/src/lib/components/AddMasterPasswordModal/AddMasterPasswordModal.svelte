<script lang="ts">
  import { Button, Input } from '../..'
  import Modal from '../ui/Modal.svelte'

  let { open = $bindable(false) } = $props()
  let passwordName = $state('')
</script>

<Modal bind:isOpen={open} class="modal">
  <div class="modal-content">
    <form
      onsubmit={async () => {
        try {
          await window.api.masterPassword.create(passwordName)
          window.location.reload()
        } catch (error) {
          console.error('Error writing file:', error)
        }
      }}
      class="modal-form"
    >
      <div class="form-fields">
        <div class="field">
          <label for="masterPassword" class="field-label">Master Password </label>
          <Input
            id="masterPassword"
            required
            placeholder="Input your master password"
            class="modal-input"
            bind:value={passwordName}
          />
        </div>
      </div>
      <Button variant="filled" type="submit">Submit</Button>
    </form>
  </div>
</Modal>
