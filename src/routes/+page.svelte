<script lang="ts">
  import { unlock } from "$lib/api";

  let passphrase = $state("");
  let error = $state("");

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    try {
      await unlock(passphrase);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<main class="container">
  <h1>PassHarbor</h1>

  <form class="column" onsubmit={submit}>
    <input
      type="password"
      placeholder="Master passphrase..."
      bind:value={passphrase}
    />
    <button type="submit">Unlock</button>
  </form>

  {#if error}
    <p class="error">{error}</p>
  {/if}
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;
  }

  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .column {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
    width: 16em;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }

  .error {
    color: #d32f2f;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    input,
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }

    .error {
      color: #ff6b6b;
    }
  }
</style>
