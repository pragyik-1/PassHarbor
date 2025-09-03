<script lang="ts">
  import { Card, Divider, Grid, Input, Switch } from '../../lib'
  import { settings as _settings } from '../../global.svelte'

  const settings = _settings.value
  let newSettings: Settings = JSON.parse(JSON.stringify(settings))
</script>

<div style="margin-top: 6rem; width: 70%; margin-left: auto; margin-right: auto;">
  <Card>
    <h3>General Settings</h3>
    <Divider />
    <Grid row justify="between">
      <p>Master Password Enabled</p>
      <Switch bind:checked={newSettings.masterPassword.enabled} />
    </Grid>
    {#if newSettings.masterPassword.enabled}
      <Grid row justify="between">
        <p>Master Password Save Location</p>
        <Grid col>
          <Input
            type="directory"
            webkitdirectory
            bind:value={newSettings.masterPassword.saveLocation}
          />
          <p class="description-text">currently: {newSettings.masterPassword.saveLocation}</p>
        </Grid>
      </Grid>
    {/if}
    <Grid row justify="between">
      <p>Dark Mode</p>
      <Switch bind:checked={newSettings.darkMode} />
    </Grid>
    <Divider />
    <h3>Vault Settings</h3>
    <Grid justify="between"></Grid>
  </Card>
</div>
