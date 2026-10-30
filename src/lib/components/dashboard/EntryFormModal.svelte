<script lang="ts">
  import { Modal, Button, Input, Textarea, Select } from "@hermitk/bluenite";
  import type { SelectOption } from "@hermitk/bluenite";
  import { EyeIcon, EyeOffIcon } from "lucide-svelte";
  import type { Entry, NewEntry, UpdateEntry } from "#lib/api.ts";

  type Mode = "create" | "edit";

  interface Props {
    open: boolean;
    mode: Mode;
    /** Existing entry to prefill when mode === "edit". */
    entry?: Entry | null;
    /** All available kinds (vault-derived + user-defined). */
    availableKinds?: string[];
    busy?: boolean;
    error?: string;
    oncreate: (data: NewEntry) => void;
    onedit: (data: UpdateEntry) => void;
    oncancel: () => void;
  }

  let {
    open = $bindable(),
    mode,
    entry = null,
    availableKinds = [],
    busy = false,
    error = "",
    oncreate,
    onedit,
    oncancel,
  }: Props = $props();

  const MAX_KIND = 64;
  const MAX_NAME = 512;

  import { getAllKinds } from "#lib/kinds.ts";

  let kind = $state("login");

  const kindOptions = $derived<SelectOption[]>(
    getAllKinds([...availableKinds, ...(kind ? [kind] : [])]).map((k) => ({
      label: k,
      value: k,
    })),
  );
  let name = $state("");
  let username = $state("");
  let email = $state("");
  let url = $state("");
  let note = $state("");
  let password = $state("");
  let showPassword = $state(false);
  let localError = $state("");

  $effect(() => {
    if (!open) {
      localError = "";
      showPassword = false;
      password = "";
      return;
    }
    if (mode === "edit" && entry) {
      kind = entry.kind;
      name = entry.name;
      username = entry.username ?? "";
      email = entry.email ?? "";
      url = entry.url ?? "";
      note = entry.note ?? "";
    } else {
      kind = "login";
      name = "";
      username = "";
      email = "";
      url = "";
      note = "";
    }
  });

  function nullIfBlank(v: string): string | null {
    const trimmed = v.trim();
    return trimmed === "" ? null : trimmed;
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    localError = "";

    const trimmedKind = kind.trim();
    const trimmedName = name.trim();

    if (!trimmedKind) {
      localError = "Kind is required.";
      return;
    }
    if (trimmedKind.length > MAX_KIND) {
      localError = `Kind must be at most ${MAX_KIND} characters.`;
      return;
    }
    if (!trimmedName) {
      localError = "Name is required.";
      return;
    }
    if (trimmedName.length > MAX_NAME) {
      localError = `Name must be at most ${MAX_NAME} characters.`;
      return;
    }

    if (mode === "create") {
      oncreate({
        kind: trimmedKind,
        name: trimmedName,
        username: nullIfBlank(username),
        email: nullIfBlank(email),
        url: nullIfBlank(url),
        note: nullIfBlank(note),
        password: nullIfBlank(password),
      });
    } else {
      onedit({
        kind: trimmedKind,
        name: trimmedName,
        username: nullIfBlank(username),
        email: nullIfBlank(email),
        url: nullIfBlank(url),
        note: nullIfBlank(note),
      });
    }
  }

  const title = $derived(mode === "create" ? "New entry" : "Edit entry");
  const displayError = $derived(localError || error);
</script>

<Modal
  bind:open
  size="md"
  {title}
  onClose={oncancel}
  closeOnBackdropClick={!busy}
>
  <form id="entry-form" onsubmit={handleSubmit} novalidate>
    <div class="row-2">
      <div class="field">
        <label for="entry-kind"
          >Kind <span class="req" aria-hidden="true">*</span></label
        >
        <Select
          id="entry-kind"
          bind:value={kind}
          options={kindOptions}
          placeholder="Select a kind…"
          disabled={busy}
        />
      </div>

      <div class="field">
        <label for="entry-name"
          >Name <span class="req" aria-hidden="true">*</span></label
        >
        <Input
          id="entry-name"
          bind:value={name}
          placeholder="example.com"
          disabled={busy}
          maxlength={MAX_NAME}
        />
      </div>
    </div>

    <div class="row-2">
      <div class="field">
        <label for="entry-username">Username</label>
        <Input
          id="entry-username"
          bind:value={username}
          placeholder="Optional"
          autocomplete="off"
          disabled={busy}
        />
      </div>

      <div class="field">
        <label for="entry-email">Email</label>
        <Input
          id="entry-email"
          type="email"
          bind:value={email}
          placeholder="Optional"
          autocomplete="off"
          disabled={busy}
        />
      </div>
    </div>

    <div class="field">
      <label for="entry-url">URL</label>
      <Input
        id="entry-url"
        bind:value={url}
        placeholder="https://example.com"
        autocomplete="off"
        disabled={busy}
      />
    </div>

    <div class="field">
      <label for="entry-note">Note</label>
      <Textarea
        id="entry-note"
        bind:value={note}
        placeholder="Optional notes"
        rows={3}
        disabled={busy}
      />
    </div>

    {#if mode === "create"}
      <div class="field">
        <div class="field-head">
          <label for="entry-password">Initial password</label>
          <Button
            type="button"
            variant="ghost"
            color="secondary"
            size="sm"
            class="reveal-btn"
            aria-label={showPassword ? "Hide password" : "Show password"}
            aria-pressed={showPassword}
            onclick={() => (showPassword = !showPassword)}
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
          id="entry-password"
          type={showPassword ? "text" : "password"}
          bind:value={password}
          placeholder="Optional. Can be set later"
          autocomplete="new-password"
          disabled={busy}
          class="mono-input"
        />
      </div>
    {:else}
      <p class="pw-note">
        To change the password, use the "Change password" action in the detail
        panel after saving.
      </p>
    {/if}

    {#if displayError}
      <p class="error" role="alert">{displayError}</p>
    {/if}
  </form>

  {#snippet footer()}
    <div class="actions">
      <Button
        variant="ghost"
        color="secondary"
        size="sm"
        disabled={busy}
        onclick={oncancel}
      >
        Cancel
      </Button>
      <Button type="submit" form="entry-form" size="sm" loading={busy}>
        {mode === "create" ? "Add entry" : "Save changes"}
      </Button>
    </div>
  {/snippet}
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
  }

  .row-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  @media (max-width: 520px) {
    .row-2 {
      grid-template-columns: 1fr;
    }
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
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

  .req {
    color: var(--danger);
    margin-left: 1px;
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

  .pw-note {
    margin: 0;
    font-size: 0.8rem;
    color: var(--muted-text);
    font-style: italic;
  }

  .error {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--danger);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
