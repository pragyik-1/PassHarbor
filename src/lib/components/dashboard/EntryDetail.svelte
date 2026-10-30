<script lang="ts">
  import { onDestroy } from "svelte";
  import { Button } from "@hermitk/bluenite";
  import {
    UserIcon,
    MailIcon,
    GlobeIcon,
    FileTextIcon,
    PencilIcon,
    TrashIcon,
    KeyRoundIcon,
    ClockIcon,
    PlusIcon,
  } from "lucide-svelte";
  import type { Entry, EntryDetail } from "#lib/api.ts";
  import { timeAgo, formatDateTime } from "#lib/time.ts";
  import MetaRow from "./MetaRow.svelte";
  import SecretField from "./SecretField.svelte";

  interface Props {
    entry: Entry | null;
    detail: EntryDetail | null;
    loading: boolean;
    onedit: () => void;
    ondelete: () => void;
    onchangepassword: () => void;
    onaddpassword: () => void;
    oncopy: (text: string, label: string) => void;
  }

  let {
    entry,
    detail,
    loading,
    onedit,
    ondelete,
    onchangepassword,
    onaddpassword,
    oncopy,
  }: Props = $props();

  let revealedCurrent = $state(false);
  let revealedHistory = $state<Set<number>>(new Set());
  let copiedLabel = $state<string | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | null = null;
  let previousDetail: EntryDetail | null = null;

  function resetSecrets() {
    revealedCurrent = false;
    revealedHistory = new Set();
    copiedLabel = null;
    if (copiedTimer) {
      clearTimeout(copiedTimer);
      copiedTimer = null;
    }
  }

  $effect(() => {
    if (entry === null || detail !== previousDetail) {
      previousDetail = detail;
      resetSecrets();
    }
  });

  onDestroy(() => {
    if (copiedTimer) clearTimeout(copiedTimer);
  });

  function markCopied(label: string) {
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedLabel = label;
    copiedTimer = setTimeout(() => {
      copiedLabel = null;
      copiedTimer = null;
    }, 1500);
  }

  function copy(text: string, label: string) {
    oncopy(text, label);
    markCopied(label);
  }

  function toggleHistoryReveal(id: number) {
    const next = new Set(revealedHistory);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    revealedHistory = next;
  }

  const sortedHistory = $derived(
    detail ? [...detail.history].reverse() : [],
  );
</script>

{#if !entry}
  <div class="empty-detail" aria-live="polite">
    <div class="empty-icon" aria-hidden="true">
      <KeyRoundIcon size={28} />
    </div>
    <p class="empty-title">No entry selected</p>
    <p class="empty-sub">Select an entry from the list to view its details.</p>
  </div>
{:else if loading}
  <div class="loading-detail" aria-live="polite" aria-busy="true">
    <div class="skeleton-header"></div>
    <div class="skeleton-row"></div>
    <div class="skeleton-row short"></div>
    <div class="skeleton-row"></div>
  </div>
{:else if detail}
  <article class="detail" aria-label={`Details for ${entry.name}`}>
    <!-- Header -->
    <header class="detail-header">
      <div class="header-left">
        <span class="kind-badge">{detail.entry.kind}</span>
        <h2 class="entry-name">{detail.entry.name}</h2>
        <p class="updated-at">
          <ClockIcon size={12} aria-hidden="true" />
          Updated {timeAgo(detail.entry.updated_at)}
        </p>
      </div>
      <div class="header-actions">
        <Button
          variant="ghost"
          color="secondary"
          size="sm"
          class="icon-btn"
          aria-label="Edit entry"
          title="Edit entry"
          onclick={onedit}
        >
          {#snippet icon()}
            <PencilIcon size={15} />
          {/snippet}
        </Button>
        <Button
          variant="ghost"
          color="danger"
          size="sm"
          class="icon-btn"
          aria-label="Delete entry"
          title="Delete entry"
          onclick={ondelete}
        >
          {#snippet icon()}
            <TrashIcon size={15} />
          {/snippet}
        </Button>
      </div>
    </header>

    <!-- Metadata fields -->
    <section class="meta-section" aria-label="Entry details">
      <MetaRow
        label="Username"
        value={detail.entry.username}
        copied={copiedLabel === "Username"}
        oncopy={() => copy(detail.entry.username ?? "", "Username")}
      >
        {#snippet icon()}
          <UserIcon size={14} />
        {/snippet}
      </MetaRow>

      <MetaRow
        label="Email"
        value={detail.entry.email}
        copied={copiedLabel === "Email"}
        oncopy={() => copy(detail.entry.email ?? "", "Email")}
      >
        {#snippet icon()}
          <MailIcon size={14} />
        {/snippet}
      </MetaRow>

      <MetaRow
        label="URL"
        value={detail.entry.url}
        mono
        copied={copiedLabel === "URL"}
        oncopy={() => copy(detail.entry.url ?? "", "URL")}
      >
        {#snippet icon()}
          <GlobeIcon size={14} />
        {/snippet}
      </MetaRow>

      {#if detail.entry.note}
        <MetaRow label="Note" value={detail.entry.note} multiline>
          {#snippet icon()}
            <FileTextIcon size={14} />
          {/snippet}
        </MetaRow>
      {/if}
    </section>

    <hr class="divider" />

    <!-- Password section -->
    <section class="pw-section" aria-label="Password">
      <div class="section-title">
        <KeyRoundIcon size={14} aria-hidden="true" />
        Password
      </div>

      {#if detail.current_password !== null}
        <div class="pw-row">
          <SecretField
            value={detail.current_password}
            revealed={revealedCurrent}
            copied={copiedLabel === "Password"}
            label="Password"
            onreveal={() => (revealedCurrent = !revealedCurrent)}
            oncopy={() => copy(detail.current_password ?? "", "Password")}
          />
        </div>

        <div class="pw-footer-actions">
          <Button
            variant="ghost"
            color="secondary"
            size="sm"
            onclick={onchangepassword}
          >
            Change password
          </Button>
        </div>
      {:else}
        <div class="no-pw">
          <p class="no-pw-text">No password set for this entry.</p>
          <Button size="sm" onclick={onaddpassword}>
            {#snippet icon()}
              <PlusIcon size={15} />
            {/snippet}
            Add password
          </Button>
        </div>
      {/if}
    </section>

    <!-- Password history -->
    {#if sortedHistory.length > 0}
      <section class="history-section" aria-label="Password history">
        <div class="section-title">
          <ClockIcon size={14} aria-hidden="true" />
          History
          <span class="history-count" aria-label="{sortedHistory.length} versions">
            {sortedHistory.length}
          </span>
        </div>

        <ol class="history-list" reversed>
          {#each sortedHistory as pw, i (pw.id)}
            {@const isCurrent = i === 0}
            <li class="history-item">
              <div class="history-meta">
                <span class="history-date">{formatDateTime(pw.created_at)}</span>
                {#if isCurrent}
                  <span class="current-badge">Current</span>
                {/if}
              </div>
              <div class="history-pw-row">
                <SecretField
                  compact
                  value={pw.password}
                  revealed={revealedHistory.has(pw.id)}
                  copied={copiedLabel === `History ${pw.id}`}
                  label="This password version"
                  onreveal={() => toggleHistoryReveal(pw.id)}
                  oncopy={() => copy(pw.password, `History ${pw.id}`)}
                />
              </div>
            </li>
          {/each}
        </ol>
      </section>
    {/if}

    <!-- Timestamps footer -->
    <footer class="timestamps">
      <span>Created {formatDateTime(detail.entry.created_at)}</span>
      <span>·</span>
      <span>Updated {formatDateTime(detail.entry.updated_at)}</span>
    </footer>
  </article>
{/if}

<style>
  /* ── Empty / loading ─────────────────────────────────────── */

  .empty-detail {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 0.5rem;
    padding: 2rem;
    text-align: center;
    color: var(--muted-text);
  }

  .empty-icon {
    display: grid;
    place-items: center;
    width: 3rem;
    height: 3rem;
    border-radius: var(--round-lg);
    background: color-mix(in srgb, var(--primary) 8%, transparent);
    color: var(--primary);
    margin-bottom: 0.5rem;
  }

  .empty-title {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--secondary-text);
  }

  .empty-sub {
    margin: 0;
    font-size: 0.8125rem;
  }

  .loading-detail {
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
  }

  .skeleton-header,
  .skeleton-row {
    background: var(--surface-hover);
    border-radius: var(--round-sm);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .skeleton-header {
    height: 1.25rem;
    width: 60%;
  }

  .skeleton-row {
    height: 1rem;
    width: 100%;
  }

  .skeleton-row.short {
    width: 45%;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.45; }
  }

  @media (prefers-reduced-motion: reduce) {
    .skeleton-header,
    .skeleton-row {
      animation: none;
    }
  }

  /* ── Detail article ──────────────────────────────────────── */

  .detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    scrollbar-width: thin;
  }

  /* ── Header ──────────────────────────────────────────────── */

  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 1.25rem 1.25rem 0;
    flex-shrink: 0;
  }

  .header-left {
    min-width: 0;
    flex: 1;
  }

  .kind-badge {
    display: inline-block;
    padding: 0.15rem 0.5rem;
    border-radius: var(--round-sm);
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: color-mix(in srgb, var(--primary) 12%, transparent);
    color: var(--primary);
    margin-bottom: 0.35rem;
  }

  .entry-name {
    margin: 0 0 0.3rem;
    font-size: 1rem;
    font-weight: 650;
    letter-spacing: -0.015em;
    color: var(--text);
    word-break: break-word;
    line-height: 1.3;
  }

  .updated-at {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    margin: 0;
    font-size: 0.75rem;
    color: var(--muted-text);
  }

  .header-actions {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  :global(.icon-btn) {
    width: 2rem !important;
    height: 2rem !important;
    padding: 0 !important;
  }

  /* ── Meta section ────────────────────────────────────────── */

  .meta-section {
    padding: 1rem 1.25rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  /* ── Divider ─────────────────────────────────────────────── */

  .divider {
    border: none;
    border-top: 1px solid var(--border);
    margin: 1rem 1.25rem 0;
  }

  /* ── Password section ────────────────────────────────────── */

  .pw-section {
    padding: 1rem 1.25rem 0;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-text);
    margin-bottom: 0.625rem;
  }

  .history-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.3rem;
    border-radius: 999px;
    background: var(--surface-hover);
    color: var(--secondary-text);
    font-size: 0.7rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .pw-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--surface-hover);
    border-radius: var(--round-md);
    padding: 0.5rem 0.625rem;
  }

  .pw-footer-actions {
    margin-top: 0.5rem;
    display: flex;
    gap: 0.5rem;
  }

  .no-pw {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem 0.875rem;
    border: 1px dashed var(--border);
    border-radius: var(--round-md);
    background: color-mix(in srgb, var(--surface-hover) 50%, transparent);
  }

  .no-pw-text {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--secondary-text);
  }

  /* ── History section ─────────────────────────────────────── */

  .history-section {
    padding: 1rem 1.25rem 0;
  }

  .history-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .history-item {
    border: 1px solid var(--border);
    border-radius: var(--round-md);
    overflow: hidden;
  }

  .history-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.625rem;
    background: var(--surface-hover);
    border-bottom: 1px solid var(--border);
  }

  .history-date {
    font-size: 0.75rem;
    color: var(--secondary-text);
    font-variant-numeric: tabular-nums;
  }

  .current-badge {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: color-mix(in srgb, var(--success) 15%, transparent);
    color: var(--success);
  }

  .history-pw-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.5rem 0.4rem 0.625rem;
  }

  /* ── Timestamps footer ───────────────────────────────────── */

  .timestamps {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem 1.25rem 1.25rem;
    margin-top: auto;
    font-size: 0.72rem;
    color: var(--muted-text);
    font-variant-numeric: tabular-nums;
    flex-wrap: wrap;
  }
</style>
