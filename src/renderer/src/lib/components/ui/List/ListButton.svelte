<script lang="ts">
  import Grid from '../Grid.svelte'
  import '../../stylesheets/ListButton.css'

  type ButtonProps = {
    onclick?: (arg?: any) => void
    disabled?: boolean
    children?: any
    icon?: any
    iconSize?: number
    class?: string
    style?: string
    fullWidth?: boolean
    loading?: boolean
  }

  let {
    onclick = () => {},
    children,
    disabled = false,
    icon = null,
    iconSize = 20,
    class: _class = '',
    style = '',
    fullWidth = true,
    loading = false
  }: ButtonProps = $props()
</script>

<button {disabled} {style} {onclick} class={`list-button ${fullWidth && 'full-width'} ${_class}`}>
  {#if loading}
    <div class="button-loader"></div>
  {:else}
    <Grid gap={4} row style="flex-wrap: nowrap !important">
      {#if icon}
        {@const IconComponent = icon}
        <IconComponent size={iconSize} />
      {/if}
      {@render children?.()}
    </Grid>
  {/if}
</button>
