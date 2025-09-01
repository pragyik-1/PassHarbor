<script lang="ts">
  import { fly } from 'svelte/transition'
  import { onClickOutside } from '../../utils'
  import '../stylesheets/DropDownButton.css'
  import type { Snippet } from 'svelte'

  type DropDownMenuProps = {
    class?: string
    menuClass?: string
    menuStyle?: string
    variant?: 'filled' | 'outlined' | 'ghost'
    disabled?: boolean
    position?: 'left' | 'right' | 'center'
    animate?: boolean
    duration?: number
    fullWidth?: boolean
    children?: Snippet
    open?: boolean
    anchor?: Snippet
    wrapperStyle?: string
  }

  let {
    class: _class = '',
    menuClass = '',
    menuStyle = '',
    position = 'center',
    animate = true,
    duration = 200,
    fullWidth = false,
    open = $bindable(false),
    anchor,
    children,
    wrapperStyle
  }: DropDownMenuProps = $props()

  const positionClasses = {
    left: 'pos-left',
    right: 'pos-right',
    center: 'pos-center'
  }

  const transitionClass = animate ? 'animate-dropdown' : ''
</script>

<div
  style={wrapperStyle}
  use:onClickOutside={() => (open = false)}
  class="dropdown-container {fullWidth ? 'full' : ''}"
>
  {@render anchor()}

  {#if open}
    <div
      style={`--duration: ${duration}ms; ${menuStyle}`}
      transition:fly={{ y: -15, duration }}
      class="dropdown-menu {menuClass} {positionClasses[position]} {transitionClass}"
    >
      {@render children?.()}
    </div>
  {/if}
</div>
