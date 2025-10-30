import { scryptSync, createHash } from "crypto"
import { encrypt, randomChars, decrypt } from "./encryptor"
import { MasterPasswordFile } from "../main/global"

function deriveKey(password: string, salt: string): Buffer {
    return scryptSync(password, salt, 32)
}

function hashKey(key: Buffer): string {
    return createHash('sha256').update(key).digest('hex')
}

export async function createMasterPassword(password: string): Promise<boolean> {
    if (typeof password !== 'string' || !password.length) {
        throw new Error('Invalid master password')
    }
    const salt = randomChars(16)
    const key = deriveKey(password, salt)
    const hash = hashKey(key)

    const payload = JSON.stringify({ salt, hash })
    const encryptedPayload = encrypt(payload)

    let success = true
    try {
        success = await (await MasterPasswordFile.ensure()).exists()
        MasterPasswordFile.write(encryptedPayload)
    } catch (e) {
        console.error("Failed to create master password:\n", e)
        success = false
    } finally {
        key.fill(0)
        return success
    }
}

export async function verifyMasterPassword(password: string): Promise<boolean> {
    if (typeof password !== 'string' || !password.length) {
        throw new Error('Invalid master password')
    }
    if (!(await MasterPasswordFile.exists())) {
        return false
    }
    try {
        const encryptedData = await MasterPasswordFile.read()
        const decrypted = decrypt(encryptedData)
        const { salt, hash } = JSON.parse(decrypted)

        const derived = deriveKey(password, salt)
        const checkHash = hashKey(derived)

        return checkHash === hash
    } catch (err) {
        console.error('Master password verification failed:', err)
        return false
    }
}