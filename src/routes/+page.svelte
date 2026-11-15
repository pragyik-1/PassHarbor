<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { goto } from '$app/navigation'
  import { Button, Card, Input } from '@hermitk/bluenite'
  import {
    unlock,
    unlockWithKeychain,
    getSession,
    listVaults,
    isInvalidMasterKeyError,
    isKeychainStaleError,
    isKeychainUnavailableError,
    isTooManyAttemptsError,
    type VaultSummary,
  } from '#lib/api.ts'
  import {
    EyeIcon,
    EyeOffIcon,
    KeyRoundIcon,
    LockIcon,
    TriangleAlertIcon,
    PlusIcon,
    ArrowLeftRightIcon,
  } from 'lucide-svelte'
  import BrandMark from '#lib/components/BrandMark.svelte'
  import ThemeToggle from '#lib/components/ThemeToggle.svelte'
  import VaultSelectionModal from '#lib/components/VaultSelectionModal.svelte'
  import CreateVaultModal from '#lib/components/CreateVaultModal.svelte'
  import { clearManualLock, isManualLockPending } from '#lib/utils.svelte.ts'

  let passphrase = $state('')
  let error = $state('')
  let keychainNotice = $state('')
  let busy = $state(false)
  let showPassphrase = $state(false)
  let capsLock = $state(false)
  let passphraseInput = $state<HTMLInputElement | null>(null)
  let manualLockPending = $state(isManualLockPending())

  let lockoutRemaining = $state(0)
  let lockoutInterval: ReturnType<typeof setInterval> | null = null
  const isLockedOut = $derived(lockoutRemaining > 0)

  let vaults = $state<VaultSummary[]>([])
  let selectedVaultId = $state<string | null>(null)
  let vaultModalOpen = $state(false)
  let createModalOpen = $state(false)

  const selectedVault = $derived(vaults.find((v) => v.id === selectedVaultId) ?? null)

  const passphraseHelper = $derived(
    vaults.length === 0
      ? "If this is your first time here, this will be your master passphrase and it can't be recovered."
      : manualLockPending
        ? 'Vault locked. Enter your master passphrase to unlock it.'
        : selectedVault?.keychain_enabled
          ? 'Keychain unlock is enabled. Leave this empty and press Unlock to use it, or enter the master passphrase.'
          : `Enter the master passphrase for ${selectedVault?.name ?? 'this vault'}.`,
  )

  async function tryKeychainUnlock(vaultId: string | null): Promise<boolean> {
    if (vaultId === null || busy) return false
    const vault = vaults.find((v) => v.id === vaultId)
    if (!vault?.keychain_enabled) return false

    error = ''
    keychainNotice = ''
    busy = true
    try {
      await unlockWithKeychain(vaultId)
      clearManualLock()
      manualLockPending = false
      passphrase = ''
      await goto('/dashboard', { replace: true })
      return true
    } catch (cause) {
      if (isKeychainStaleError(cause)) {
        keychainNotice =
          'The saved keychain entry is no longer valid. Enter your master passphrase.'
        vaults = vaults.map((v) => (v.id === vaultId ? { ...v, keychain_enabled: false } : v))
      } else if (isKeychainUnavailableError(cause)) {
        keychainNotice = 'The system keychain is unavailable. Enter your master passphrase.'
      } else {
        error = describeError(cause)
      }
      passphraseInput?.focus()
      return false
    } finally {
      busy = false
    }
  }

  function startLockoutCountdown(seconds: number) {
    if (lockoutInterval !== null) clearInterval(lockoutInterval)
    lockoutRemaining = seconds
    lockoutInterval = setInterval(() => {
      lockoutRemaining--
      if (lockoutRemaining <= 0) {
        if (lockoutInterval !== null) clearInterval(lockoutInterval)
        lockoutInterval = null
        lockoutRemaining = 0
        error = ''
      } else {
        error = `Too many failed attempts. Please try again after ${lockoutRemaining} seconds`
      }
    }, 1000)
  }

  onMount(async () => {
    try {
      const session = await getSession()
      if (session.unlocked) {
        await goto('/dashboard', { replace: true })
        return
      }
    } catch {
      // Ignored: vault is locked
    }

    try {
      vaults = await listVaults()
      const defaultVault = vaults.find((v) => v.is_default) ?? vaults[0] ?? null
      if (defaultVault) {
        selectedVaultId = defaultVault.id
      }
      if (!manualLockPending && defaultVault?.keychain_enabled) {
        const unlocked = await tryKeychainUnlock(defaultVault.id)
        if (unlocked) return
      }
      if (vaults.length > 1) {
        vaultModalOpen = true
      }
    } catch {
      // Ignored: failed to load vaults
    }

    passphraseInput?.focus()
  })

  onDestroy(() => {
    if (lockoutInterval !== null) clearInterval(lockoutInterval)
  })

  function describeError(cause: unknown): string {
    return isInvalidMasterKeyError(cause)
      ? 'Incorrect passphrase. Please try again.'
      : String(cause)
  }

  function checkCapsLock(event: KeyboardEvent) {
    capsLock = event.getModifierState('CapsLock')
  }

  function toggleVisibility() {
    showPassphrase = !showPassphrase
    passphraseInput?.focus()
  }

  function handleVaultSelected(vault: VaultSummary) {
    selectedVaultId = vault.id
    error = ''
    keychainNotice = ''
    passphrase = ''
    if (!manualLockPending && vault.keychain_enabled) {
      void tryKeychainUnlock(vault.id)
    } else {
      setTimeout(() => passphraseInput?.focus(), 50)
    }
  }

  function handleVaultsChanged(updated: VaultSummary[]) {
    vaults = updated
    if (selectedVaultId === null) {
      const defaultVault = updated.find((v) => v.is_default) ?? updated[0] ?? null
      if (defaultVault) selectedVaultId = defaultVault.id
    }
  }

  async function handleVaultCreated(created: VaultSummary) {
    try {
      vaults = await listVaults()
    } catch {
      // Best effort refresh
    }
    selectedVaultId = created.id
    error = ''
    passphrase = ''
    setTimeout(() => passphraseInput?.focus(), 50)
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault()
    if (busy) return

    if (passphrase.length === 0) {
      if (!manualLockPending && selectedVault?.keychain_enabled) {
        const unlocked = await tryKeychainUnlock(selectedVaultId)
        if (unlocked) return
      }
      if (isLockedOut) return
      if (keychainNotice === '') {
        error = 'Enter your master passphrase.'
      }
      passphraseInput?.focus()
      return
    }

    if (isLockedOut) return

    error = ''
    keychainNotice = ''
    busy = true
    try {
      await unlock(passphrase, selectedVaultId)
      clearManualLock()
      manualLockPending = false
      passphrase = ''
      await goto('/dashboard', { replace: true })
    } catch (cause) {
      error = describeError(cause)
      passphraseInput?.focus()
      if (isTooManyAttemptsError(cause)) {
        const match = String(cause).match(/(\d+)\s+seconds/)
        const secs = match ? parseInt(match[1], 10) : 30
        startLockoutCountdown(secs)
      }
    } finally {
      busy = false
    }
  }
</script>

<svelte:head>
  <title>Unlock · PassHarbor</title>
</svelte:head>

<div class="page">
  <header class="topbar">
    <a class="brand" href="/" aria-label="PassHarbor home">
      <BrandMark />
      <span class="brand-name">PassHarbor</span>
    </a>

    <ThemeToggle />
  </header>

  <main class="main">
    <div class="auth">
      <Card class="login-card" noPadding>
        {#snippet header()}
          <div class="form-head">
            <h2>Unlock {selectedVault ? selectedVault.name : 'your vault'}</h2>
            {#if vaults.length > 1}
              <div class="switch-vault-wrapper">
                <Button
                  type="button"
                  variant="outline"
                  color="primary"
                  size="sm"
                  class="switch-vault-btn"
                  onclick={() => (vaultModalOpen = true)}
                >
                  {#snippet icon()}
                    <ArrowLeftRightIcon size={14} />
                  {/snippet}
                  Switch vault
                </Button>
              </div>
            {/if}
          </div>
        {/snippet}
        <form class="login-form" onsubmit={submit}>
          <div class="field">
            <div class="field-head">
              <label for="passphrase">
                <KeyRoundIcon size={14} style="margin-right: 4px" />
                Master passphrase
              </label>
              <Button
                type="button"
                variant="ghost"
                color="secondary"
                size="sm"
                class="reveal-toggle"
                aria-label={showPassphrase ? 'Hide passphrase' : 'Show passphrase'}
                aria-pressed={showPassphrase}
                onclick={toggleVisibility}
              >
                {#snippet icon()}
                  {#if showPassphrase}
                    <EyeOffIcon size={18} />
                  {:else}
                    <EyeIcon size={18} />
                  {/if}
                {/snippet}
              </Button>
            </div>

            <Input
              id="passphrase"
              bind:ref={passphraseInput}
              bind:value={passphrase}
              class="login-input"
              helperText={passphraseHelper}
              type={showPassphrase ? 'text' : 'password'}
              placeholder="Enter your master passphrase"
              disabled={busy}
              error={error || undefined}
              onkeydown={checkCapsLock}
              onkeyup={checkCapsLock}
              onblur={() => {
                capsLock = false
              }}
            />

            {#if capsLock}
              <p class="caps-warning" role="status">
                <TriangleAlertIcon size={14} />
                Caps Lock is on
              </p>
            {/if}

            {#if keychainNotice}
              <p class="keychain-notice" role="status">
                <KeyRoundIcon size={14} />
                {keychainNotice}
              </p>
            {/if}
          </div>

          <Button type="submit" size="md" class="login-submit" loading={busy}>
            {busy ? 'Unlocking…' : `Unlock ${selectedVault?.name ?? 'vault'}`}
          </Button>
        </form>
      </Card>
    </div>
  </main>

  <footer class="footer">
    <LockIcon size={14} />
    Encrypted locally with SQLCipher. Nothing leaves this device.
    <Button
      type="button"
      variant="ghost"
      color="secondary"
      size="sm"
      class="new-vault-btn"
      onclick={() => (createModalOpen = true)}
    >
      {#snippet icon()}
        <PlusIcon size={13} />
      {/snippet}
      New vault
    </Button>
  </footer>
</div>

<VaultSelectionModal
  bind:open={vaultModalOpen}
  bind:selectedVaultId
  bind:vaults
  onSelect={handleVaultSelected}
  onVaultsChanged={handleVaultsChanged}
/>

<CreateVaultModal bind:open={createModalOpen} onCreated={handleVaultCreated} />

<style>
  .page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    min-height: 100dvh;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1.25rem 1.5rem;
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: 0.65rem;
    color: var(--text);
    text-decoration: none;
  }

  .brand-name {
    font-size: 1rem;
    font-weight: 650;
    letter-spacing: -0.015em;
  }

  .main {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem 1.5rem 2.5rem;
  }

  .auth {
    width: min(100%, 26rem);
  }

  .auth :global(.login-card) {
    border-radius: var(--round-xl);
    height: 100%;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1.5rem;
  }

  .form-head {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
  }

  .form-head h2 {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 650;
    letter-spacing: -0.015em;
    text-align: center;
  }

  .switch-vault-wrapper {
    margin-top: 0.35rem;
    display: flex;
    justify-content: center;
  }

  .auth :global(.switch-vault-btn) {
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .field {
    display: flex;
    flex-direction: column;
  }

  .field-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    min-height: 1.75rem;
    margin-bottom: 0.4rem;
  }

  .field-head label {
    color: var(--primary-text);
    font-size: 0.875rem;
    font-weight: 500;
  }

  .field :global(.login-input) {
    padding: 0.65rem 0.75rem;
    font-size: 1rem;
  }

  .caps-warning {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0.5rem 0 0;
    color: var(--warn);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .keychain-notice {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0.5rem 0 0;
    color: var(--muted-text);
    font-size: 0.75rem;
    line-height: 1.4;
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 0 1.5rem 1.25rem;
    color: var(--muted-text);
    font-size: 0.72rem;
  }

  .footer :global(.new-vault-btn) {
    font-size: 0.72rem;
    opacity: 0.6;
  }

  .footer :global(.new-vault-btn:hover) {
    opacity: 1;
  }

  :global(.theme-toggle) {
    width: 2.25rem;
    height: 2.25rem;
    padding: 0;
    border-radius: 999px;
    color: var(--secondary-text);
  }

  :global(.theme-toggle:hover) {
    color: var(--primary-text);
  }

  @media (max-height: 660px) {
    .topbar {
      padding: 0.85rem 1.25rem;
    }

    .main {
      padding: 0.75rem 1.25rem 1rem;
    }

    .login-form {
      gap: 0.9rem;
      padding: 1.15rem 1.25rem;
    }

    .footer {
      padding-bottom: 0.75rem;
    }
  }

  @media (max-width: 420px) {
    .topbar {
      padding: 1rem 1.1rem;
    }

    .main {
      padding: 1rem 1.1rem 2rem;
    }

    .login-form {
      padding: 1.25rem;
    }
  }
</style>
