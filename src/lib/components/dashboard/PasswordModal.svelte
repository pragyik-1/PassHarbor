<script lang="ts">
  import { Modal, Button, Input } from "@hermitk/bluenite";
  import { EyeIcon, EyeOffIcon } from "lucide-svelte";
  import DialogActions from "./DialogActions.svelte";

  interface Props {
    open: boolean;
    entryName: string;
    busy?: boolean;
    error?: string;
    onsubmit: (password: string) => void;
    oncancel: () => void;
  }

  let {
    open = $bindable(),
    entryName,
    busy = false,
    error = "",
    onsubmit,
    oncancel,
  }: Props = $props();

  let password = $state("");
  let confirm = $state("");
  let showPassword = $state(false);
  let localError = $state("");

  $effect(() => {
    if (!open) {
      password = "";
      confirm = "";
      showPassword = false;
      localError = "";
    }
  });

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    localError = "";

    const trimmed = password;
    if (!trimmed) {
      localError = "Password must not be empty.";
      return;
    }
    if (trimmed !== confirm) {
      localError = "Passwords do not match.";
      return;
    }
    onsubmit(trimmed);
  }

  function toggleVisibility() {
    showPassword = !showPassword;
  }

  const displayError = $derived(localError || error);
</script>

<Modal
  bind:open
  size="sm"
  title="Change password"
  onClose={oncancel}
  closeOnBackdropClick={!busy}
>
  <p class="subtitle">
    Setting a new password for <strong>{entryName}</strong>. The previous
    password will remain in history.
  </p>

  <form id="pw-form" onsubmit={handleSubmit}>
    <div class="field">
      <div class="field-head">
        <label for="new-password">New password</label>
        <Button
          type="button"
          variant="ghost"
          color="secondary"
          size="sm"
          class="reveal-btn"
          aria-label={showPassword ? "Hide password" : "Show password"}
          aria-pressed={showPassword}
          onclick={toggleVisibility}
        >
          {#snippet icon()}
            {#if showPassword}
              <EyeOffIcon size={16} />
            {:else}
              <EyeIcon size={16} />
            {/if}
          {/snippet}
        </Button>
      </div>
      <Input
        id="new-password"
        type={showPassword ? "text" : "password"}
        bind:value={password}
        placeholder="Enter new password"
        autocomplete="new-password"
        disabled={busy}
        class="mono-input"
      />
    </div>

    <div class="field">
      <label for="confirm-password">Confirm password</label>
      <Input
        id="confirm-password"
        type={showPassword ? "text" : "password"}
        bind:value={confirm}
        placeholder="Confirm new password"
        autocomplete="new-password"
        disabled={busy}
        class="mono-input"
      />
    </div>

    {#if displayError}
      <p class="error" role="alert">{displayError}</p>
    {/if}
  </form>

  {#snippet footer()}
    <DialogActions
      busy={busy}
      confirmLabel="Set password"
      submitForm="pw-form"
      {oncancel}
    />
  {/snippet}
</Modal>

<style>
  .subtitle {
    margin: 0 0 1.125rem;
    font-size: 0.8125rem;
    color: var(--secondary-text);
    line-height: 1.5;
  }

  .subtitle strong {
    color: var(--text);
    font-weight: 600;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .field-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 1.5rem;
  }

  label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--primary-text);
  }

  :global(.reveal-btn) {
    width: 1.75rem !important;
    height: 1.75rem !important;
    padding: 0 !important;
  }

  :global(.mono-input input) {
    font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
    font-size: 0.875rem;
    letter-spacing: 0.03em;
  }

  .error {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--danger);
  }
</style>
