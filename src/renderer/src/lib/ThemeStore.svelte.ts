export const isBrowser = typeof window !== 'undefined'

/**  Get the initial theme from localStorage or system preference */
const getInitialTheme = (): string => {
  if (!isBrowser) {
    return 'light'
  }
  const savedTheme = localStorage.getItem('theme')
  if (savedTheme) {
    return savedTheme
  }

  // If no saved theme, check system preference
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  return prefersDark ? 'dark' : 'light'
}

export const theme = $state({
  value: getInitialTheme()
})

/**  Update theme in DOM and localStorage */
const updateTheme = (value: string) => {
  if (!isBrowser) return

  if (value === 'light') {
    document.documentElement.classList.add('light')
    document.documentElement.classList.remove('dark')
  } else {
    document.documentElement.classList.remove('light')
    document.documentElement.classList.add('dark')
  }

  localStorage.setItem('theme', value)
}

// Initialize theme
updateTheme(theme.value)

export const toggleTheme = (): void => {
  if (!isBrowser) return
  theme.value = theme.value === 'light' ? 'dark' : 'light'
  updateTheme(theme.value)
}