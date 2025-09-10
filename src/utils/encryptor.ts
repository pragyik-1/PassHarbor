import { createCipheriv, createDecipheriv, randomBytes, scryptSync, CipherGCMTypes } from 'crypto'
import { MasterMetaFile } from '../main/global'

const ALGORITHM: CipherGCMTypes = 'aes-256-gcm'
const SECRET_KEY_LENGTH = 32
const IV_LENGTH = 16

if (!await MasterMetaFile.exists()) {
  await MasterMetaFile.create()
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

let passphrase: string
let salt: string

const metaFileContents = await MasterMetaFile.read() || ""
if (metaFileContents.trim() !== "") {
  [passphrase, salt] = metaFileContents.split(":")
} else {
  passphrase = randomChars(32)
  salt = randomChars(32)
  await MasterMetaFile.write(`${passphrase}:${salt}:`)
}

const key = scryptSync(passphrase, salt, SECRET_KEY_LENGTH)

export function encrypt(text: string): string {
  if (typeof text !== 'string') {
    throw new Error(`encrypt expected a string but got ${typeof text}`)
  }
  const iv = randomBytes(IV_LENGTH)
  const cipher = createCipheriv(ALGORITHM, key, iv)
  let encrypted = cipher.update(text, 'utf8', 'hex')
  encrypted += cipher.final('hex')
  const tag = cipher.getAuthTag()
  return `${iv.toString('hex')}:${encrypted}:${tag.toString('hex')}`
}

export function decrypt(text: string): string {
  if (typeof text !== 'string') {
    throw new Error(`decrypt expected a string but got ${typeof text}`)
  }
  const parts = text.split(':')
  if (parts.length !== 3) {
    throw new Error('Invalid encrypted text format')
  }
  const iv = Buffer.from(parts[0], 'hex')
  const encrypted = parts[1]
  const tag = Buffer.from(parts[2], 'hex')
  const decipher = createDecipheriv(ALGORITHM, key, iv)
  decipher.setAuthTag(tag)
  let decrypted = decipher.update(encrypted, 'hex', 'utf8')
  decrypted += decipher.final('utf8')
  return decrypted
}
