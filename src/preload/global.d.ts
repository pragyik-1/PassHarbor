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
        change: (vault: string, passwordFor: string, newPassword: string) => Promise<boolean>
      }
      masterPassword: {
        create: (password: string) => Promise<boolean>
        exists: () => Promise<boolean>
        verify: (password: string) => Promise<boolean>
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
      symbolsToUse: string
      upper: boolean
      lower: boolean
      digits: boolean
      symbols: boolean
    }
  }
}
