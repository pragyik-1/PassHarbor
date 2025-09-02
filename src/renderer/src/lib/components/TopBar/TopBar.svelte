<script lang="ts">
  import { Vault, PlusIcon, KeyIcon, SettingsIcon } from 'lucide-svelte'
  import { AppBar, Button, Dropdown, List, ListButton } from '../..'
  import { vaults, currentVault } from '../../../global.svelte'
  import { colors } from '../../utils'
  import AddMasterPasswordModal from '../AddMasterPasswordModal/AddMasterPasswordModal.svelte'
  import AddVaultModal from '../AddVaultModal/AddVaultModal.svelte'

  let addVaultModalOpen = $state(false)
  let addMasterPasswordModalOpen = $state(false)
  let dropdownOpen = $state(false)
  let settingsDropdownOpen = $state(false)
</script>

{#snippet DropdownButton()}
  <Button
    variant="ghost"
    iconLeft={Vault}
    iconSize={25}
    onclick={() => (dropdownOpen = !dropdownOpen)}
    >Vaults
  </Button>
{/snippet}

{#snippet SettingsButton()}
  <Button
    variant="ghost"
    iconLeft={SettingsIcon}
    iconSize={25}
    onclick={() => (settingsDropdownOpen = !settingsDropdownOpen)}
  ></Button>
{/snippet}

<AppBar style="padding:0.25rem" fixed>
  <div>
    <p style="font-weight: 600; font-size: 1.15rem;">Pass Harbor</p>
  </div>
  <div>
    <Dropdown wrapperStyle="margin-right: 3rem;" anchor={DropdownButton} bind:open={dropdownOpen}>
      <List>
        {#each vaults.value as vault}
          <ListButton
            style={`color:${vault == currentVault.value && colors.info + `;background-color: ${colors.hover} `}`}
            icon={Vault}
            onclick={() => {
              currentVault.value = vault
              localStorage.setItem('currentVault', vault)
            }}>{vault.replace(/\n/g, ' ')}</ListButton
          >
        {/each}
        <ListButton icon={PlusIcon} onclick={() => (addVaultModalOpen = true)}>Vault</ListButton>
      </List>
    </Dropdown>
    <Dropdown
      wrapperStyle="margin-right: 2rem;"
      anchor={SettingsButton}
      bind:open={settingsDropdownOpen}
    >
      <List>
        <ListButton icon={KeyIcon} onclick={() => (addMasterPasswordModalOpen = true)}
          >Master Password</ListButton
        >
      </List>
    </Dropdown>
  </div>
</AppBar>

<AddVaultModal bind:open={addVaultModalOpen} />
<AddMasterPasswordModal bind:open={addMasterPasswordModalOpen} />
