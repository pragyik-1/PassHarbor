import { Path, File, Dir, Json } from '../utils/path-helper'
import { app } from 'electron'

export const RootDir = new Path(`${app.getPath('userData')}/logsafe`)

export const DefaultSettings = {
  darkMode: false,
  vaults: {
    all: [],
    main: 'main',
    saveLocation: RootDir.join('vaults').toString()
  },
  masterPassword: {
    enabled: false,
    saveLocation: RootDir.join('master-password.pwd').toString()
  },
  enabled: false,
  passwordGenerator: {
    symbolsToUse: '_-.',
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

export const SettingsFile = new Json(RootDir.join('settings.json'))

await initSettings()

const settings = await SettingsFile.read<Settings>()

export const VaultsDir = new Dir(settings.vaults.saveLocation)
export const MasterPasswordPath = new File(settings.masterPassword.saveLocation)
