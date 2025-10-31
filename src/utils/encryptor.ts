// encryptor.ts
import {
  createCipheriv,
  createDecipheriv,
  randomBytes,
  scryptSync,
  hkdfSync,
  CipherGCMTypes
} from 'crypto'
import { MasterMetaFile } from '../main/global'

const ALGORITHM: CipherGCMTypes = 'aes-256-gcm'
const KEY_LEN = 32
const IV_LEN = 12
const PER_MSG_SALT_LEN = 16

type ScryptOptions = {
  N: number
  r: number
  p: number
  maxmem: number
}

const SCRYPT_PARAMS: ScryptOptions = {
  N: 16384,
  r: 8,
  p: 1,
  maxmem: 32 * 1024 * 1024
}

// optional info label for HKDF derivation to ensure domain separation
const HKDF_INFO = Buffer.from('my-app:master->permsg:v1', 'utf8')

export const randomChars = (length: number): string => {
  if (length <= 0) return ''
  const charset =
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;,.<>?'
  const randBytes = randomBytes(length)
  let result = ''
  const charsetLength = charset.length
  for (let i = 0; i < length; i++) {
    result += charset[randBytes[i] % charsetLength]
  }
  return result
}

// masterSecret is stored as string; when empty -> not initialized
let masterSecret: string = ''
// cachedMasterKey holds the scrypt-derived key in memory after init
let cachedMasterKey: Buffer | null = null

export async function initEncryptor(): Promise<void> {
  if (cachedMasterKey) return // already initialized

  await MasterMetaFile.ensure()

  const metaBuffer = await MasterMetaFile.read()
  const metaContents = metaBuffer?.toString().trim() ?? ''
  if (metaContents) {
    masterSecret = metaContents.split(':')[0] ?? ''
  } else {
    masterSecret = randomChars(64)
    await MasterMetaFile.write(`${masterSecret}:`)
  }

  if (!masterSecret) {
    throw new Error('Failed to obtain master secret')
  }

  const masterSalt = Buffer.from('master-key-salt-v1', 'utf8')
  const masterKey = scryptSync(masterSecret, masterSalt, KEY_LEN, SCRYPT_PARAMS as any)
  cachedMasterKey = Buffer.from(masterKey)
  masterKey.fill(0)
}

function derivePerMessageKey(perMsgSalt: Buffer): Buffer {
  if (!cachedMasterKey) {
    throw new Error('Encryptor not initialized; call initEncryptor() first')
  }
  // hkdfSync(digest, ikm, salt, info, keylen)
  const derived = hkdfSync('sha256', cachedMasterKey, perMsgSalt, HKDF_INFO, KEY_LEN)
  return Buffer.isBuffer(derived) ? derived : Buffer.from(derived)
}

type CipherPayloadV2 = {
  v: number
  kdf: { type: string; scrypt?: { N: number; r: number; p: number } }
  salt: string
  iv: string
  ct: string
  tag: string
}

export function encrypt(text: string): string {
  if (typeof text !== 'string') {
    throw new TypeError(`encrypt expected string, got ${typeof text}`)
  }
  if (!cachedMasterKey) {
    throw new Error('Encryptor not initialized; call initEncryptor() first')
  }

  const perMsgSalt = randomBytes(PER_MSG_SALT_LEN)
  const iv = randomBytes(IV_LEN)
  const key = derivePerMessageKey(perMsgSalt)

  try {
    const cipher = createCipheriv(ALGORITHM, key, iv)
    let encrypted = cipher.update(text, 'utf8', 'hex')
    encrypted += cipher.final('hex')
    const tag = cipher.getAuthTag()

    const payload: CipherPayloadV2 = {
      v: 2,
      kdf: { type: 'scrypt+hkdf', scrypt: { N: SCRYPT_PARAMS.N, r: SCRYPT_PARAMS.r, p: SCRYPT_PARAMS.p } },
      salt: perMsgSalt.toString('hex'),
      iv: iv.toString('hex'),
      ct: encrypted,
      tag: tag.toString('hex')
    }

    return Buffer.from(JSON.stringify(payload), 'utf8').toString('base64')
  } finally {
    // zero key material
    key.fill(0)
  }
}

export function decrypt(encoded: string): string {
  if (typeof encoded !== 'string') {
    throw new TypeError(`decrypt expected string, got ${typeof encoded}`)
  }
  if (!cachedMasterKey) {
    throw new Error('Encryptor not initialized; call initEncryptor() first')
  }

  let payload: CipherPayloadV2
  try {
    const raw = Buffer.from(encoded, 'base64').toString('utf8')
    payload = JSON.parse(raw) as CipherPayloadV2
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
  const key = derivePerMessageKey(perMsgSalt)

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
