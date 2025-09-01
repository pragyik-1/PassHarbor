<script lang="ts">
  import '../stylesheets/Switch.css'
  export type SwitchProps = {
    checked?: boolean
    disabled?: boolean
    name?: string
    id?: string
    onLabel?: string
    offLabel?: string
    style?: string
    required?: boolean
    class?: string
    fullwidth?: boolean
    onchange?: (event: Event & { currentTarget: HTMLInputElement }) => void
    onclick?: () => void
  }

  let {
    checked = $bindable(false),
    disabled = false,
    name = '',
    id = 'switch',
    onLabel = '',
    offLabel = '',
    style = '',
    required = false,
    class: _class = '',
    fullwidth = false,
    onchange = (): void => {},
    onclick = (): void => {}
  }: SwitchProps = $props()

  let switchClasses = $state('switch-container')
  if (fullwidth) {
    switchClasses += ' switch-full-width'
  }
  if (_class) {
    switchClasses += ` ${_class}`
  }

  const handleChange = (event: Event): void => {
    if (disabled) return
    checked = (event.target as HTMLInputElement).checked
    onchange(event as Event & { currentTarget: HTMLInputElement })
  }
</script>

<div {style} class={switchClasses}>
  <label class="switch-label" aria-disabled={disabled}>
    <input
      type="checkbox"
      {checked}
      {name}
      {id}
      {required}
      {disabled}
      onchange={handleChange}
      {onclick}
      class="hidden-input"
      role="switch"
      aria-checked={checked}
    />
    <span class="slider round">
      {#if checked && onLabel}
        <span class="switch-text on-text">{onLabel}</span>
      {:else if !checked && offLabel}
        <span class="switch-text off-text">{offLabel}</span>
      {/if}
    </span>
  </label>
</div>
