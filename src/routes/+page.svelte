<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { Button, Card, Input } from "@hermitk/bluenite";
  import { unlock, getSession, isInvalidMasterKeyError } from "#lib/api.ts";
  import {
    EyeIcon,
    EyeOffIcon,
    KeyRoundIcon,
    LockIcon,
    TriangleAlertIcon,
  } from "lucide-svelte";
  import BrandMark from "#lib/components/BrandMark.svelte";
  import ThemeToggle from "#lib/components/ThemeToggle.svelte";

  const STORAGE_ATTEMPTS_KEY = "ph_unlock_attempts";
  const STORAGE_DELAY_KEY = "ph_unlock_delay_ms";
  const THROTTLE_THRESHOLD = 5;
  const INITIAL_DELAY_MS = 30_000;

  function readStoredInt(key: string, fallback: number): number {
    const raw = sessionStorage.getItem(key);
    if (raw === null) return fallback;
    const n = parseInt(raw, 10);
    return Number.isFinite(n) ? n : fallback;
  }

  let passphrase = $state("");
  let error = $state("");
  let busy = $state(false);
  let showPassphrase = $state(false);
  let capsLock = $state(false);
  let passphraseInput = $state<HTMLInputElement | null>(null);

  // Persisted across in-session navigation so reloading the page does not reset
  // the throttle. Both values are cleared when the webview session ends.
  let unlockAttempts = $state(readStoredInt(STORAGE_ATTEMPTS_KEY, 0));
  let requiredTimeBeforeUnlock = $state(
    readStoredInt(STORAGE_DELAY_KEY, INITIAL_DELAY_MS),
  );

  let lockoutTimer: ReturnType<typeof setTimeout> | null = null;

  const isLockedOut = $derived(unlockAttempts >= THROTTLE_THRESHOLD);

  function startLockoutTimer() {
    if (lockoutTimer !== null) clearTimeout(lockoutTimer);
    lockoutTimer = setTimeout(() => {
      lockoutTimer = null;
      unlockAttempts = 0;
      requiredTimeBeforeUnlock *= 2;
      sessionStorage.setItem(STORAGE_ATTEMPTS_KEY, "0");
      sessionStorage.setItem(
        STORAGE_DELAY_KEY,
        String(requiredTimeBeforeUnlock),
      );
      error = "";
    }, requiredTimeBeforeUnlock);
  }

  onMount(async () => {
    // Restore an active lockout: the timer was lost when the page reloaded, so
    // show the message and restart it from however long is left. Without a
    // stored start-time this is approximate — the full delay restarts, which is
    // conservative (safer than allowing a login too early).
    if (isLockedOut) {
      error = `Too many failed attempts. Please try again after ${requiredTimeBeforeUnlock / 1000} seconds.`;
      startLockoutTimer();
    }

    try {
      const session = await getSession();
      if (session.unlocked) {
        await goto("/dashboard", { replace: true });
        return;
      }
    } catch {
      // Ignored: vault is locked
    }
    passphraseInput?.focus();
  });

  onDestroy(() => {
    if (lockoutTimer !== null) clearTimeout(lockoutTimer);
  });

  function describeError(cause: unknown): string {
    return isInvalidMasterKeyError(cause)
      ? "Incorrect passphrase. Please try again."
      : String(cause);
  }

  function checkCapsLock(event: KeyboardEvent) {
    capsLock = event.getModifierState("CapsLock");
  }

  function toggleVisibility() {
    showPassphrase = !showPassphrase;
    passphraseInput?.focus();
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy || isLockedOut) return;
    if (passphrase.length === 0) {
      error = "Enter your master passphrase.";
      passphraseInput?.focus();
      return;
    }

    error = "";
    busy = true;
    try {
      await unlock(passphrase);
      passphrase = "";
      sessionStorage.removeItem(STORAGE_ATTEMPTS_KEY);
      sessionStorage.removeItem(STORAGE_DELAY_KEY);
      await goto("/dashboard", { replace: true });
    } catch (cause) {
      unlockAttempts++;
      sessionStorage.setItem(STORAGE_ATTEMPTS_KEY, String(unlockAttempts));
      sessionStorage.setItem(STORAGE_DELAY_KEY, String(requiredTimeBeforeUnlock));

      if (unlockAttempts >= THROTTLE_THRESHOLD) {
        error = `Too many failed attempts. Please try again after ${requiredTimeBeforeUnlock / 1000} seconds.`;
        startLockoutTimer();
      } else {
        error = describeError(cause);
        passphraseInput?.focus();
      }
    } finally {
      busy = false;
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
            <h2>Unlock your vault</h2>
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
                aria-label={showPassphrase
                  ? "Hide passphrase"
                  : "Show passphrase"}
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
              helperText="If this is your first time here, this will be your master passphrase and it can't be recovered."
              type={showPassphrase ? "text" : "password"}
              placeholder="Enter your master passphrase"
              disabled={busy}
              error={error || undefined}
              onkeydown={checkCapsLock}
              onkeyup={checkCapsLock}
              onblur={() => {
                capsLock = false;
              }}
            />

            {#if capsLock}
              <p class="caps-warning" role="status">
                <TriangleAlertIcon size={14} />
                Caps Lock is on
              </p>
            {/if}
          </div>

          <Button type="submit" size="md" class="login-submit" loading={busy}>
            {busy ? "Unlocking…" : "Unlock vault"}
          </Button>
        </form>
      </Card>
    </div>
  </main>

  <footer class="footer">
    <LockIcon size={14} />
    Encrypted locally with SQLCipher. Nothing leaves this device.
  </footer>
</div>

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

  .form-head h2 {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 650;
    letter-spacing: -0.015em;
    text-align: center;
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

  .footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    padding: 0 1.5rem 1.25rem;
    color: var(--muted-text);
    font-size: 0.72rem;
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
