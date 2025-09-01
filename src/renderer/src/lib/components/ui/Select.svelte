<script lang="ts">
  import { fly } from 'svelte/transition'
  import Input from './Input.svelte'
  import { onClickOutside } from '../../utils'
  import ListButton from './List/ListButton.svelte'
  import List from './List/List.svelte'
  import type { InputProps } from './Input.svelte'
  import '../stylesheets/Select.css'

  export type MenuItem = {
    value: string
    label: string
  }

  type MenuProps = InputProps & {
    position?: 'bottom' | 'top' | 'left' | 'right'
    menuClass?: string
    menuStyle?: string
    duration?: number
    items?: MenuItem[]
    value?: string
  }

  let {
    position = 'bottom',
    menuClass = '',
    menuStyle = '',
    duration = 200,
    placeholder = '',
    type = 'text',
    value = $bindable(''),
    required = false,
    class: _class = '',
    style = '',
    id = '',
    onchange = (e) => {
      e.preventDefault()
    },
    name = '',
    disabled = false,
    maxlength = 255,
    items = [],
    fullWidth = false,
    label = ''
  }: MenuProps = $props()

  let open = $state(false)

  const toggleMenu = () => {
    if (!disabled) open = !open
  }

  const selectItem = (item: MenuItem) => {
    value = item.value
    open = false
  }

  let positionClass = $state('')
  switch (position) {
    case 'top':
      positionClass = 'menu-top'
      break
    case 'left':
      positionClass = 'menu-left'
      break
    case 'right':
      positionClass = 'menu-right'
      break
    default:
      positionClass = 'menu-bottom'
  }
</script>

<div use:onClickOutside={() => (open = false)} class="menu-outer">
  <div class="menu-inner">
    <div class="input-wrapper">
      <Input
        {placeholder}
        {type}
        bind:value
        {required}
        class={_class}
        {style}
        {id}
        {name}
        {disabled}
        {maxlength}
        onclick={toggleMenu}
        {onchange}
        {fullWidth}
        readonly
        {label}
      />
      <!-- Dropdown arrow (the one near the Input) -->
      <div class="dropdown-arrow">
        <svg
          class="dropdown-icon {open ? 'rotated' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </div>
    </div>

    {#if open && items.length > 0}
      <div
        style={menuStyle}
        transition:fly={{ y: position === 'top' ? 15 : -15, duration }}
        class="menu-container {menuClass} {positionClass}"
      >
        <!-- Arrow container for the menu popover arrow -->
        <div class="arrow-container">
          <svg
            class="arrow-svg"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <!-- A simple triangle arrow shape -->
            <path d="M12 4L4 12h16L12 4z" />
          </svg>
        </div>
        <List>
          {#each items as item}
            <ListButton
              onclick={(e: { preventDefault: () => void }) => {
                e.preventDefault()
                selectItem(item)
              }}
            >
              {item.label}
            </ListButton>
          {/each}
        </List>
      </div>
    {/if}
  </div>
</div>
