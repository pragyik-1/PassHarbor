import { Path, File, Dir, Json } from '../utils/path-helper'
import { app } from 'electron'

export const DefaultSettings = {
  darkMode: false,
  vaults: {
    main: 'main',
    saveLocation: 'vaults'
  },
  masterPassword: {
    enabled: false,
    saveLocation: 'mpwd.txt'
  },
  enabled: false,
  saveLocation: 'settings.json',
  passwordGenerator: {
    length: 12,
    upper: true,
    lower: true,
    digits: true,
    symbols: true
  }
}

// Helper function
export async function initSettings() {
  if (!(await SettingsFile.exists())) {
    await SettingsFile.create()
    await SettingsFile.write(DefaultSettings)
  }
}

export const RootDir = new Path(`${app.getPath('userData')}/logsafe`)
export const SettingsFile = new Json(RootDir.join('settings.json'))

await initSettings()

let settings = await SettingsFile.read<Settings>()

export const VaultsDir = new Dir(RootDir.join(settings.vaults.saveLocation))
export const MasterPasswordPath = new File(RootDir.join(settings.masterPassword.saveLocation))
