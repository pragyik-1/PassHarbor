<script lang="ts">
  import '../stylesheets/Textarea.css'
  type TextareaProps = {
    placeholder?: string
    value?: string
    class?: string
    style?: string
    rows?: number
    cols?: number
    required?: boolean
    maxlength?: number
    disabled?: boolean
    readonly?: boolean
  }

  let {
    placeholder = '',
    value = $bindable(''),
    class: _class = '',
    style = '',
    rows = 4,
    cols = 20,
    required = false,
    maxlength = 500,
    disabled = false,
    readonly = false
  }: TextareaProps = $props()
</script>

<textarea
  {placeholder}
  {required}
  bind:value
  {style}
  {rows}
  {cols}
  {maxlength}
  {disabled}
  {readonly}
  class={`textarea ${_class}`}
></textarea>
