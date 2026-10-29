<script lang="ts">
  import type { Snippet } from "svelte";
  import { initTheme } from "@hermitk/bluenite";
  import "@hermitk/bluenite/styles/theme.css";
  import "#lib/styles/global.css";

  let { children }: { children: Snippet } = $props();

  // Apply the stored theme before the app renders to avoid a color flash.
  initTheme();
</script>

{@render children()}
