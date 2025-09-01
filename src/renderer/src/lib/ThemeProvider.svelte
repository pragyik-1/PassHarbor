<script lang="ts">
  import { theme } from './ThemeStore.svelte'

  let { children } = $props()

  $effect(() => {
    if (theme.value === 'light') {
      document.documentElement.classList.add('light')
      document.documentElement.classList.remove('dark')
    } else {
      document.documentElement.classList.remove('light')
      document.documentElement.classList.add('dark')
    }
  })
</script>

{@render children?.()}
