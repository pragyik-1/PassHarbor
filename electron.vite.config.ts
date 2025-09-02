import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    // Add this section 👇
    build: {
      rollupOptions: {
        output: {
          format: 'es'
        }
      }
    }
    // Add this section 👆
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    // Add this section 👇
    build: {
      rollupOptions: {
        output: {
          format: 'es'
        }
      }
    }
    // Add this section 👆
  },
  renderer: {
    plugins: [svelte()]
  }
})
