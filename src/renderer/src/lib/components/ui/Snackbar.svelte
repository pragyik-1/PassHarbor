<script lang="ts">
  import { fly } from 'svelte/transition'
  import { useTimeout, onEscapeKey, onClickOutside } from '../../utils'
  import '../stylesheets/Snackbar.css'

  type SnackbarProps = {
    children?: any
    style?: string
    class?: string
    duration?: number
    position?: 'top' | 'bottom' | 'left' | 'right' | 'center'
    onClose?: () => void
    visible: boolean
    fullWidth?: boolean
  }

  let {
    children,
    class: _class = '',
    style = '',
    duration = 3000,
    onClose = () => {
      visible = false
    },
    visible = $bindable(false),
    fullWidth = false
  }: SnackbarProps = $props()

  $effect(() => {
    if (visible) {
      useTimeout(() => {
        onClose?.()
      }, duration)
    }
    onEscapeKey(() => onClose?.())
  })
</script>

{#if visible}
  <div
    use:onClickOutside={() => onClose?.()}
    in:fly={{ y: 100, duration: 500 }}
    out:fly={{ y: 100, duration: 1000 }}
    class="snackbar-wrapper {fullWidth ? 'full-width' : ''}"
  >
    <div {style} class={`snackbar ${!_class.includes('border') ? 'border-surface' : ''} ${_class}`}>
      {@render children?.()}
    </div>
  </div>
{/if}
