export const currentVault = $state({
  value: localStorage.getItem('currentVault') || 'main'
})

export const settings = $state({
  value: await window.api.settings.get()
})

export async function refreshVault(vault: string): Promise<void> {
  currentVault.value = vault
  localStorage.setItem('currentVault', vault)

  vaults.value = await window.api.vault.getAll()
  passwords.value = await window.api.password.all(currentVault.value)
}

export function generatePassword(length = 16): string {
  const upper = settings.value.passwordGenerator.upper ? 'ABCDEFGHIJKLMNOPQRSTUVWXYZ' : ""
  const lower = settings.value.passwordGenerator.lower ?'abcdefghijklmnopqrstuvwxyz' : ""
  const digits = settings.value.passwordGenerator.digits ? '0123456789' : ""
  const symbols = settings.value.passwordGenerator.symbolsToUse

  const all = upper + lower + digits + symbols
  const getRandomChar = (chars: string) =>
    chars[Math.floor((crypto.getRandomValues(new Uint32Array(1))[0] / 2 ** 32) * chars.length)]

  if (length < 8) length = 8

  const password: string[] = []

  password.push(getRandomChar(upper))
  password.push(getRandomChar(lower))
  password.push(getRandomChar(digits))
  password.push(getRandomChar(symbols))

  const remainingLength = length - password.length
  const randomValues = new Uint32Array(remainingLength)
  crypto.getRandomValues(randomValues)

  for (let i = 0; i < remainingLength; i++) {
    password.push(all[randomValues[i] % all.length])
  }

  for (let i = password.length - 1; i > 0; i--) {
    const j = crypto.getRandomValues(new Uint32Array(1))[0] % (i + 1)
    const tmp = password[i]
    password[i] = password[j]
    password[j] = tmp
  }

  return password.join('')
}

export const vaults = $state({
  value: await window.api.vault.getAll()
})

export const passwords = $state({
  value: await window.api.password.all(currentVault.value)
})
4
