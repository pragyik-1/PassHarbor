<script lang="ts">
  import { EyeIcon, EyeOffIcon, X } from 'lucide-svelte'
  import { Button, Grid, Input } from '../..'
  import Modal from '../ui/Modal.svelte'
  import './AddPasswordModal.css'
  import { currentVault, generatePassword, refreshVault } from '../../../global.svelte'
  import { analyzePasswordSecurity, getStrengthColor } from '../../../utils/passwordStrength'

  let { open = $bindable(false) } = $props()
  let password = $state('')
  let passwordTitle = $state('')
  let passwordVisible = $state(false)
</script>

<Modal style="border-radius: 1rem" bind:isOpen={open} class="modal">
  <div class="modal-content">
    <header class="modal-header">
      <h2 class="modal-title">Add Password</h2>
      <Button variant="ghost" class="close-button" onclick={() => (open = false)} iconLeft={X} />
    </header>

    <form
      onsubmit={async (e) => {
        e.preventDefault()
        await window.api.password.save(currentVault.value, passwordTitle, password)
        refreshVault(currentVault.value)
        open = false
        password = ''
        passwordTitle = ''
      }}
      class="modal-form"
    >
      <div class="form-fields">
        <div class="field">
          <label for="passwordTitle" class="field-label">Password Title</label>
          <Input
            id="passwordTitle"
            required
            placeholder="Enter a title for this password"
            class="modal-input"
            bind:value={passwordTitle}
          />
        </div>

        <div class="field">
          <label for="password" class="field-label">Password</label>
          <Grid row justify="between">
            <Input
              id="password"
              type={passwordVisible ? 'text' : 'password'}
              required
              placeholder="Enter your password"
              class="modal-input"
              style="width: 80%;"
              bind:value={password}
            />
            <Button
              variant="ghost"
              iconLeft={passwordVisible ? EyeIcon : EyeOffIcon}
              onclick={() => (passwordVisible = !passwordVisible)}
            />
          </Grid>
        </div>
        <p style={`color: ${getStrengthColor(analyzePasswordSecurity(password).strength)};`}>
          Password strength: {analyzePasswordSecurity(password).strength}
        </p>
      </div>

      <footer class="modal-footer">
        <Button
          style="margin-right: auto;"
          variant="outlined"
          class="modal-generate-password"
          onclick={() => (password = generatePassword())}>Generate Password</Button
        >
        <Button variant="filled" type="submit">Submit</Button>
      </footer>
    </form>
  </div>
</Modal>
