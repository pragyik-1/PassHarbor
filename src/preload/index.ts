import { contextBridge, ipcRenderer } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'

// Custom APIs for renderer
const api = {
  password: {
    get(vault: string = 'main', passwordFor: string): Promise<string | undefined> {
      return ipcRenderer.invoke('password:get', vault, passwordFor)
    },
    save(vault: string = 'main', passwordFor: string, password: string): Promise<boolean> {
      return ipcRenderer.invoke('password:save', vault, passwordFor, password)
    },
    all(vault: string = 'main'): Promise<Record<string, string>> {
      return ipcRenderer.invoke('password:all', vault)
    },
    delete(vault: string = 'main', passwordFor: string): Promise<boolean> {
      return ipcRenderer.invoke('password:delete', vault, passwordFor)
    },
    encrypt(password: string): Promise<string> {
      return ipcRenderer.invoke('password:encryptAndDecrypt', password)
    },
    decrypt(password: string): Promise<string> {
      return ipcRenderer.invoke('password:encryptAndDecrypt', password)
    },
    change(vault: string = 'main', passwordFor: string, newPassword: string): Promise<boolean> {
      return ipcRenderer.invoke('password:change', vault, passwordFor, newPassword)
    }
  },

  masterPassword: {
    create(password: string): Promise<boolean> {
      return ipcRenderer.invoke('masterpassword:create', password)
    },
    exists(): Promise<boolean> {
      return ipcRenderer.invoke('masterpassword:exists')
    },
    get(): Promise<string | undefined> {
      return ipcRenderer.invoke('masterpassword:get')
    }
  },

  vault: {
    getAll(): Promise<string[]> {
      return ipcRenderer.invoke('vault:all')
    },
    create(vault: string): Promise<boolean> {
      return ipcRenderer.invoke('vault:create', vault)
    },
    delete(vault: string): Promise<boolean> {
      return ipcRenderer.invoke('vault:delete', vault)
    }
  }
}

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', electronAPI)
    contextBridge.exposeInMainWorld('api', api)
  } catch (error) {
    console.error(error)
  }
} else {
  window.electron = electronAPI
  window.api = api
}
