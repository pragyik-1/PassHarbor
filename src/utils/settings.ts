import { SettingsFile } from '../main/global'
import { DefaultSettings } from '../main/global'

const settings = await SettingsFile.read<Settings>()

export function getSettings(): Settings {
  return {
    ...DefaultSettings,
    ...settings
  }
}

export async function resetSettings() {
  await SettingsFile.write(DefaultSettings)
}
