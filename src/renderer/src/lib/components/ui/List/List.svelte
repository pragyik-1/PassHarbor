<script lang="ts">
  import Grid from '../Grid.svelte'

  type ListProps = {
    children?: any
    class?: string
    style?: string
    ariaLabel?: string
  }

  let {
    children,
    class: _class = '',
    style = '',
    ariaLabel = 'List of items'
  }: ListProps = $props()
</script>

<Grid col class={`list-container ${_class}`} {style} role="list" {ariaLabel}>
  {@render children?.()}
</Grid>

<style>
  :global(.list-container) {
    overflow-y: auto;
    border-radius: 0.375rem;
  }
</style>
