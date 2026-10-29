<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { Button, Card } from "@hermitk/bluenite";
  import { lock } from "#lib/api.ts";
  import { forgetVault, recallVault } from "#lib/session.ts";

  let busy = $state(false);
  let error = $state("");
  let vaultName = $state<string | null>(null);

  onMount(() => {
    const name = recallVault();
    if (!name) {
      void goto("/", { replace: true });
      return;
    }
    vaultName = name;
  });

  async function lockVault() {
    if (busy) return;
    busy = true;
    error = "";
    try {
      await lock();
      forgetVault();
      await goto("/", { replace: true });
    } catch (cause) {
      error = String(cause);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:head>
  <title>Vault · PassHarbor</title>
</svelte:head>

{#if vaultName}
  <main class="page">
    <Card
      class="dashboard-card"
      title="Vault unlocked"
      subtitle={`${vaultName} is open. This is a placeholder for the vault workspace.`}
    >
      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}

      <div class="actions">
        <Button
          variant="outline"
          color="danger"
          loading={busy}
          onclick={lockVault}
        >
          Lock vault
        </Button>
      </div>
    </Card>
  </main>
{/if}

<style>
  .page {
    display: grid;
    place-items: center;
    min-height: 100vh;
    min-height: 100dvh;
    padding: 2rem 1.5rem;
  }

  .page :global(.dashboard-card) {
    width: min(100%, 30rem);
    border-radius: var(--round-xl);
  }

  .error {
    margin: 0 0 1rem;
    color: var(--danger);
    font-size: 0.875rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
