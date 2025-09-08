import { dialog, ipcMain } from 'electron'
import {
  changePassword,
  createVault,
  deletePassword,
  getPassword,
  getPasswords,
  getVaults,
  savePasswordIfMissing
} from '../utils/password-utils'
import { decrypt, encrypt } from '../utils/encryptor'
import { Dir } from '../utils/path-helper'
import { MasterPasswordPath, VaultsDir } from './global.js'
import { setSettings, getSettings } from '../utils/settings'

ipcMain.handle('dialog:selectFile', async (_, options) => {
  try {
    const result = await dialog.showOpenDialog({
      properties: ['openFile'],
      ...options
    })
    return result.filePaths[0] || null
  } catch (error) {
    return null
  }
})

ipcMain.handle('password:save', async (_, vault = 'main', passwordFor, password) => {
  return await savePasswordIfMissing(vault, passwordFor, password)
})

ipcMain.handle('password:get', async (_, vault = 'main', passwordFor) => {
  return await getPassword(vault, passwordFor)
})

ipcMain.handle('password:delete', async (_, vault = 'main', passwordFor) => {
  return await deletePassword(vault, passwordFor)
})

ipcMain.handle('password:all', async (_, vault = 'main') => {
  return await getPasswords(vault)
})

ipcMain.handle('password:change', async (_, vault = 'main', passwordFor, password) => {
  return await changePassword(vault, passwordFor, password)
})

ipcMain.handle('masterpassword:exists', async () => {
  return await MasterPasswordPath.exists()
})

ipcMain.handle('masterpassword:create', async (_, password) => {
  const encryptedPassword = encrypt(password)
  return await MasterPasswordPath.write(encryptedPassword)
})

ipcMain.handle('masterpassword:get', async () => {
  return decrypt(await MasterPasswordPath.read())
})

ipcMain.handle('vault:exists', async (_, vault = 'main') => {
  return await VaultsDir.join(vault, 'passwords.json').exists()
})

ipcMain.handle('vault:create', async (_, vault = 'main') => {
  return await createVault(vault)
})

ipcMain.handle('vault:delete', async (_, vault = 'main') => {
  const _vault = VaultsDir.join(vault).as(Dir)
  return await _vault.remove()
})

ipcMain.handle('vault:all', async () => {
  return await getVaults()
})

ipcMain.handle('settings:get', () => {
  return getSettings()
})

ipcMain.handle('settings:set', async (_, settings) => {
  return await setSettings(settings)
})
