<script lang="ts">
  import { Button } from "@hermitk/bluenite";
  import {
    SearchIcon,
    XIcon,
    ShieldIcon,
    SortAscIcon,
    PlusIcon,
  } from "lucide-svelte";
  import type { Entry } from "#lib/api.ts";
  import { countKinds, kindMatches, isKindActive } from "#lib/kinds.ts";
  import { timeAgo } from "#lib/time.ts";

  type SortKey = "name" | "updated" | "created";

  interface Props {
    entries: Entry[];
    selectedId: number | null;
    loading: boolean;
    kindFilter: string | null;
    onfilterkind: (k: string | null) => void;
    onselect: (id: number) => void;
    onnew: () => void;
  }

  let { entries, selectedId, loading, kindFilter, onfilterkind, onselect, onnew }: Props = $props();

  let search = $state("");
  let sortKey = $state<SortKey>("name");

  const kinds = $derived(countKinds(entries.map((entry) => entry.kind)));

  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();

    let result = entries.filter((e) => {
      if (!kindMatches(e.kind, kindFilter)) return false;
      if (!q) return true;
      return (
        e.name.toLowerCase().includes(q) ||
        (e.username ?? "").toLowerCase().includes(q) ||
        (e.email ?? "").toLowerCase().includes(q) ||
        (e.url ?? "").toLowerCase().includes(q) ||
        (e.note ?? "").toLowerCase().includes(q)
      );
    });

    if (sortKey === "name") {
      result = result.sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { sensitivity: "base" }),
      );
    } else if (sortKey === "updated") {
      result = result.sort((a, b) => b.updated_at - a.updated_at);
    } else {
      result = result.sort((a, b) => b.created_at - a.created_at);
    }

    return result;
  });

  function clearSearch() {
    search = "";
  }

  function secondaryLine(e: Entry): string {
    return e.username ?? e.email ?? e.url ?? "";
  }

  function handleKeydown(event: KeyboardEvent, id: number) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onselect(id);
    }
  }

  const sortOptions: { key: SortKey; label: string }[] = [
    { key: "name", label: "Name A–Z" },
    { key: "updated", label: "Recently updated" },
    { key: "created", label: "Recently added" },
  ];

  const currentSortLabel = $derived(
    sortOptions.find((o) => o.key === sortKey)?.label ?? "Sort",
  );

  let sortOpen = $state(false);
  let sortAnchor = $state<HTMLElement | null>(null);

  function selectSort(key: SortKey) {
    sortKey = key;
    sortOpen = false;
  }

  function handleSortKeydown(e: KeyboardEvent, key: SortKey) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      selectSort(key);
    }
  }
</script>

<div class="list-panel">
  <!-- Search + sort bar -->
  <div class="toolbar">
    <div class="search-wrap">
      <span class="search-icon" aria-hidden="true"><SearchIcon size={14} /></span>
      <input
        id="entry-search"
        class="search-input"
        type="search"
        bind:value={search}
        placeholder="Search entries…"
        aria-label="Search entries"
        autocomplete="off"
        spellcheck="false"
      />
      {#if search}
        <button
          class="clear-btn"
          type="button"
          aria-label="Clear search"
          title="Clear search"
          onclick={clearSearch}
        >
          <XIcon size={13} />
        </button>
      {/if}
    </div>

    <div class="sort-wrap">
      <button
        bind:this={sortAnchor}
        class="sort-btn"
        type="button"
        aria-haspopup="listbox"
        aria-expanded={sortOpen}
        aria-label="Sort: {currentSortLabel}"
        title="Change sort order"
        onclick={() => (sortOpen = !sortOpen)}
      >
        <SortAscIcon size={14} aria-hidden="true" />
      </button>

      {#if sortOpen}
        <div
          class="sort-menu"
          role="listbox"
          aria-label="Sort options"
          tabindex="-1"
        >
          {#each sortOptions as opt (opt.key)}
            <button
              class="sort-option"
              class:active={sortKey === opt.key}
              role="option"
              aria-selected={sortKey === opt.key}
              type="button"
              onclick={() => selectSort(opt.key)}
              onkeydown={(e) => handleSortKeydown(e, opt.key)}
            >
              {opt.label}
            </button>
          {/each}
        </div>

        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="sort-backdrop"
          onclick={() => (sortOpen = false)}
        ></div>
      {/if}
    </div>
  </div>

  <!-- Kind filter chips -->
  {#if kinds.length > 0}
    <div class="kind-chips" role="group" aria-label="Filter by kind">
      <button
        class="kind-chip"
        class:active={kindFilter === null}
        type="button"
        onclick={() => onfilterkind(null)}
        aria-pressed={kindFilter === null}
      >
        All
        <span class="chip-count">{entries.length}</span>
      </button>
      {#each kinds as { kind, count } (kind)}
        {@const active = isKindActive(kind, kindFilter)}
        <button
          class="kind-chip"
          class:active={active}
          type="button"
          onclick={() => onfilterkind(active ? null : kind)}
          aria-pressed={active}
        >
          {kind}
          <span class="chip-count">{count}</span>
        </button>
      {/each}
    </div>
  {/if}

  <!-- Entry list or states -->
  <div class="list-scroll" role="listbox" aria-label="Entries">
    {#if loading}
      {#each { length: 5 } as _, i (i)}
        <div class="skeleton-row" aria-hidden="true">
          <div class="sk-badge"></div>
          <div class="sk-body">
            <div class="sk-name"></div>
            <div class="sk-sub"></div>
          </div>
          <div class="sk-time"></div>
        </div>
      {/each}
    {:else if entries.length === 0}
      <div class="empty-state">
        <div class="empty-icon-wrap" aria-hidden="true">
          <ShieldIcon size={26} />
        </div>
        <p class="empty-title">Your vault is empty</p>
        <p class="empty-sub">
          Add your first entry to start storing credentials securely.
        </p>
        <Button size="sm" onclick={onnew}>
          {#snippet icon()}
            <PlusIcon size={15} />
          {/snippet}
          Add your first entry
        </Button>
      </div>
    {:else if filtered.length === 0}
      <div class="empty-state">
        <p class="empty-title">No entries match your search</p>
        <button class="link-btn" type="button" onclick={clearSearch}>
          Clear search
        </button>
      </div>
    {:else}
      {#each filtered as entry (entry.id)}
        {@const secondary = secondaryLine(entry)}
        <button
          class="entry-row"
          class:selected={selectedId === entry.id}
          type="button"
          role="option"
          aria-selected={selectedId === entry.id}
          onclick={() => onselect(entry.id)}
          onkeydown={(e) => handleKeydown(e, entry.id)}
        >
          <span class="row-badge" aria-hidden="true">
            {entry.kind.slice(0, 2).toUpperCase()}
          </span>
          <span class="row-body">
            <span class="row-name">{entry.name}</span>
            {#if secondary}
              <span class="row-sub">{secondary}</span>
            {/if}
          </span>
          <span class="row-time">{timeAgo(entry.updated_at)}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .list-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    border-right: 1px solid var(--border);
  }

  /* ── Toolbar ──────────────────────────────────────────────── */

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .search-wrap {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 0.5rem;
    color: var(--muted-text);
    pointer-events: none;
    display: flex;
    align-items: center;
  }

  .search-input {
    width: 100%;
    height: 2rem;
    padding: 0 2rem 0 1.75rem;
    background: var(--input);
    border: 1px solid var(--border);
    border-radius: var(--round-md);
    color: var(--text);
    font-size: 0.8125rem;
    outline: none;
    transition: border-color 120ms ease;
    appearance: none;
    -webkit-appearance: none;
  }

  .search-input::-webkit-search-cancel-button {
    display: none;
  }

  .search-input:focus {
    border-color: var(--primary);
  }

  .clear-btn {
    position: absolute;
    right: 0.35rem;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    background: none;
    border: none;
    border-radius: var(--round-sm);
    color: var(--muted-text);
    cursor: pointer;
    padding: 0;
    transition: color 120ms ease;
  }

  .clear-btn:hover {
    color: var(--text);
  }

  /* ── Sort dropdown ────────────────────────────────────────── */

  .sort-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .sort-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    background: var(--input);
    border: 1px solid var(--border);
    border-radius: var(--round-md);
    color: var(--secondary-text);
    cursor: pointer;
    padding: 0;
    transition: border-color 120ms ease, color 120ms ease;
  }

  .sort-btn:hover,
  .sort-btn[aria-expanded="true"] {
    border-color: var(--primary);
    color: var(--primary);
  }

  .sort-menu {
    position: absolute;
    top: calc(100% + 0.35rem);
    right: 0;
    z-index: 100;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--round-md);
    padding: 0.25rem;
    min-width: 11rem;
    box-shadow: 0 4px 16px -4px color-mix(in srgb, var(--text) 15%, transparent);
  }

  .sort-option {
    display: block;
    width: 100%;
    padding: 0.4rem 0.65rem;
    background: none;
    border: none;
    border-radius: var(--round-sm);
    text-align: left;
    font-size: 0.8125rem;
    color: var(--text);
    cursor: pointer;
    transition: background 100ms ease;
  }

  .sort-option:hover,
  .sort-option.active {
    background: var(--surface-hover);
  }

  .sort-option.active {
    color: var(--primary);
    font-weight: 600;
  }

  .sort-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  /* ── Kind chips ───────────────────────────────────────────── */

  .kind-chips {
    display: flex;
    gap: 0.375rem;
    flex-wrap: wrap;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .kind-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.2rem 0.55rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: none;
    font-size: 0.75rem;
    color: var(--secondary-text);
    cursor: pointer;
    transition: border-color 120ms ease, color 120ms ease, background 120ms ease;
    white-space: nowrap;
  }

  .kind-chip:hover {
    border-color: var(--primary);
    color: var(--primary);
  }

  .kind-chip.active {
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 10%, transparent);
    color: var(--primary);
    font-weight: 600;
  }

  .chip-count {
    font-variant-numeric: tabular-nums;
    font-size: 0.7rem;
    opacity: 0.75;
  }

  /* ── Scrollable list ──────────────────────────────────────── */

  .list-scroll {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    padding: 0.375rem 0;
  }

  /* ── Entry row ────────────────────────────────────────────── */

  .entry-row {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: none;
    border: none;
    border-radius: 0;
    cursor: pointer;
    text-align: left;
    transition: background 100ms ease;
    min-height: 3rem;
  }

  .entry-row:hover {
    background: var(--surface-hover);
  }

  .entry-row:focus-visible {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
  }

  .entry-row.selected {
    background: color-mix(in srgb, var(--primary) 8%, transparent);
    border-left: 2px solid var(--primary);
    padding-left: calc(0.75rem - 2px);
  }

  .row-badge {
    flex-shrink: 0;
    display: grid;
    place-items: center;
    width: 2rem;
    height: 2rem;
    border-radius: var(--round-md);
    background: color-mix(in srgb, var(--primary) 12%, transparent);
    color: var(--primary);
    font-size: 0.65rem;
    font-weight: 800;
    letter-spacing: 0.02em;
  }

  .row-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .row-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .row-sub {
    font-size: 0.75rem;
    color: var(--secondary-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .row-time {
    flex-shrink: 0;
    font-size: 0.7rem;
    color: var(--muted-text);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  /* ── Skeleton rows ────────────────────────────────────────── */

  .skeleton-row {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.5rem 0.75rem;
    min-height: 3rem;
  }

  .sk-badge {
    flex-shrink: 0;
    width: 2rem;
    height: 2rem;
    border-radius: var(--round-md);
    background: var(--surface-hover);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .sk-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .sk-name {
    height: 0.875rem;
    width: 60%;
    border-radius: var(--round-sm);
    background: var(--surface-hover);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .sk-sub {
    height: 0.75rem;
    width: 40%;
    border-radius: var(--round-sm);
    background: var(--surface-hover);
    animation: pulse 1.4s 0.1s ease-in-out infinite;
  }

  .sk-time {
    width: 3rem;
    height: 0.75rem;
    border-radius: var(--round-sm);
    background: var(--surface-hover);
    animation: pulse 1.4s 0.2s ease-in-out infinite;
    flex-shrink: 0;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.45; }
  }

  @media (prefers-reduced-motion: reduce) {
    .sk-badge,
    .sk-name,
    .sk-sub,
    .sk-time {
      animation: none;
    }
  }

  /* ── Empty / no-results states ────────────────────────────── */

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2.5rem 1.5rem;
    text-align: center;
  }

  .empty-icon-wrap {
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
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--secondary-text);
  }

  .empty-sub {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--muted-text);
    max-width: 22ch;
    line-height: 1.5;
  }

  .link-btn {
    background: none;
    border: none;
    padding: 0;
    font-size: 0.8125rem;
    color: var(--primary);
    cursor: pointer;
    text-decoration: underline;
  }
</style>
