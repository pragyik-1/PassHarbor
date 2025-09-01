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
import { encrypt } from '../utils/encryptor'
import { Path, File, Dir } from '../utils/path-helper'
import { MasterPasswordPath } from './globals.js'

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

ipcMain.handle('password:encryptAndDecrypt', (_, password) => {
  return encrypt(password)
})

ipcMain.handle('password:change', async (_, vault = 'main', passwordFor, password) => {
  return await changePassword(vault, passwordFor, password)
})

ipcMain.handle('masterpassword:exists', async () => {
  return await MasterPasswordPath.exists()
})

ipcMain.handle('masterpassword:create', async (_, password) => {
  return await MasterPasswordPath.write(password)
})

ipcMain.handle('masterpassword:get', async () => {
  return await MasterPasswordPath.read()
})

ipcMain.handle('vault:exists', async (_, vault = 'main') => {
  return await new Path(`data/vaults/${vault}/passwords.json`).exists()
})

ipcMain.handle('vault:create', async (_, vault = 'main') => {
  return await createVault(vault)
})

ipcMain.handle('vault:delete', async (_, vault = 'main') => {
  const _vault = new Dir(`data/vaults/${vault}`)
  return await _vault.remove()
})

ipcMain.handle('vault:all', async () => {
  return await getVaults()
})
