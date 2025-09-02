<script lang="ts">
  import Router from 'svelte-spa-router'
  import { ThemeProvider } from '../lib'
  import Home from './Home/Home.svelte'
  import Settings from './Settings/Settings.svelte'
  import MasterPasswordModal from '../lib/components/MasterPasswordModal/MasterPasswordModal.svelte'
  import { onMount } from 'svelte'
  import TopBar from '../lib/components/TopBar/TopBar.svelte'
  let routes = $state({
    '/': Home,
    '/home': Home,
    '/settings': Settings,
    '*': Home
  })
  let masterPasswordModalOpen = $state(false)

  onMount(async () => {
    if (
      (await window.api.masterPassword.exists()) &&
      sessionStorage.getItem('haslogged') !== 'true'
    ) {
      masterPasswordModalOpen = true
    } else {
      masterPasswordModalOpen = false
    }
  })
</script>

<ThemeProvider>
  <Router {routes} />
  <TopBar />
  <MasterPasswordModal bind:open={masterPasswordModalOpen} />
</ThemeProvider>
