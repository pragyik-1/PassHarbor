<script lang="ts">
  import Card from './Card.svelte'
  import Grid from './Grid.svelte'
  import { onMount } from 'svelte'
  import '../stylesheets/Appbar.css'

  type AppBarProps = {
    class?: string
    style?: string
    bgColor?: string
    children?: any
    gap?: number
    position?: 'top' | 'bottom'
    fixed?: boolean
    dynamic?: boolean
    justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
    align?: 'start' | 'center' | 'end' | 'stretch' | 'baseline'
  }

  let {
    class: _class = '',
    style = '',
    bgColor = 'surface',
    gap = 4,
    position = 'top',
    fixed = false,
    children,
    dynamic = false,
    justify = 'between',
    align = 'center'
  }: AppBarProps = $props()

  let dynamicClass = $state('')
  let lastScrollY = 0

  const handleScroll = () => {
    if (dynamic) {
      const currentScrollY = window.scrollY
      const visible = currentScrollY < lastScrollY || currentScrollY < 10
      dynamicClass = visible ? 'translate-y-0' : '-translate-y-full'
      lastScrollY = currentScrollY
    }
  }

  onMount(() => {
    if (dynamic) window.addEventListener('scroll', handleScroll)
    return () => dynamic && window.removeEventListener('scroll', handleScroll)
  })
</script>

<nav
  class={`appbar ${fixed ? 'fixed' : 'static'} ${dynamicClass} ${
    position === 'bottom' ? 'bottom' : 'top'
  } ${_class}`}
>
  <Card
    style={`background-color: ${bgColor}; border: none; border-radius: 0; ${style}`}
    class="full-width"
  >
    <Grid row {gap} {justify} {align} class="appbar-grid">
      {@render children?.()}
    </Grid>
  </Card>
</nav>
