<script lang="ts">
  import { Card, Accordion, Input, Switch, Grid, Button } from '../../lib'
  import { settings as _settings, vaults } from '../../global.svelte'
  import SettingsItem from './SubComponents/SettingsItem.svelte'
  import DirInput from './SubComponents/DirInput.svelte'
  import SettingsCategory from './SubComponents/SettingsCategory.svelte'
  import { XIcon } from 'lucide-svelte'

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
          <Grid>
            <p style="margin-right: 1rem;">Change name to:</p>
            <Input bind:value={vault} />
            <Button
              size="lg"
              iconLeft={XIcon}
              iconSize={20}
              variant="ghost"
              style="margin-left: auto;"
              tooltip="Delete Vault"
            />
          </Grid>
        </SettingsItem>
      {/each}
    </Accordion>
    <SettingsCategory title="Password Generator Settings" />
    <SettingsItem title="Length">
      <Input type="number" bind:value={newSettings.passwordGenerator.length} />
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
</div>
