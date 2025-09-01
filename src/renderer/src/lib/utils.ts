import { onDestroy, onMount } from 'svelte'

/**
 * Calls the given handler when the user clicks outside of the given node.
 * @param {HTMLElement} node The node to check for clicks outside of.
 * @param {() => void} handler The function to call when a click outside occurs.
 */
export function onClickOutside(node: HTMLElement, handler: () => void): void {
  const handleClick = (event: MouseEvent): void => {
    if (node && !node.contains(event.target as Node)) {
      handler()
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClick, true)
  })

  onDestroy(() => {
    document.removeEventListener('click', handleClick, true)
  })
}

/**
 * Registers a handler function to be called when the Escape key is pressed.
 * The handler is automatically attached to the 'keydown' event when the component
 * is mounted and removed when the component is destroyed.
 *
 * @param handler - The function to be called when the Escape key is pressed.
 */

export function onEscapeKey(handler: () => void): void {
  const handleKeydown = (event: KeyboardEvent): void => {
    if (event.key === 'Escape') {
      handler()
    }
  }

  onMount(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeydown)
  })
}

/**
 * Calls the given callback function after the given delay. The callback function
 * is automatically cleared when the component is destroyed. The function
 * returned by this hook can be used to manually clear the timeout.
 *
 * @param callback - The function to be called after the delay.
 * @param delay - The delay in milliseconds to wait before calling the callback.
 * @returns A function to manually clear the timeout.
 */
export function useTimeout(callback: () => void, delay: number): void | (() => void) {
  const timeoutId = setTimeout(callback, delay)

  onDestroy(() => {
    clearTimeout(timeoutId)
  })

  return () => clearTimeout(timeoutId)
}

/**
 * Calls the given callback function at the given interval. The callback function
 * is automatically cleared when the component is destroyed. The function
 * returned by this hook can be used to manually clear the interval.
 *
 * @param callback - The function to be called at the given interval.
 * @param interval - The interval in milliseconds to wait between calls.
 * @returns A function to manually clear the interval.
 */
export function useInterval(callback: () => void, interval: number): void | (() => void) {
  const intervalId = setInterval(callback, interval)

  onDestroy(() => {
    clearInterval(intervalId)
  })

  return () => clearInterval(intervalId)
}

/** All the available colors for the theme in a map for easy access compared to CSS variables */
export const colors = {
  primary: 'var(--primary)',
  secondary: 'var(--secondary)',
  text: 'var(--text)',
  surface: 'var(--surface)',
  background: 'var(--background)',
  border: 'var(--border)',
  hover: 'var(--hover)',
  primaryHover: 'var(--primaryHover)',
  primaryText: 'var(--primary-text)',
  secondaryHover: 'var(--secondaryHover)',
  error: 'var(--error)',
  errorHover: 'var(--errorHover)',
  info: 'var(--info)',
  infoHover: 'var(--infoHover)',
  success: 'var(--success)',
  successHover: 'var(--successHover)',
  warning: 'var(--warning)',
  warningHover: 'var(--warningHover)',
  input: 'var(--input)',
  inputHover: 'var(--inputHover)'
} as const
