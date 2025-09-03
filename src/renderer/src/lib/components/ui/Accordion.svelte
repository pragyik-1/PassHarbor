<script lang="ts">
  import { slide } from 'svelte/transition'
  import { ChevronUp } from 'lucide-svelte'
  import Grid from './Grid.svelte'
  import Button from './Button.svelte'
  import '../stylesheets/Accordion.css'

  export type AccordionProps = {
    title: string
    open?: boolean
    children?: any
  }

  let { title, open = $bindable(false), children }: AccordionProps = $props()
</script>

<div class="accordion">
  <Button fullWidth variant="ghost" onclick={() => (open = !open)}>
    <Grid row justify="between" align="center" fullWidth gap={8}>
      <span>{title}</span>
      <ChevronUp class={'chevron ' + (open ? 'rotate' : '')} />
    </Grid>
  </Button>
  {#if open}
    <div transition:slide class="accordion-content">
      {@render children?.()}
    </div>
  {/if}
</div>
