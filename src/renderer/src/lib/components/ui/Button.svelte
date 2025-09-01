<script lang="ts">
  import Grid from './Grid.svelte'
  import '../stylesheets/Button.css'

  type ButtonProps = {
    onclick?: (...args: unknown[]) => void
    class?: string
    children?: any
    type?: 'button' | 'submit' | 'reset' | undefined
    variant?: 'filled' | 'outlined' | 'ghost' | 'link'
    disabled?: boolean
    iconSize?: number
    iconLeft?: any
    iconRight?: any
    style?: string
    size?: 'sm' | 'md' | 'lg'
    fullWidth?: boolean
    ariaLabel?: string
    tooltip?: string
    loading?: boolean
  }

  let {
    onclick = () => {},
    children,
    class: _class = '',
    variant = 'filled',
    type = 'button',
    disabled = false,
    iconSize = 20,
    style = '',
    size = 'md',
    fullWidth = false,
    ariaLabel = '',
    iconLeft = null,
    iconRight = null,
    tooltip = '',
    loading = false
  }: ButtonProps = $props()

  let icon = $state(iconLeft || iconRight)
  let iconPosition = $state(iconLeft ? 'left' : 'right')

  $effect(() => {
    icon = iconLeft || iconRight
    iconPosition = iconLeft ? 'left' : 'right'
  })
</script>

<button
  {style}
  {disabled}
  aria-label={ariaLabel}
  title={tooltip}
  {type}
  {onclick}
  class={`button ${variant} ${size} ${fullWidth ? 'full-width' : ''} ${_class}`}
>
  <Grid row gap={4} style="flex-wrap: nowrap !important">
    {#if loading}
      <div class="button-loader"></div>
    {:else if icon && iconPosition === 'left'}
      {@const IconComponent = icon}
      <IconComponent size={iconSize} />
    {/if}
    {@render children?.()}
    {#if icon && iconPosition === 'right'}
      {@const IconComponent = icon}
      <IconComponent size={iconSize} />
    {/if}
  </Grid>
</button>
