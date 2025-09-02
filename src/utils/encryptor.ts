import { createCipheriv, createDecipheriv, randomBytes, scryptSync, CipherGCMTypes } from 'crypto'

const ALGORITHM: CipherGCMTypes = 'aes-256-gcm'
const SECRET_KEY_LENGTH = 32
const IV_LENGTH = 16

const passphrase = 'encpassword'
const key = scryptSync(passphrase, 'salt', SECRET_KEY_LENGTH)

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
