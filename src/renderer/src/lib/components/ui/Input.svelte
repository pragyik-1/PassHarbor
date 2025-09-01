<script lang="ts">
  import Grid from './Grid.svelte'
  import '../stylesheets/Input.css'

  export type InputProps = {
    label?: string
    placeholder?: string
    type?: 'text' | 'password' | 'number' | 'file' | 'email' | 'tel' | 'directory'
    value?: string
    required?: boolean
    style?: string
    id?: string
    name?: string
    disabled?: boolean
    class?: string
    maxlength?: number
    onclick?: () => void
    onchange?: (event: Event) => void
    readonly?: boolean
    fullWidth?: boolean
    webkitdirectory?: boolean
  }

  let {
    label = '',
    placeholder = '',
    type = 'text',
    value = $bindable(''),
    required = false,
    style = '',
    id = 'default',
    class: _class = '',
    name = '',
    disabled = false,
    maxlength = 255,
    onclick = () => {},
    onchange = () => {},
    readonly = false,
    fullWidth = false,
    webkitdirectory = false
  }: InputProps = $props()

  let fileName: string = $state('')

  function handleFileChange(event: Event) {
    const input = event.target as HTMLInputElement
    if (input.files?.length) {
      // For directories, we use the first file's path (browser security limitations)
      fileName = webkitdirectory
        ? input.files[0].webkitRelativePath.split('/')[0] + '/'
        : input.files[0].name

      // Update the bound value
      value = fileName
    }
    onchange(event)
  }

  let inputClasses = $state('input-base')
  if (fullWidth) inputClasses += ' input-full-width'
  if (disabled) inputClasses += ' input-disabled'
  if (_class) inputClasses += ` ${_class}`
</script>

{#if type === 'file' || type === 'directory'}
  <Grid col>
    {#if label}
      <label for={id} class="input-label">{label}</label>
    {/if}

    <input
      {id}
      {name}
      type="file"
      {required}
      {disabled}
      {style}
      onchange={handleFileChange}
      class="hidden"
      {webkitdirectory}
    />

    <label for={id} class="file-label">
      {type === 'directory' ? '📁 Choose Directory' : '📂 Choose File'}
    </label>

    {#if fileName}
      <div class="file-info">
        Selected: {fileName}
      </div>
    {/if}
  </Grid>
{:else}
  {#if label}
    <label for={id} class="input-label">{label}</label>
  {/if}

  <input
    {id}
    {name}
    {placeholder}
    {type}
    {required}
    bind:value
    {style}
    {disabled}
    {onchange}
    {maxlength}
    {onclick}
    class={inputClasses}
    aria-describedby={id ? `${id}-helper` : undefined}
    {readonly}
  />
{/if}
