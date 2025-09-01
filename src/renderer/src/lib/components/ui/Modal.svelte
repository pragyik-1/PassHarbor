<script lang="ts">
  import Card from './Card.svelte'
  import '../stylesheets/Modal.css'

  type ModalProps = {
    isOpen?: boolean
    onClose?: () => void
    style?: string
    class?: string
    children?: any
    role?: string
    tabIndex?: number
    ariaLabel?: string
    ariaDescribedBy?: string
    onclick?: () => void
  }

  let {
    isOpen = $bindable(false),
    onClose,
    style = '',
    class: _class = '',
    children,
    role = 'dialog',
    ariaLabel = 'Modal dialog',
    ariaDescribedBy = '',
    onclick,
    ...restProps
  }: ModalProps = $props()
</script>

{#if isOpen}
  <div class="modal-backdrop" onclick={onClose} role="presentation" tabIndex={-1}>
    <Card
      {style}
      class={`card-container-modal ${_class}`}
      {role}
      {ariaLabel}
      {ariaDescribedBy}
      {onclick}
      {...restProps}
    >
      {@render children?.()}
    </Card>
  </div>
{/if}
