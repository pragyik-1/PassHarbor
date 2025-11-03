<script lang="ts">
  import { Button, Input, Switch } from '../../../lib'
  import SettingsCategory from './SettingsCategory.svelte'
  import SettingsItem from './SettingsItem.svelte'

  let { settings = $bindable(), modalOpen } = $props()
</script>

<SettingsCategory title="General Settings" />
<SettingsItem title="Master Password Enabled">
  <Switch bind:checked={settings.masterPassword.enabled} />
</SettingsItem>
{#if settings.masterPassword.enabled}
  <SettingsItem title="Master Password Save File">
    <Input type="file" bind:value={settings.masterPassword.saveLocation} />
  </SettingsItem>
  <SettingsItem title="Save Master Password: ">
    <Button onclick={() => (modalOpen = !modalOpen)}>Submit</Button>
  </SettingsItem>
{/if}
<SettingsItem title="Dark Mode">
  <Switch bind:checked={settings.darkMode} />
</SettingsItem>