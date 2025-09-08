<script lang="ts">
  import { CheckIcon, XIcon } from 'lucide-svelte'
  import { Grid, List } from '../../../lib'
  let { currentPasswordSecurity = $bindable() } = $props()
</script>

{#snippet PasswordCheck(label: string, passed: boolean, warning = false)}
  <Grid row justify="between" align="center" fullWidth>
    <span>{label}</span>
    {#if passed}
      <CheckIcon color="green" />
    {:else if warning}
      <XIcon color="orange" />
    {:else}
      <XIcon color="red" />
    {/if}
  </Grid>
{/snippet}

<List style="width: 100%;">
  <p class="password-details-header" style="margin: 0 0 0.5rem 0;">Security Check</p>

  <Grid col gap={1} style="font-size: 0.9rem; color: var(--text-secondary);" fullWidth>
    {@render PasswordCheck('At least 8 characters', currentPasswordSecurity.length >= 8)}
    {@render PasswordCheck('Has uppercase letter', currentPasswordSecurity.hasUpper)}
    {@render PasswordCheck('Has lowercase letter', currentPasswordSecurity.hasLower)}
    {@render PasswordCheck('Has number', currentPasswordSecurity.hasDigit)}
    {@render PasswordCheck('Has symbol', currentPasswordSecurity.hasSymbol)}
    {@render PasswordCheck(
      'No repeated characters',
      !currentPasswordSecurity.hasRepetition,
      currentPasswordSecurity.hasRepetition
    )}
  </Grid>
</List>
