<script lang="ts">
  import type { Snippet } from "svelte";
  import { initTheme } from "@hermitk/bluenite";
  import "@hermitk/bluenite/styles/theme.css";

  let { children }: { children: Snippet } = $props();

  // Apply the stored theme before the app renders to avoid a color flash.
  initTheme();
</script>

<div>
  {@render children()}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    min-height: 100%;
  }

  :global(body) {
    min-height: 100vh;
    min-height: 100dvh;
  }
</style>
