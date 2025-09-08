<script lang="ts">
  import { Button, Input } from '../..'
  import Modal from '../ui/Modal.svelte'

  let { open = $bindable(false) } = $props()
  let passwordName = $state('')
  let storedPassword = $state('')
</script>

<Modal bind:isOpen={open} class="modal">
  <div class="modal-content">
    <form
      onsubmit={async (e) => {
        e.preventDefault()
        storedPassword = await window.api.masterPassword.get()

        if (passwordName === storedPassword) {
          open = false
          sessionStorage.setItem('haslogged', 'true')
        } else {
          console.log('Incorrect password')
        }
      }}
      class="modal-form"
    >
      <div class="form-fields">
        <div class="field">
          <label for="masterPassword" class="field-label">Master Password</label>
          <Input
            id="masterPassword"
            type="password"
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
