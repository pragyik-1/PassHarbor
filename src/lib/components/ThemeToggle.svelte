<script lang="ts">
  import { Button, theme, toggleTheme } from "@hermitk/bluenite";
  import { MoonIcon, SunIcon } from "lucide-svelte";

  interface Props {
    iconSize?: number;
  }

  let { iconSize = 18 }: Props = $props();

  const label = $derived(
    theme.value === "dark" ? "Switch to light theme" : "Switch to dark theme",
  );
</script>

<Button
  type="button"
  variant="ghost"
  color="secondary"
  size="sm"
  class="theme-toggle"
  aria-label={label}
  title={label}
  onclick={toggleTheme}
>
  {#snippet icon()}
    {#if theme.value === "dark"}
      <SunIcon size={iconSize} />
    {:else}
      <MoonIcon size={iconSize} />
    {/if}
  {/snippet}
</Button>
