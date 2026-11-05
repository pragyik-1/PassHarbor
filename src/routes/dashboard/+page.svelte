<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { ToastManager, toast, Button } from "@hermitk/bluenite";
  import { ArrowLeftIcon } from "lucide-svelte";
  import {
    queryEntries,
    getEntry,
    addEntry,
    updateEntry,
    deleteEntry,
    setPassword,
    lock,
    listCustomKinds,
    isVaultLockedError,
  } from "#lib/api.ts";
  import type {
    EntrySummary,
    EntryDetail,
    NewEntry,
    UpdateEntry,
    KindCount,
  } from "#lib/api.ts";
  import { DEFAULT_KINDS, mergeKinds } from "#lib/kinds.ts";
  import { recallVault, forgetVault } from "#lib/session.ts";
  import type { SortKey } from "#lib/components/dashboard/EntryList.svelte";

  import AppShell from "#lib/components/dashboard/AppShell.svelte";
  import EntryList from "#lib/components/dashboard/EntryList.svelte";
  import EntryDetailPanel from "#lib/components/dashboard/EntryDetail.svelte";
  import EntryFormModal from "#lib/components/dashboard/EntryFormModal.svelte";
  import PasswordModal from "#lib/components/dashboard/PasswordModal.svelte";
  import ConfirmDialog from "#lib/components/dashboard/ConfirmDialog.svelte";

  let vaultName = $state<string | null>(null);

  let entries = $state<EntrySummary[]>([]);
  let kindCounts = $state<KindCount[]>([]);
  let customKinds = $state<string[]>([]);
  let totalCount = $derived(kindCounts.reduce((acc, k) => acc + k.count, 0));
  let listLoading = $state(true);

  let search = $state("");
  let kindFilter = $state<string | null>(null);
  let sortKey = $state<SortKey>("name");

  function setKindFilter(k: string | null) {
    kindFilter = k;
  }

  // All available kinds: built-in + vault custom kinds + kinds used by entries
  const allKinds = $derived(
    mergeKinds(
      DEFAULT_KINDS,
      customKinds,
      kindCounts.map((k) => k.kind),
    ),
  );

  let selectedId = $state<number | null>(null);
  let detail = $state<EntryDetail | null>(null);
  let detailLoading = $state(false);
  /** Monotonically increasing counter; guards against stale getEntry results. */
  let detailNonce = $state(0);

  let lockBusy = $state(false);
  let lockError = $state("");

  let formOpen = $state(false);
  let formMode = $state<"create" | "edit">("create");
  let formBusy = $state(false);
  let formError = $state("");

  let pwOpen = $state(false);
  let pwBusy = $state(false);
  let pwError = $state("");

  let confirmOpen = $state(false);
  let confirmBusy = $state(false);

  const selectedEntry = $derived(
    entries.find((e) => e.id === selectedId) ??
      (detail?.entry.id === selectedId ? detail.entry : null),
  );

  function handleVaultLocked() {
    forgetVault();
    toast.show({
      message: "Vault locked",
      description: "Your session has ended. Please unlock again.",
      variant: "warn",
    });
    void goto("/", { replace: true });
  }

  /** Guards against a stale search/filter response overwriting a newer one. */
  let listNonce = 0;

  async function refreshList(): Promise<void> {
    const nonce = ++listNonce;
    try {
      const res = await queryEntries({
        search,
        kind: kindFilter,
        sort: sortKey,
      });
      if (nonce !== listNonce) return;
      entries = res.entries;
      kindCounts = res.kinds;
    } catch (err) {
      if (nonce !== listNonce) return;
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      toast.show({ message: "Failed to refresh entries", variant: "danger" });
    }
  }

  async function loadCustomKinds(): Promise<void> {
    try {
      customKinds = await listCustomKinds();
    } catch (err) {
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      toast.show({ message: "Failed to load custom kinds", variant: "danger" });
    }
  }

  $effect(() => {
    // Re-query whenever search, kindFilter, or sortKey changes (when unlocked)
    if (vaultName) {
      void refreshList();
    }
  });

  async function loadDetail(id: number): Promise<void> {
    const nonce = ++detailNonce;
    detailLoading = true;
    detail = null;

    try {
      const result = await getEntry(id);
      if (nonce !== detailNonce) return; // stale — another selection happened
      detail = result;
      if (!result) {
        // Entry no longer exists
        selectedId = null;
        toast.show({ message: "Entry not found", variant: "warn" });
        await refreshList();
      }
    } catch (err) {
      if (nonce !== detailNonce) return;
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      toast.show({
        message: "Failed to load entry details",
        variant: "danger",
      });
    } finally {
      if (nonce === detailNonce) {
        detailLoading = false;
      }
    }
  }

  // ── Selection ──────────────────────────────────────────────

  function selectEntry(id: number | null) {
    if (id === null) {
      selectedId = null;
      detail = null;
      return;
    }
    if (selectedId === id) return;
    selectedId = id;
    detail = null;
    void loadDetail(id);
  }

  // ── Lock vault ─────────────────────────────────────────────

  async function lockVault() {
    if (lockBusy) return;
    lockBusy = true;
    lockError = "";
    try {
      await lock();
      forgetVault();
      await goto("/", { replace: true });
    } catch (err) {
      lockError = String(err);
    } finally {
      lockBusy = false;
    }
  }

  // ── Create entry ───────────────────────────────────────────

  function openCreate() {
    formMode = "create";
    formError = "";
    formOpen = true;
  }

  async function handleCreate(data: NewEntry) {
    formBusy = true;
    formError = "";
    try {
      const newId = await addEntry(data);
      formOpen = false;
      await refreshList();
      selectedId = newId;
      void loadDetail(newId);
      toast.show({ message: "Entry added", variant: "success" });
    } catch (err) {
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      formError = String(err);
    } finally {
      formBusy = false;
    }
  }

  // ── Edit entry ─────────────────────────────────────────────

  function openEdit() {
    formMode = "edit";
    formError = "";
    formOpen = true;
  }

  async function handleEdit(data: UpdateEntry) {
    if (!selectedId) return;
    formBusy = true;
    formError = "";
    try {
      const ok = await updateEntry(selectedId, data);
      formOpen = false;
      if (!ok) {
        toast.show({ message: "Entry not found", variant: "warn" });
        selectedId = null;
        detail = null;
      } else {
        toast.show({ message: "Entry updated", variant: "success" });
      }
      await refreshList();
      if (selectedId) void loadDetail(selectedId);
    } catch (err) {
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      formError = String(err);
    } finally {
      formBusy = false;
    }
  }

  // ── Delete entry ───────────────────────────────────────────

  function openDelete() {
    confirmOpen = true;
  }

  async function handleDelete() {
    if (!selectedId) return;
    confirmBusy = true;
    try {
      const ok = await deleteEntry(selectedId);
      confirmOpen = false;
      selectedId = null;
      detail = null;
      if (!ok) {
        toast.show({ message: "Entry not found", variant: "warn" });
      } else {
        toast.show({ message: "Entry deleted", variant: "success" });
      }
      await refreshList();
    } catch (err) {
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      toast.show({ message: String(err), variant: "danger" });
      confirmOpen = false;
    } finally {
      confirmBusy = false;
    }
  }

  // ── Change / add password ──────────────────────────────────

  function openChangePassword() {
    pwError = "";
    pwOpen = true;
  }

  async function handleSetPassword(password: string) {
    if (!selectedId) return;
    pwBusy = true;
    pwError = "";
    try {
      await setPassword(selectedId, password);
      pwOpen = false;
      toast.show({ message: "Password updated", variant: "success" });
      // Refresh the detail to show new password and updated history
      void loadDetail(selectedId);
      // Note: setPassword does NOT change entry.updated_at — do not refresh list
    } catch (err) {
      if (isVaultLockedError(err)) {
        handleVaultLocked();
        return;
      }
      pwError = String(err);
    } finally {
      pwBusy = false;
    }
  }

  async function handleCopy(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.show({
        message: `${label} copied`,
        variant: "success",
        duration: 2000,
      });
    } catch {
      toast.show({
        message: "Copy failed",
        description: "Your browser may have blocked clipboard access.",
        variant: "danger",
      });
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const inInput =
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target.isContentEditable;

    // Escape → deselect entry
    if (
      e.key === "Escape" &&
      selectedId !== null &&
      !formOpen &&
      !pwOpen &&
      !confirmOpen
    ) {
      selectEntry(null);
      return;
    }

    // Ctrl/Cmd+K or / → focus search
    if (
      (e.key === "/" || ((e.ctrlKey || e.metaKey) && e.key === "k")) &&
      !inInput
    ) {
      e.preventDefault();
      document.getElementById("entry-search")?.focus();
      return;
    }

    // Ctrl/Cmd+N → new entry
    if ((e.ctrlKey || e.metaKey) && e.key === "n" && !formOpen) {
      e.preventDefault();
      openCreate();
      return;
    }
  }

  const confirmMessage = $derived(
    selectedEntry
      ? `"${selectedEntry.name}" and all its password history will be permanently deleted. This cannot be undone.`
      : "",
  );

  onMount(() => {
    const name = recallVault();
    if (!name) {
      void goto("/", { replace: true });
      return;
    }
    vaultName = name;

    void (async () => {
      listLoading = true;
      try {
        await Promise.all([refreshList(), loadCustomKinds()]);
      } catch (err) {
        if (isVaultLockedError(err)) {
          handleVaultLocked();
          return;
        }
        toast.show({ message: "Failed to load entries", variant: "danger" });
      } finally {
        listLoading = false;
      }
    })();
  });
</script>

<svelte:head>
  <title>Vault · PassHarbor</title>
</svelte:head>

<svelte:document onkeydown={handleGlobalKeydown} />

{#if vaultName}
  <AppShell
    {vaultName}
    {totalCount}
    {kindCounts}
    {kindFilter}
    onfilterkind={setKindFilter}
    {lockBusy}
    {lockError}
    onlock={lockVault}
    onnew={openCreate}
  >
    <div class="workspace">
      <!-- Entry list -->
      <div class="list-col" class:has-selection={selectedId !== null}>
        <EntryList
          {entries}
          kinds={kindCounts}
          {totalCount}
          {selectedId}
          loading={listLoading}
          {kindFilter}
          bind:search
          bind:sortKey
          onfilterkind={setKindFilter}
          onselect={selectEntry}
          onnew={openCreate}
        />
      </div>

      <!-- Detail panel -->
      <div class="detail-col" class:has-selection={selectedId !== null}>
        <!-- Mobile back bar for small screens -->
        <div class="mobile-nav-bar">
          <Button
            variant="ghost"
            color="secondary"
            size="sm"
            onclick={() => selectEntry(null)}
          >
            {#snippet icon()}
              <ArrowLeftIcon size={16} />
            {/snippet}
            Back to list
          </Button>
        </div>

        <EntryDetailPanel
          entry={selectedEntry}
          {detail}
          loading={detailLoading}
          onedit={openEdit}
          ondelete={openDelete}
          onchangepassword={openChangePassword}
          onaddpassword={openChangePassword}
          oncopy={handleCopy}
        />
      </div>
    </div>
  </AppShell>

  <!-- Modals -->
  <EntryFormModal
    bind:open={formOpen}
    mode={formMode}
    entry={formMode === "edit" ? (detail?.entry ?? null) : null}
    availableKinds={allKinds}
    busy={formBusy}
    error={formError}
    oncreate={handleCreate}
    onedit={handleEdit}
    oncancel={() => {
      formOpen = false;
    }}
  />

  <PasswordModal
    bind:open={pwOpen}
    entryName={selectedEntry?.name ?? ""}
    busy={pwBusy}
    error={pwError}
    onsubmit={handleSetPassword}
    oncancel={() => {
      pwOpen = false;
    }}
  />

  <ConfirmDialog
    bind:open={confirmOpen}
    title="Delete entry"
    message={confirmMessage}
    confirmLabel="Delete entry"
    busy={confirmBusy}
    onconfirm={handleDelete}
    oncancel={() => {
      confirmOpen = false;
    }}
  />
{/if}

<style>
  .workspace {
    display: grid;
    /* List column gets min 320px up to 1fr; Detail column scales up to 720px in fullscreen (~2x of 350px) */
    grid-template-columns: minmax(320px, 1fr) minmax(460px, 900px);
    height: 100%;
    overflow: hidden;
  }

  .list-col {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .detail-col {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-left: 1px solid var(--border);
    min-width: 0;
  }

  .mobile-nav-bar {
    display: none;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  /* Medium windows (760px to 1099px): icon rail in sidebar */
  @media (max-width: 1099px) and (min-width: 760px) {
    .workspace {
      grid-template-columns: minmax(280px, 1fr) minmax(380px, 1.25fr);
    }
  }

  /* Small windows (<760px): single-pane master-detail */
  @media (max-width: 759px) {
    .workspace {
      grid-template-columns: 1fr;
      height: 100%;
    }

    .mobile-nav-bar {
      display: flex;
      align-items: center;
    }

    /* When an item is selected, hide list and show detail */
    .list-col.has-selection {
      display: none;
    }

    .detail-col {
      display: none;
      border-left: none;
    }

    .detail-col.has-selection {
      display: flex;
      flex: 1;
      height: 100%;
    }
  }
</style>
