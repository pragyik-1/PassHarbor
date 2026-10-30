<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "@hermitk/bluenite";
  import {
    ShieldIcon,
    LockIcon,
    PlusIcon,
    ListIcon,
    TagIcon,
    SettingsIcon,
  } from "lucide-svelte";
  import type { Entry } from "#lib/api.ts";
  import { countKinds, isKindActive } from "#lib/kinds.ts";
  import BrandMark from "#lib/components/BrandMark.svelte";
  import ThemeToggle from "#lib/components/ThemeToggle.svelte";

  interface Props {
    vaultName: string;
    entries: Entry[];
    kindFilter: string | null;
    onfilterkind: (k: string | null) => void;
    lockBusy: boolean;
    lockError: string;
    onlock: () => void;
    onnew: () => void;
    children: Snippet;
  }

  let {
    vaultName,
    entries,
    kindFilter,
    onfilterkind,
    lockBusy,
    lockError,
    onlock,
    onnew,
    children,
  }: Props = $props();

  const kindCounts = $derived(countKinds(entries.map((entry) => entry.kind)));
</script>

<div class="shell">
  <aside class="sidebar" aria-label="Sidebar">
    <!-- Brand -->
    <div class="brand">
      <BrandMark size="sm" />
      <span class="brand-name">PassHarbor</span>
    </div>

    <!-- Vault name -->
    <div class="vault-label" title={vaultName}>
      <span class="vault-tag">Vault</span>
      <span class="vault-name">{vaultName}</span>
    </div>

    <!-- New entry -->
    <div class="sidebar-action">
      <Button size="sm" class="new-btn" onclick={onnew}>
        {#snippet icon()}
          <PlusIcon size={15} />
        {/snippet}
        New entry
      </Button>
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav" aria-label="Vault navigation">
      <div class="nav-section-label">Library</div>

      <!-- All items — clears kind filter -->
      <button
        type="button"
        class="nav-item"
        class:active={kindFilter === null}
        aria-label="All items — {entries.length} entries"
        onclick={() => onfilterkind(null)}
      >
        <ListIcon size={15} aria-hidden="true" />
        <span class="nav-label">All items</span>
        <span class="nav-count" aria-hidden="true">{entries.length}</span>
      </button>

      {#if kindCounts.length > 0}
        <div class="nav-section-label kinds-label">Kinds</div>
        {#each kindCounts as { kind, count } (kind)}
          {@const active = isKindActive(kind, kindFilter)}
          <button
            class="nav-item kind-item"
            class:active
            type="button"
            aria-label="{kind} — {count}"
            aria-pressed={active}
            onclick={() => onfilterkind(active ? null : kind)}
          >
            <TagIcon size={14} aria-hidden="true" />
            <span class="nav-label">{kind}</span>
            <span class="nav-count" aria-hidden="true">{count}</span>
          </button>
        {/each}
      {/if}
    </nav>

    <!-- Footer -->
    <footer class="sidebar-footer">
      {#if lockError}
        <p class="lock-error" role="alert">{lockError}</p>
      {/if}

      <!-- Settings link -->
      <a
        href="/settings"
        class="settings-link"
        aria-label="Settings"
        title="Settings"
      >
        <SettingsIcon size={15} aria-hidden="true" />
        <span class="settings-label">Settings</span>
      </a>

      <div class="footer-actions">
        <ThemeToggle iconSize={16} />

        <Button
          variant="outline"
          color="danger"
          size="sm"
          class="lock-btn"
          loading={lockBusy}
          aria-label="Lock vault"
          title="Lock vault"
          onclick={onlock}
        >
          {#snippet icon()}
            <LockIcon size={15} />
          {/snippet}
          <span class="lock-label">Lock vault</span>
        </Button>
      </div>

      <p class="encryption-note">
        <ShieldIcon size={10} aria-hidden="true" />
        Encrypted locally with SQLCipher. Nothing leaves this device.
      </p>
    </footer>
  </aside>

  <!-- Main area -->
  <main class="main" id="main-content" tabindex="-1">
    {@render children()}
  </main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: 220px 1fr;
    height: 100dvh;
    overflow: hidden;
    background: var(--bg);
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    overflow: hidden;
    background: var(--surface);
    border-right: 1px solid var(--border);
    flex-shrink: 0;
  }

  /* Brand */
  .brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 1rem 1rem 0.7rem;
    flex-shrink: 0;
  }

  .brand-name {
    font-size: 0.9rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text);
  }

  /* Vault name */
  .vault-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0 0.625rem;
    padding: 0.3rem 0.575rem;
    border-radius: var(--round-md);
    background: color-mix(in srgb, var(--primary) 7%, transparent);
    border: 1px solid color-mix(in srgb, var(--primary) 15%, transparent);
    overflow: hidden;
  }

  .vault-tag {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--primary);
    flex-shrink: 0;
  }

  .vault-name {
    font-size: 0.775rem;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* New entry */
  .sidebar-action {
    padding: 0.625rem 0.625rem 0.2rem;
    flex-shrink: 0;
  }

  :global(.new-btn) {
    width: 100% !important;
    justify-content: flex-start !important;
  }

  /* Nav */
  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    padding: 0.2rem 0.5rem 0.5rem;
  }

  .nav-section-label {
    padding: 0.55rem 0.5rem 0.2rem;
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted-text);
  }

  .kinds-label {
    margin-top: 0.25rem;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.375rem 0.5rem;
    border-radius: var(--round-md);
    border: none;
    background: none;
    color: var(--secondary-text);
    font-size: 0.8rem;
    text-align: left;
    cursor: pointer;
    transition:
      background 110ms ease,
      color 110ms ease;
  }

  .nav-item:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .nav-item:focus-visible {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
  }

  .nav-item.active {
    background: color-mix(in srgb, var(--primary) 10%, transparent);
    color: var(--primary);
    font-weight: 600;
  }

  .kind-item {
    cursor: pointer;
  }

  .nav-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-count {
    font-size: 0.7rem;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--muted-text);
    flex-shrink: 0;
  }

  /* Footer */
  .sidebar-footer {
    flex-shrink: 0;
    padding: 0.5rem 0.625rem 0.75rem;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .lock-error {
    margin: 0;
    font-size: 0.73rem;
    color: var(--danger);
    padding: 0.3rem 0.45rem;
    background: color-mix(in srgb, var(--danger) 8%, transparent);
    border-radius: var(--round-sm);
  }

  /* Settings link */
  .settings-link {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.5rem;
    border-radius: var(--round-md);
    color: var(--secondary-text);
    font-size: 0.8rem;
    text-decoration: none;
    transition:
      background 110ms ease,
      color 110ms ease;
  }

  .settings-link:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .settings-link:focus-visible {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
  }

  .settings-label {
    flex: 1;
  }

  .footer-actions {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  :global(.theme-toggle) {
    width: 2rem !important;
    height: 2rem !important;
    padding: 0 !important;
    flex-shrink: 0;
    border-radius: 999px !important;
  }

  :global(.lock-btn) {
    flex: 1;
    justify-content: flex-start !important;
    font-size: 0.8rem !important;
  }

  .lock-label {
    font-size: 0.8rem;
  }

  .encryption-note {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin: 0;
    font-size: 0.65rem;
    color: var(--muted-text);
    line-height: 1.4;
  }

  .main {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    overflow: hidden;
    min-width: 0;
    outline: none;
  }

  /* Icon rail at 760–1099px */
  @media (max-width: 1099px) and (min-width: 760px) {
    .shell {
      grid-template-columns: 64px 1fr;
    }

    .brand-name,
    .vault-label,
    .sidebar-action,
    .nav-section-label,
    .nav-count,
    .nav-label,
    .lock-label,
    .settings-label,
    .encryption-note,
    .lock-error {
      display: none;
    }

    .brand {
      justify-content: center;
      padding: 0.875rem 0.5rem 0.625rem;
    }

    .sidebar-nav {
      padding: 0.25rem 0.375rem;
    }

    .nav-item {
      justify-content: center;
      padding: 0.45rem;
    }

    .sidebar-footer {
      align-items: center;
      padding: 0.5rem 0.375rem;
      gap: 0.25rem;
    }

    .settings-link {
      justify-content: center;
      padding: 0.45rem;
    }

    .footer-actions {
      flex-direction: column;
    }

    :global(.lock-btn) {
      flex: unset !important;
      width: 2rem !important;
      height: 2rem !important;
      padding: 0 !important;
      justify-content: center !important;
    }
  }

  /* Single pane below 760px */
  @media (max-width: 759px) {
    .shell {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }

    .sidebar {
      height: auto;
      flex-direction: row;
      align-items: center;
      border-right: none;
      border-bottom: 1px solid var(--border);
      padding: 0.5rem 0.75rem;
      gap: 0.5rem;
      flex-wrap: wrap;
    }

    .brand {
      padding: 0;
      flex-shrink: 0;
    }

    .vault-label {
      margin: 0;
      flex-shrink: 1;
    }

    .sidebar-action {
      padding: 0;
      flex-shrink: 0;
    }

    :global(.new-btn) {
      width: auto !important;
    }

    .sidebar-nav,
    .nav-section-label,
    .kinds-label,
    .encryption-note {
      display: none;
    }

    .sidebar-footer {
      margin-left: auto;
      padding: 0;
      border-top: none;
      flex-direction: row;
      gap: 0.25rem;
    }

    .settings-link {
      padding: 0.35rem;
    }

    .settings-label,
    .lock-label {
      display: none;
    }

    :global(.lock-btn) {
      flex: unset !important;
      width: 2rem !important;
      height: 2rem !important;
      padding: 0 !important;
      justify-content: center !important;
    }

    .lock-error {
      display: none;
    }

    .main {
      height: 100%;
      flex: 1;
      min-height: 0;
      overflow: hidden;
    }
  }
</style>
