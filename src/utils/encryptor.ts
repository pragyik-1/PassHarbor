import { createCipheriv, createDecipheriv, randomBytes, scryptSync, CipherGCMTypes } from 'crypto'
import { MasterMetaFile, MasterPasswordFile, SettingsFile } from '../main/global'


const ALGORITHM: CipherGCMTypes = 'aes-256-gcm'
const KEY_LEN = 32
const IV_LEN = 12
const PER_MSG_SALT_LEN = 16

const SCRYPT_PARAMS = {
  N: 16384,
  r: 8,
  p: 1,
  maxmem: 32 * 1024 * 1024
}


export const randomChars = (length: number) => {
  if (length <= 0) {
    return ''
  }

  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;,.<>?'
  let result = ''
  const charsetLength = charset.length
  const randBytes = randomBytes(length)

  for (let i = 0; i < length; i++) {
    const randomIndex = randBytes[i] % charsetLength
    result += charset[randomIndex]
  }

  return result
}
await MasterMetaFile.ensure()
let masterSecret: string

export async function initEncryptor(): Promise<void> {
  if (masterSecret) return // already initialized

  await MasterMetaFile.ensure()

  const metaContents = (await MasterMetaFile.read())?.toString().trim()
  if (metaContents) {
    masterSecret = metaContents.split(':')[0]
  } else {
    masterSecret = randomChars(64)
    await MasterMetaFile.write(`${masterSecret}:`)
  }

}

function deriveKey(master: string, salt: Buffer): Buffer {
  return scryptSync(master, salt, KEY_LEN, SCRYPT_PARAMS)
}

export function encrypt(text: string): string {
  if (typeof text !== 'string') {
    throw new Error(`encrypt expected string, got ${typeof text}`)
  }

  const perMsgSalt = randomBytes(PER_MSG_SALT_LEN)
  const iv = randomBytes(IV_LEN)
  const key = deriveKey(masterSecret, perMsgSalt)

  try {
    const cipher = createCipheriv(ALGORITHM, key, iv)
    let encrypted = cipher.update(text, 'utf8', 'hex')
    encrypted += cipher.final('hex')
    const tag = cipher.getAuthTag()

    const payload = {
      v: 2,
      kdf: { N: SCRYPT_PARAMS.N, r: SCRYPT_PARAMS.r, p: SCRYPT_PARAMS.p },
      salt: perMsgSalt.toString('hex'),
      iv: iv.toString('hex'),
      ct: encrypted,
      tag: tag.toString('hex')
    }

    return Buffer.from(JSON.stringify(payload)).toString('base64')
  } finally {
    key.fill(0)
  }
}

export function decrypt(encoded: string): string {
  if (typeof encoded !== 'string') {
    throw new Error(`decrypt expected string, got ${typeof encoded}`)
  }

  let payload
  try {
    payload = JSON.parse(Buffer.from(encoded, 'base64').toString('utf8'))
  } catch {
    throw new Error('Invalid ciphertext format')
  }

  if (!payload || payload.v !== 2) {
    throw new Error('Unsupported ciphertext version')
  }

  const { salt, iv, ct, tag } = payload
  if (!salt || !iv || !ct || !tag) {
    throw new Error('Corrupt ciphertext payload')
  }

  const perMsgSalt = Buffer.from(salt, 'hex')
  const ivBuf = Buffer.from(iv, 'hex')
  const tagBuf = Buffer.from(tag, 'hex')
  const key = deriveKey(masterSecret, perMsgSalt)

  try {
    const decipher = createDecipheriv(ALGORITHM, key, ivBuf)
    decipher.setAuthTag(tagBuf)
    let decrypted = decipher.update(ct, 'hex', 'utf8')
    decrypted += decipher.final('utf8')
    return decrypted
  } catch {
    throw new Error('Decryption failed (wrong key or corrupted data)')
  } finally {
    key.fill(0)
  }
}