<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    Button,
    DropdownMenu,
    DropdownMenuItem,
    initTheme,
    Navbar,
    theme,
    toggleTheme,
  } from "@hermitk/bluenite";
  import { isNotLoginPage } from "#lib/utils.svelte.ts";
  import { MoonIcon, SettingsIcon, SunIcon } from "lucide-svelte";

  import "@hermitk/bluenite/styles/theme.css";
  import "#lib/styles/global.css";
  import { goto } from "$app/navigation";

  let { children }: { children: Snippet } = $props();

  let settingsButtonRef = $state<HTMLButtonElement | null>(null);
  let themeDropdownOpen = $state(false);

  // Apply the stored theme before the app renders to avoid a color flash.
  initTheme();
</script>

{#if isNotLoginPage()}
  <Navbar height="64px">
    <div>
      <a href="/dashboard">
        <p class="brand">PassHarbor</p>
      </a>
    </div>
    <div>
      <Button
        onclick={() => (themeDropdownOpen = !themeDropdownOpen)}
        variant="ghost"
        icon={settingsIcon}
        bind:ref={settingsButtonRef}
      />
      <DropdownMenu bind:open={themeDropdownOpen} anchor={settingsButtonRef}>
        <DropdownMenuItem icon={themeToggleIcon} onclick={toggleTheme}>
          Toggle Themes
        </DropdownMenuItem>
        <DropdownMenuItem onclick={() => goto("/settings")}>
          More Settings
        </DropdownMenuItem>
      </DropdownMenu>
    </div>
  </Navbar>

  <!-- Spacer div: since the navbar has a height of 64px to ensure content doesn't hide behind it -->
  <div style="margin-bottom: 64px"></div>
{/if}

{@render children()}

{#snippet settingsIcon()}
  <SettingsIcon size={20} />
{/snippet}

{#snippet themeToggleIcon()}
  {#if theme.value == "dark"}
    <SunIcon size={20} />
  {:else}
    <MoonIcon size={20} />
  {/if}
{/snippet}

<style>
  .brand {
    display: inline-flex;
    align-items: center;
    gap: 0.65rem;
    color: var(--text);
    text-decoration: none;
    font-weight: 800;
  }
</style>
