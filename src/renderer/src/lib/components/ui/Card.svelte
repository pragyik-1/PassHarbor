<script lang="ts">
  import '../stylesheets/Card.css'
  type CardProps = {
    style?: string
    class?: string
    children?: any
    role?: string
    ariaLabel?: string
    ariaDescribedBy?: string
    onclick?: () => void
    fullWidth?: boolean
    bordered?: boolean
  }

  let {
    style = '',
    class: _class = '',
    children,
    role = 'region',
    ariaLabel = '',
    ariaDescribedBy = '',
    onclick = () => {},
    fullWidth = false,
    bordered = false
  }: CardProps = $props()
</script>

<div
  {onclick}
  {style}
  class={`card-container ${!bordered && 'no-border'} ${fullWidth && 'full-width'} ${_class}`}
  {role}
  aria-label={ariaLabel}
  aria-describedby={ariaDescribedBy}
>
  {@render children?.()}
</div>
