import { SettingsFile, VaultsDir } from '../main/global'
import { DefaultSettings } from '../main/global'

let settings = await SettingsFile.read<Settings>()

if (await VaultsDir.exists()) {
  settings = {
    ...settings,
    vaults: {
      ...settings.vaults
    }
  }
}

export function getSettings(): Settings {
  return {
    ...DefaultSettings,
    ...settings
  }
}

export async function resetSettings() {
  await SettingsFile.write(DefaultSettings)
}

export async function setSettings(_settings: Settings) {
  await SettingsFile.write(_settings)
  settings = _settings
}
