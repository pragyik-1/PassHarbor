import { encrypt, decrypt } from './encryptor'
import { Dir, Json } from '../utils/path-helper'
import { VaultsDir } from '../main/globals.js'

/** Load encrypted vault from disk. */
async function loadVault(vault: string): Promise<Record<string, string>> {
  const vaultPath = new Dir(VaultsDir.join(vault))
  const passwordPath = new Json(vaultPath.join('passwords.json'))
  if (!(await passwordPath.exists())) return {}
  return (await passwordPath.read()) || {}
}

export async function createVault(vault: string): Promise<boolean> {
  const vaultPath = new Dir(VaultsDir.join(vault))
  vaultPath.create()
  const passwordPath = new Json(vaultPath.join('passwords.json'))
  try {
    if (await passwordPath.exists()) return false
    await passwordPath.create()
    return true
  } catch (err) {
    console.error(`Error in creating vault ${vault} at ${vaultPath.toString()}:`, err)
    return false
  }
}

/** Write encrypted vault back to disk. */
async function saveVault(vault: string, data: Record<string, string>): Promise<boolean> {
  const passwordPath = new Json(VaultsDir.join(vault, 'passwords.json'))
  try {
    await passwordPath.create()
    await passwordPath.write(data)
    return true
  } catch (err) {
    console.error(`Error persisting vault "${vault}":`, err)
    return false
  }
}

/**
 *  - Save an encrypted password
 *  - Only to be used by the backend as it risks duplication
 */
export async function savePassword(
  vault = 'main',
  passwordFor: string,
  password: string
): Promise<boolean> {
  try {
    const encryptedKey = encrypt(passwordFor)
    const encryptedValue = encrypt(password)
    if (!encryptedKey || !encryptedValue) {
      console.log('Error encrypting password')
      return false
    }
    const data = await loadVault(vault)
    data[encryptedKey] = encryptedValue
    return await saveVault(vault, data)
  } catch (error) {
    console.error(`Error saving password for "${passwordFor}" in vault "${vault}":`, error)
    return false
  }
}

/**
 * - Wrapper for Save password
 * - Can be used by the frontend
 */
export async function savePasswordIfMissing(
  vault = 'main',
  passwordFor: string,
  password: string
): Promise<boolean> {
  try {
    if (await getPassword(vault, passwordFor)) return false
    return await savePassword(vault, passwordFor, password)
  } catch (error) {
    console.error(`Error saving password for "${passwordFor}" in vault "${vault}":`, error)
    return false
  }
}

/** Get a decrypted password by title. */
export async function getPassword(
  vault = 'main',
  passwordFor: string
): Promise<string | undefined> {
  try {
    const encryptedKey = encrypt(passwordFor)
    if (!encryptedKey) return undefined
    const data = await loadVault(vault)
    const encryptedValue = data[encryptedKey]
    return encryptedValue ? decrypt(encryptedValue)! : undefined
  } catch (error) {
    console.error(`Error getting password for "${passwordFor}" in vault "${vault}":`, error)
    return undefined
  }
}

/** Delete a password by title. */
export async function deletePassword(vault = 'main', passwordFor: string): Promise<boolean> {
  try {
    const encryptedKey = encrypt(passwordFor)
    if (!encryptedKey) return false
    const data = await loadVault(vault)
    if (!(encryptedKey in data)) return false
    delete data[encryptedKey]
    return await saveVault(vault, data)
  } catch (error) {
    console.error(`Error deleting password for "${passwordFor}" in vault "${vault}":`, error)
    return false
  }
}

export async function changePassword(
  vault = 'main',
  passwordFor: string,
  newPassword: string
): Promise<boolean> {
  try {
    const password = getPassword(vault, passwordFor)
    if (!password) return false
    return await savePassword(vault, passwordFor, newPassword)
  } catch (error) {
    console.error(`Error changing password for "${passwordFor}" in vault "${vault}":`, error)
    return false
  }
}

/**
 * Returns a fully decrypted record of all passwords with titles as keys.
 */
export async function getPasswords(vault = 'main'): Promise<Record<string, string>> {
  try {
    const encrypted = await loadVault(vault)
    const decrypted: Record<string, string> = {}

    for (const encryptedKey in encrypted) {
      const decryptedKey = decrypt(encryptedKey)
      const decryptedValue = decrypt(encrypted[encryptedKey])
      if (!decryptedKey || !decryptedValue) continue
      decrypted[decryptedKey] = decryptedValue
    }

    return decrypted
  } catch (error) {
    console.error(`Error listing passwords in vault "${vault}":`, error)
    return {}
  }
}

/** List all available vaults */
export async function getVaults(): Promise<string[]> {
  try {
    await VaultsDir.create()
    console.log((await VaultsDir.list()).map((vault) => vault.basename()))
    return (await VaultsDir.list()).map((vault) => vault.basename())
  } catch (error) {
    console.error(`Error getting vault list:`, error)
    return []
  }
}
