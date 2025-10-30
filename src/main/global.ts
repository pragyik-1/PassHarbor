import { Path, File, Directory, Json } from '../utils/path-helper'
import { app } from 'electron'

export const RootDir = Path.at(`${app.getPath('userData')}/logsafe_test`)

export const DefaultSettings = {
  darkMode: true,
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

export const SettingsFile = Json.at(RootDir.join('settings.json'))

await initSettings()

const settings = await SettingsFile.read<Settings>()

export const VaultsDir = Directory.at(settings.vaults.saveLocation)
export const MasterPasswordFile = File.at(settings.masterPassword.saveLocation)
export const MasterMetaFile = File.at(RootDir.join('MasterMeta')) // Stores some additional info such as salts
