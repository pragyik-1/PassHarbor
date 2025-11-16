<script lang="ts">
  import { Card, Accordion, Input, Switch, Button, Grid } from '../../lib'
  import { settings as globalSettings, vaults } from '../../global.svelte'
  import SettingsItem from './SubComponents/SettingsItem.svelte'
  import DirInput from './SubComponents/DirInput.svelte'
  import SettingsCategory from './SubComponents/SettingsCategory.svelte'
  import AddMasterPasswordModal from '../../lib/components/AddMasterPasswordModal/AddMasterPasswordModal.svelte'
  import { XIcon } from 'lucide-svelte'
  import "./Settings.css"

  let newSettings: Settings = $state(JSON.parse(JSON.stringify(globalSettings.value)))
  let modalOpen = $state(false)

  async function saveSettings() {
    const settingsToSave = JSON.parse(JSON.stringify(newSettings))
    await window.api.settings.set(settingsToSave)
    globalSettings.value = settingsToSave
  }
</script>

<main class="settings-page">
  <h1>Settings</h1>
  <form
    class="settings-form"
    onsubmit={async (e) => {
      e.preventDefault()
      await saveSettings()
    }}
  >
    <Grid col gap={6}>
      <Card fullWidth>
        <SettingsCategory title="General Settings" />
        <SettingsItem title="Master Password Enabled">
          <Switch bind:checked={newSettings.masterPassword.enabled} />
        </SettingsItem>
        {#if newSettings.masterPassword.enabled}
          <SettingsItem title="Master Password Save File">
            <Input type="file" bind:value={newSettings.masterPassword.saveLocation} />
          </SettingsItem>
          <SettingsItem title="Save Master Password: ">
            <Button onclick={() => (modalOpen = !modalOpen)}>Submit</Button>
          </SettingsItem>
        {/if}
        <SettingsItem title="Dark Mode">
          <Switch bind:checked={newSettings.darkMode} />
        </SettingsItem>
      </Card>
      <Card fullWidth>
        <SettingsCategory title="Password Generator Settings" />
        <SettingsItem title="Length">
          <Input id="password-length" type="number" bind:value={newSettings.passwordGenerator.length} />
        </SettingsItem>
        <SettingsItem title="Symbols to be used (other than a-z, A-Z, 0-9)">
          <Input bind:value={newSettings.passwordGenerator.symbolsToUse} />
        </SettingsItem>
        <SettingsItem title="Use Lowercase">
          <Switch bind:checked={newSettings.passwordGenerator.lower} />
        </SettingsItem>
        <SettingsItem title="Use Uppercase">
          <Switch bind:checked={newSettings.passwordGenerator.upper} />
        </SettingsItem>
        <SettingsItem title="Use Digits">
          <Switch bind:checked={newSettings.passwordGenerator.digits} />
        </SettingsItem>
        <SettingsItem title="Use Symbols">
          <Switch bind:checked={newSettings.passwordGenerator.symbols} />
        </SettingsItem>
      </Card>
      <Card fullWidth>
        <SettingsCategory title="Vault Settings" />
        <SettingsItem title="Vault Save Location">
          <DirInput id="vault-location" bind:value={newSettings.vaults.saveLocation} />
        </SettingsItem>
        <SettingsItem title="Main Vault">
          <Input bind:value={newSettings.vaults.main} />
        </SettingsItem>
        <Accordion title="All Vaults">
          {#each vaults.value as _, i}
            <SettingsItem title={''}>
              <Input bind:value={vaults.value[i]} />
              <Button
                size="lg"
                iconLeft={XIcon}
                iconSize={20}
                variant="ghost"
                style="margin-left: auto;"
                tooltip="Delete Vault"
              />
            </SettingsItem>
          {/each}
        </Accordion>
      </Card>
    </Grid>
    <Button type="submit" size="lg" style="margin-top: 1.5rem; width: 100%;"
      >Save All Settings</Button
    >
  </form>
</main>

<AddMasterPasswordModal open={modalOpen} />

<style>
  h1 {
    margin-bottom: 1.5rem;
  }
</style>
