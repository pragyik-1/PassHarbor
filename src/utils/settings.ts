import { SettingsFile } from '../main/global'
import { DefaultSettings } from '../main/global'

let settings = await SettingsFile.read<Settings>()

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
