<script lang="ts">
  import { Card, Accordion, Input, Switch } from '../../lib'
  import { settings as _settings, vaults } from '../../global.svelte'
  import SettingsItem from './SubComponents/SettingsItem.svelte'
  import DirInput from './SubComponents/DirInput.svelte'
  import SettingsCategory from './SubComponents/SettingsCategory.svelte'

  const settings = _settings.value
  let newSettings: Settings = JSON.parse(JSON.stringify(settings))
</script>

<div style="margin-top: 6rem; width: 70%; margin-left: auto; margin-right: auto;">
  <Card>
    <SettingsCategory title="General Settings" />
    <SettingsItem title="Master Password Enabled">
      <Switch bind:checked={newSettings.masterPassword.enabled} />
    </SettingsItem>
    {#if newSettings.masterPassword.enabled}
      <SettingsItem title="Master Password Save Location">
        <DirInput bind:value={newSettings.masterPassword.saveLocation} />
      </SettingsItem>
    {/if}
    <SettingsItem title="Dark Mode">
      <Switch bind:checked={newSettings.darkMode} />
    </SettingsItem>
    <SettingsCategory title="Vault Settings" />
    <SettingsItem title="Vault Save Location">
      <DirInput bind:value={newSettings.vaults.saveLocation} />
    </SettingsItem>
    <SettingsItem title="Main Vault">
      <Input bind:value={newSettings.vaults.main} />
    </SettingsItem>
    <Accordion title="All Vaults">
      {#each vaults.value as vault}
        <SettingsItem title={vault}>
          <Input bind:value={vault} />
        </SettingsItem>
      {/each}
    </Accordion>
  </Card>
</div>
