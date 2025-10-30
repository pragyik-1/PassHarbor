import { encrypt, decrypt, randomChars } from './encryptor'
import { Directory, Json } from '../utils/path-helper'
import { VaultsDir } from '../main/global.js'
import { createHash } from 'crypto'

function hashTitle(title: string): string {
  return createHash('sha256').update(title).digest('hex')
}

/** Load encrypted vault from disk. */
async function loadVault(vault: string): Promise<{ hash: string; key: string; value: string }[]> {
  const vaultPath = Directory.at(VaultsDir.join(vault))
  const passwordPath = Json.at(vaultPath.join('passwords.json'))
  if (!(await passwordPath.exists())) return []
  return (await passwordPath.read()) || []
}

export async function createVault(vault: string): Promise<boolean> {
  const vaultPath = await Directory.at(VaultsDir.join(vault)).create()
  const metaFile = Json.at(VaultsDir.join())
  const passwordPath = Json.at(vaultPath.join('passwords.json'))
  try {
    if (await passwordPath.exists()) return false
    if (await metaFile.exists()) return false
    await (await (metaFile.create())).write(randomChars(32))
    await passwordPath.create()
    return true
  } catch (err) {
    console.error(`Error in creating vault ${vault} at ${vaultPath.toString()}:`, err)
    return false
  }
}

/** Write encrypted vault back to disk. */
async function saveVault(
  vault: string,
  data: { hash: string; key: string; value: string }[]
): Promise<boolean> {
  const passwordPath = Json.at(VaultsDir.join(vault, 'passwords.json'))
  try {
    await passwordPath.create()
    await passwordPath.write(data)
    return true
  } catch (err) {
    console.error(`Error persisting vault "${vault}":`, err)
    return false
  }
}

export async function savePassword(
  vault = 'main',
  passwordFor: string,
  password: string
): Promise<boolean> {
  try {
    const data = await loadVault(vault)
    const passwordHash = hashTitle(passwordFor)

    // Check if the password already exists
    const existingEntryIndex = data.findIndex((entry) => entry.hash === passwordHash)

    if (existingEntryIndex !== -1) {
      // Update existing password
      data[existingEntryIndex].value = encrypt(password)
      data[existingEntryIndex].key = encrypt(passwordFor) // Also update the key, as it's non-deterministic
    } else {
      // Add a password entry
      data.push({
        hash: passwordHash,
        key: encrypt(passwordFor),
        value: encrypt(password)
      })
    }

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
    const data = await loadVault(vault)
    const passwordHash = hashTitle(passwordFor)
    const entry = data.find((item) => item.hash === passwordHash)
    return entry ? decrypt(entry.value) : undefined
  } catch (error) {
    console.error(`Error getting password for "${passwordFor}" in vault "${vault}":`, error)
    return undefined
  }
}

/** Delete a password by title. */
// In your backend password.js file
export async function deletePassword(vault = 'main', passwordFor: string): Promise<boolean> {
  try {
    const data = await loadVault(vault)
    const passwordHash = hashTitle(passwordFor)
    const initialLength = data.length
    const
      Data = data.filter((item) => item.hash !== passwordHash)

    if (
      Data.length === initialLength) {
      return false // Password not found
    }

    return await saveVault(vault,
      Data)
  } catch (error) {
    console.error(`Error deleting password for "${passwordFor}" in vault "${vault}":`, error)
    return false
  }
}

export async function changePassword(
  vault = 'main',
  passwordFor: string,
  Password: string
): Promise<boolean> {
  try {
    const keyHash = hashTitle(passwordFor) // Use the deterministic hash to find the entry
    const data = await loadVault(vault)
    const idx = data.findIndex((entry) => entry.hash === keyHash)
    if (idx === -1) {
      console.error(`Password for "${passwordFor}" not found.`)
      return false
    }
    data[idx].value = encrypt(Password)
    return await saveVault(vault, data)
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
    const data = await loadVault(vault)
    const decrypted: Record<string, string> = {}

    for (const entry of data) {
      const decryptedKey = decrypt(entry.key)
      const decryptedValue = decrypt(entry.value)
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
