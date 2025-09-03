import { ElectronAPI } from '@electron-toolkit/preload'

declare global {
  interface Window {
    electron: ElectronAPI
    api: {
      password: {
        save: (vault: string, passwordFor: string, password: string) => Promise<boolean>
        get: (vault: string, passwordFor: string) => Promise<string | undefined>
        all: (vault: string) => Promise<Record<string, string>>
        delete: (vault: string, passwordFor: string) => Promise<boolean>
        encrypt: (password: string) => Promise<string>
        decrypt: (password: string) => Promise<string>
        change: (vault: string, passwordFor: string, newPassword: string) => Promise<boolean>
      }
      masterPassword: {
        create: (password: string) => Promise<boolean>
        exists: () => Promise<boolean>
        get: () => Promise<string | undefined>
      }
      vault: {
        getAll: () => Promise<string[]>
        create: (vault: string) => Promise<boolean>
        delete: (vault: string) => Promise<boolean>
      }
      settings: {
        get: () => Promise<Settings>
        set: (settings: Settings) => Promise<void>
      }
    }
  }
  export type Settings = {
    darkMode: boolean
    vaults: {
      main: string
      saveLocation: string
    }
    masterPassword: {
      enabled: boolean
      saveLocation: string
    }
    passwordGenerator: {
      length: number
      upper: boolean
      lower: boolean
      digits: boolean
      symbols: boolean
    }
    behavior: {
      hidePasswordsByDefault: boolean
    }
  }
}
