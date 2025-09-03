<script lang="ts">
  import {
    CheckIcon,
    CopyIcon,
    EditIcon,
    EyeIcon,
    EyeOffIcon,
    KeyIcon,
    PlusIcon,
    RefreshCwIcon,
    SaveIcon,
    ShieldIcon,
    TrashIcon
  } from 'lucide-svelte'
  import { Card, Grid, Input, List, ListButton, Button } from '../../lib'
  import { useTimeout } from '../../lib/utils'
  import {
    currentVault,
    generatePassword,
    passwords,
    refreshVault,
    vaults
  } from '../../global.svelte'
  import { analyzePasswordSecurity, getStrengthColor } from '../../utils/passwordStrength'
  import './Home.css'
  import PasswordChecks from './SubComponents/PasswordChecks.svelte'
  import AddPasswordModal from '../../lib/components/AddPasswordModal/AddPasswordModal.svelte'

  let password = window.api.password

  let selectedPassword = $state('')
  let copiedStates = $state<Record<string, boolean>>({})
  let justCopied = $state(false)
  let addPasswordModalOpen = $state(false)
  let editMode = $state(false)
  let showPassword = $state(false)
  let originalPassword = $state('')

  let currentPasswordSecurity = $derived(analyzePasswordSecurity(passwords.value[selectedPassword]))

  function handleEditToggle() {
    if (!editMode) {
      originalPassword = passwords.value[selectedPassword]
    } else {
      passwords.value[selectedPassword] = originalPassword
    }
    editMode = !editMode
  }

  async function handleSave() {
    originalPassword = passwords.value[selectedPassword]
    editMode = false
    await password.change(currentVault.value, selectedPassword, passwords.value[selectedPassword])
  }

  async function handleDelete() {
    await password.delete(currentVault.value, selectedPassword)
    refreshVault(currentVault.value)
    selectedPassword = ''
  }

  async function copyToClipboardAndHandleState(password: string, title: string) {
    {
      await navigator.clipboard.writeText(password)
      copiedStates = { ...copiedStates, [title]: true }

      useTimeout(() => {
        copiedStates = { ...copiedStates, [title]: false }
      }, 1000)
    }
  }

  $effect(() => {
    window.api.vault.getAll().then((v) => (vaults.value = v))
    password.all(currentVault.value).then((p) => (passwords.value = p))
  })

  $inspect(currentVault)
</script>

<div style="margin-bottom: 4.5rem;"></div>

<main class="home-page">
  <Grid row gap={4} align="start" style="height: 100%;">
    <!-- Sidebar -->
    <Grid col gap={4} align="start" class="sidebar-top grid">
      <Grid row fullWidth gap={2} align="center">
        <Input placeholder="Search Password" style="flex-grow: 1; border-radius:0.5rem " />
        <Button
          size="md"
          iconLeft={PlusIcon}
          iconSize={23}
          tooltip="Add Password to current Vault"
          style="border-radius: 0.5rem;"
          onclick={() => (addPasswordModalOpen = true)}
        />
      </Grid>

      <Card class="password-list-card" fullWidth>
        <Grid>
          <ShieldIcon size={25} style="margin-left: 1rem;" />
          <p class="header-text">Saved Passwords</p>
        </Grid>
        <List style="width: 100%; flex: 1; overflow-y: auto;">
          <!-- No Passwords Placeholder -->
          {#if Object.keys(passwords.value).length === 0}
            <div class="no-passwords-text">
              No passwords saved yet. Click <strong>+</strong> to add one.
            </div>
            <!-- No Passwords Placeholder -->
          {:else}
            {#each Object.entries(passwords.value) as [title, password]}
              <ListButton
                class="list-button"
                icon={KeyIcon}
                onclick={() => {
                  selectedPassword = title
                }}
              >
                {title}
                <Button
                  size="sm"
                  iconLeft={copiedStates[title] ? CheckIcon : CopyIcon}
                  iconSize={20}
                  tooltip="Copy Password to Clipboard"
                  style="margin-left: auto;"
                  variant="link"
                  onclick={async () => await copyToClipboardAndHandleState(password, title)}
                />
              </ListButton>
            {/each}
          {/if}
        </List>
      </Card>
    </Grid>

    <!-- Details Panel -->
    <Grid col gap={4} align="start" class="password-details grid">
      <Card fullWidth class="password-details card">
        <Grid col align="start" style=" height: 100%;">
          <p class="password-details-header">
            {selectedPassword || 'Select a Password'}
          </p>

          {#if selectedPassword}
            <div class="password-display" style="width: 100%;">
              <Input
                bind:value={passwords.value[selectedPassword]}
                readonly={!editMode}
                type={showPassword ? 'text' : 'password'}
                class="password-input"
                fullWidth
              />
            </div>
            <div class="password-actions">
              <Button
                iconLeft={showPassword ? EyeIcon : EyeOffIcon}
                variant="ghost"
                tooltip={showPassword ? 'Hide password' : 'Show password'}
                onclick={() => (showPassword = !showPassword)}
              />
              <Button
                iconRight={justCopied ? CheckIcon : CopyIcon}
                variant="ghost"
                tooltip="Copy password"
                onclick={async () => {
                  justCopied = true
                  await navigator.clipboard.writeText(passwords.value[selectedPassword])

                  useTimeout(() => {
                    justCopied = false
                  }, 1000)
                }}
              />
              {#if editMode}
                <Button
                  iconLeft={RefreshCwIcon}
                  variant="ghost"
                  tooltip="Generate new password"
                  onclick={() => (passwords.value[selectedPassword] = generatePassword())}
                />
              {/if}
            </div>

            <p
              style={`color: ${getStrengthColor(currentPasswordSecurity.strength)}; margin-top: 1rem;`}
            >
              Password Strength: {currentPasswordSecurity.strength}
            </p>

            <PasswordChecks bind:currentPasswordSecurity={passwords.value[selectedPassword]} />

            <Grid row fullWidth justify="between" style="margin-top: auto">
              <Button
                class={`edit-button ${editMode ? 'active' : ''}`}
                size="md"
                iconLeft={EditIcon}
                iconSize={20}
                tooltip="Edit Password"
                variant="outlined"
                onclick={handleEditToggle}
              >
                {editMode ? 'Cancel' : 'Edit'}
              </Button>
              {#if editMode}
                <Button
                  class="save-button"
                  size="md"
                  iconLeft={SaveIcon}
                  iconSize={20}
                  tooltip="Save Password"
                  variant="outlined"
                  onclick={async () => await handleSave()}
                >
                  Save
                </Button>
              {/if}
              <Button
                class="delete-button"
                size="md"
                iconLeft={TrashIcon}
                iconSize={20}
                tooltip="Delete Password"
                variant="outlined"
                onclick={async () => await handleDelete()}
              >
                Delete
              </Button>
            </Grid>
          {:else}
            <div style="padding: 2rem; text-align: center; color: gray;">
              Select a password to view its details.
            </div>
          {/if}
        </Grid>
      </Card>
    </Grid>
  </Grid>
</main>

<AddPasswordModal bind:open={addPasswordModalOpen} />
