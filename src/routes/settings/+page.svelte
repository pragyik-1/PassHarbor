<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import {
        Button,
        Input,
        Card,
        ToastManager,
        toast,
        theme,
        toggleTheme,
    } from "@hermitk/bluenite";
    import {
        ArrowLeftIcon,
        PlusIcon,
        TrashIcon,
        ShieldIcon,
        LockIcon,
        SunIcon,
        MoonIcon,
        TagIcon,
        SparklesIcon,
        RefreshCwIcon,
    } from "lucide-svelte";
    import {
        DEFAULT_KINDS,
        addCustomKind,
        countKinds,
        getCustomKinds,
        mergeCustomKinds,
        removeCustomKind,
        resetCustomKinds,
    } from "#lib/kinds.ts";
    import { recallVault, forgetVault } from "#lib/session.ts";
    import { lock, listEntries, isVaultLockedError, type Entry } from "#lib/api.ts";

    let vaultName = $state<string | null>(null);
    let entries = $state<Entry[]>([]);
    let savedCustomKinds = $state<string[]>(getCustomKinds());
    let newKindInput = $state("");
    let inputError = $state("");
    let lockBusy = $state(false);

    const customKinds = $derived(
        mergeCustomKinds(savedCustomKinds, entries.map((entry) => entry.kind)),
    );
    const kindCounts = $derived(
        countKinds(entries.map((entry) => entry.kind)),
    );
    const usageByKind = $derived(
        new Map(kindCounts.map(({ kind, count }) => [kind.toLowerCase(), count])),
    );

    onMount(async () => {
        const name = recallVault();
        if (!name) {
            void goto("/");
            return;
        }
        vaultName = name;
        try {
            entries = await listEntries();
        } catch (err) {
            if (isVaultLockedError(err)) {
                forgetVault();
                void goto("/", { replace: true });
                return;
            }
        }
    });

    function kindUsage(kindName: string): number {
        return usageByKind.get(kindName.toLowerCase()) ?? 0;
    }

    function handleAddKind(e?: SubmitEvent) {
        if (e) e.preventDefault();
        inputError = "";
        const trimmed = newKindInput.trim();
        const res = addCustomKind(trimmed);
        if (!res.success) {
            inputError = res.error ?? "Failed to add kind";
            return;
        }
        toast.show({
            message: `Added "${trimmed}" kind`,
            variant: "success",
        });
        newKindInput = "";
        savedCustomKinds = getCustomKinds();
    }

    function handleRemoveKind(k: string) {
        const usageCount = kindUsage(k);
        if (usageCount > 0) {
            const entryText =
                usageCount === 1 ? "1 entry is" : `${usageCount} entries are`;
            toast.show({
                message: `Cannot remove "${k}"`,
                description: `${entryText} currently using this kind. Change or delete those entries first to remove it.`,
                variant: "danger",
            });
            return;
        }

        removeCustomKind(k);
        savedCustomKinds = getCustomKinds();
        toast.show({ message: `Removed "${k}" kind`, variant: "info" });
    }

    function handleResetKinds() {
        const inUse = customKinds.filter((k) => kindUsage(k) > 0);
        if (inUse.length > 0) {
            toast.show({
                message: "Cannot reset kinds",
                description: `The following custom kinds are in use by vault entries: ${inUse.join(", ")}. Change or delete those entries first.`,
                variant: "danger",
            });
            return;
        }

        resetCustomKinds();
        savedCustomKinds = getCustomKinds();
        toast.show({ message: "Reset to default kinds", variant: "info" });
    }

    async function handleLock() {
        if (lockBusy) return;
        lockBusy = true;
        try {
            await lock();
            forgetVault();
            await goto("/");
        } catch (err) {
            toast.show({
                message: `Failed to lock vault: ${err instanceof Error ? err.message : String(err)}`,
                variant: "danger",
            });
        } finally {
            lockBusy = false;
        }
    }
</script>

<svelte:head>
    <title>Settings — PassHarbor</title>
</svelte:head>

<ToastManager />

<div class="settings-page">
    <div class="settings-container">
        <!-- Top Header -->
        <header class="settings-header">
            <Button
                variant="ghost"
                color="secondary"
                size="sm"
                class="back-btn"
                onclick={() => goto("/dashboard")}
            >
                {#snippet icon()}
                    <ArrowLeftIcon size={16} />
                {/snippet}
                Back to Vault
            </Button>

            <div class="title-group">
                <h1 class="page-title">Settings</h1>
                <p class="page-desc">
                    Manage vault preferences, custom kinds, and appearance
                </p>
            </div>
        </header>

        <div class="sections">
            <!-- Section: Kinds Definition -->
            <section class="section">
                <div class="section-header">
                    <div class="section-icon">
                        <TagIcon size={18} />
                    </div>
                    <div>
                        <h2 class="section-title">Entry Kinds</h2>
                        <p class="section-desc">
                            Custom kinds allow you to categorize passwords and
                            credentials. They appear in the kind selector when
                            creating entries.
                        </p>
                    </div>
                </div>

                <Card class="settings-card">
                    <!-- Add Kind Form -->
                    <form class="add-kind-form" onsubmit={handleAddKind}>
                        <div class="input-wrapper">
                            <Input
                                bind:value={newKindInput}
                                placeholder="New kind name (e.g. Server, API Key, Banking)…"
                                error={inputError}
                                maxlength={64}
                            />
                        </div>
                        <Button
                            size="md"
                            type="submit"
                            disabled={!newKindInput.trim()}
                        >
                            {#snippet icon()}
                                <PlusIcon size={16} />
                            {/snippet}
                            Add Kind
                        </Button>
                    </form>

                    <!-- Kinds List -->
                    <div class="kinds-display">
                        <div class="kinds-group">
                            <span class="group-label">Built-in Kinds</span>
                            <div class="tags-cloud">
                                {#each DEFAULT_KINDS as kind (kind)}
                                    {@const usage = kindUsage(kind)}
                                    <div class="kind-tag builtin">
                                        <span class="tag-name">{kind}</span>
                                        <span class="tag-badge">built-in</span>
                                        {#if usage > 0}
                                            <span
                                                class="usage-badge builtin-usage"
                                                title="{usage} {usage === 1 ? 'entry' : 'entries'} in use"
                                            >
                                                {usage} in use
                                            </span>
                                        {/if}
                                    </div>
                                {/each}
                            </div>
                        </div>

                        <div class="kinds-group">
                            <div class="custom-header">
                                <span class="group-label">
                                    Custom Kinds ({customKinds.length})
                                </span>
                                {#if customKinds.length > 0}
                                    <Button
                                        variant="ghost"
                                        color="secondary"
                                        size="sm"
                                        class="reset-btn"
                                        onclick={handleResetKinds}
                                    >
                                        {#snippet icon()}
                                            <RefreshCwIcon size={13} />
                                        {/snippet}
                                        Reset
                                    </Button>
                                {/if}
                            </div>

                            {#if customKinds.length === 0}
                                <p class="no-custom-text">
                                    No custom kinds added yet. Create one above
                                    to classify your vault items.
                                </p>
                            {:else}
                                <div class="tags-cloud">
                                    {#each customKinds as kind (kind)}
                                        {@const usage = kindUsage(kind)}
                                        <div
                                            class="kind-tag custom"
                                            class:in-use={usage > 0}
                                        >
                                            <span class="tag-name">{kind}</span>
                                            {#if usage > 0}
                                                <span
                                                    class="usage-badge"
                                                    title="{usage} {usage === 1 ? 'entry' : 'entries'} in use"
                                                >
                                                    {usage} in use
                                                </span>
                                            {/if}
                                            <button
                                                type="button"
                                                class="delete-tag-btn"
                                                class:blocked={usage > 0}
                                                aria-label={usage > 0
                                                    ? `Cannot remove ${kind} (${usage} in use)`
                                                    : `Remove ${kind} kind`}
                                                title={usage > 0
                                                    ? `In use by ${usage} ${usage === 1 ? 'entry' : 'entries'} — change or delete them first to remove`
                                                    : `Remove ${kind}`}
                                                onclick={() =>
                                                    handleRemoveKind(kind)}
                                            >
                                                <TrashIcon size={12} />
                                            </button>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    </div>
                </Card>
            </section>

            <!-- Section: Appearance -->
            <section class="section">
                <div class="section-header">
                    <div class="section-icon">
                        <SparklesIcon size={18} />
                    </div>
                    <div>
                        <h2 class="section-title">Appearance</h2>
                        <p class="section-desc">
                            Customize the interface visual theme
                        </p>
                    </div>
                </div>

                <Card class="settings-card">
                    <div class="setting-row">
                        <div>
                            <div class="setting-label">Theme</div>
                            <div class="setting-sub">
                                Currently using {theme.value === "dark"
                                    ? "Dark mode"
                                    : "Light mode"}
                            </div>
                        </div>
                        <Button
                            variant="outline"
                            color="secondary"
                            size="sm"
                            onclick={toggleTheme}
                        >
                            {#snippet icon()}
                                {#if theme.value === "dark"}
                                    <SunIcon size={16} />
                                {:else}
                                    <MoonIcon size={16} />
                                {/if}
                            {/snippet}
                            Switch to {theme.value === "dark"
                                ? "Light"
                                : "Dark"}
                        </Button>
                    </div>
                </Card>
            </section>

            <!-- Section: Vault & Security -->
            <section class="section">
                <div class="section-header">
                    <div class="section-icon">
                        <ShieldIcon size={18} />
                    </div>
                    <div>
                        <h2 class="section-title">Vault & Security</h2>
                        <p class="section-desc">
                            Session state and database encryption details
                        </p>
                    </div>
                </div>

                <Card class="settings-card">
                    <div class="setting-row">
                        <div>
                            <div class="setting-label">Active Vault</div>
                            <div class="setting-sub mono-val">
                                {vaultName ?? "Unknown"}
                            </div>
                        </div>
                        <Button
                            variant="outline"
                            color="danger"
                            size="sm"
                            loading={lockBusy}
                            onclick={handleLock}
                        >
                            {#snippet icon()}
                                <LockIcon size={15} />
                            {/snippet}
                            Lock Vault
                        </Button>
                    </div>
                </Card>
            </section>
        </div>
    </div>
</div>

<style>
    .settings-page {
        min-height: 100dvh;
        background: var(--bg);
        color: var(--text);
        padding: 2rem 1.5rem 4rem;
        overflow-y: auto;
    }

    .settings-container {
        max-width: 680px;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    /* Header */
    .settings-header {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        align-items: flex-start;
    }

    .title-group {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .page-title {
        margin: 0;
        font-size: 1.625rem;
        font-weight: 700;
        letter-spacing: -0.02em;
        color: var(--text);
    }

    .page-desc {
        margin: 0;
        font-size: 0.875rem;
        color: var(--secondary-text);
    }

    /* Sections */
    .sections {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .section {
        display: flex;
        flex-direction: column;
        gap: 0.875rem;
    }

    .section-header {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
    }

    .section-icon {
        display: grid;
        place-items: center;
        width: 2rem;
        height: 2rem;
        border-radius: var(--round-md);
        background: color-mix(in srgb, var(--primary) 12%, transparent);
        color: var(--primary);
        flex-shrink: 0;
        margin-top: 0.1rem;
    }

    .section-title {
        margin: 0 0 0.2rem;
        font-size: 1.05rem;
        font-weight: 650;
        color: var(--text);
    }

    .section-desc {
        margin: 0;
        font-size: 0.8125rem;
        color: var(--muted-text);
        line-height: 1.4;
    }

    :global(.settings-card) {
        padding: 1.25rem 1.5rem !important;
        background: var(--surface) !important;
        border: 1px solid var(--border) !important;
        border-radius: var(--round-lg) !important;
    }

    /* Add Kind Form */
    .add-kind-form {
        display: flex;
        gap: 0.75rem;
        align-items: flex-start;
        margin-bottom: 1.5rem;
    }

    .input-wrapper {
        flex: 1;
        min-width: 0;
    }

    /* Kinds Display */
    .kinds-display {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .kinds-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .custom-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .group-label {
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--muted-text);
    }

    .tags-cloud {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .kind-tag {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        padding: 0.3rem 0.65rem;
        border-radius: var(--round-md);
        font-size: 0.8125rem;
        font-weight: 500;
    }

    .kind-tag.builtin {
        background: var(--surface-hover);
        color: var(--secondary-text);
        border: 1px solid var(--border);
    }

    .kind-tag.custom {
        background: color-mix(in srgb, var(--primary) 10%, transparent);
        color: var(--text);
        border: 1px solid color-mix(in srgb, var(--primary) 25%, transparent);
    }

    .tag-name {
        font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
        font-size: 0.8rem;
    }

    .tag-badge {
        font-size: 0.65rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        padding: 0.05rem 0.3rem;
        border-radius: var(--round-sm);
        background: color-mix(in srgb, var(--text) 8%, transparent);
        color: var(--muted-text);
    }

    .usage-badge {
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.02em;
        padding: 0.05rem 0.35rem;
        border-radius: var(--round-sm);
        background: color-mix(in srgb, var(--primary) 20%, transparent);
        color: var(--primary);
    }

    .usage-badge.builtin-usage {
        background: color-mix(in srgb, var(--text) 10%, transparent);
        color: var(--secondary-text);
    }

    .delete-tag-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        padding: 0.15rem;
        margin: -0.15rem;
        border-radius: var(--round-sm);
        color: var(--muted-text);
        cursor: pointer;
        transition:
            color 120ms ease,
            background 120ms ease;
    }

    .delete-tag-btn:hover {
        color: var(--danger);
        background: color-mix(in srgb, var(--danger) 12%, transparent);
    }

    .delete-tag-btn.blocked {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .no-custom-text {
        margin: 0;
        font-size: 0.8125rem;
        color: var(--muted-text);
        font-style: italic;
    }

    /* Setting Rows */
    .setting-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
    }

    .setting-label {
        font-size: 0.9375rem;
        font-weight: 600;
        color: var(--text);
    }

    .setting-sub {
        font-size: 0.8125rem;
        color: var(--secondary-text);
        margin-top: 0.15rem;
    }

    .mono-val {
        font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
        font-size: 0.85rem;
    }
</style>
