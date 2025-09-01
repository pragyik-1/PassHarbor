/**
 * (Utility)
 * Reviews the strength of a password and returns a score as string.
 */
export function analyzePasswordSecurity(password: string) {
  if (!password) {
    return {
      score: 0,
      strength: 'weak',
      length: 0,
      hasUpper: false,
      hasLower: false,
      hasDigit: false,
      hasSymbol: false,
      hasRepetition: false
    }
  }

  let score = 0

  const length = password.length
  const hasUpper = /[A-Z]/.test(password)
  const hasLower = /[a-z]/.test(password)
  const hasDigit = /[0-9]/.test(password)
  const hasSymbol = /[^A-Za-z0-9]/.test(password)
  const hasRepetition = /([a-zA-Z0-9!@#$%^&*])\1{2,}/.test(password)

  const varietyCount = [hasUpper, hasLower, hasDigit, hasSymbol].filter(Boolean).length

  // Base scoring
  if (length >= 12) score += 2
  else if (length >= 8) score += 1

  score += varietyCount
  if (hasRepetition) score -= 1
  if (length >= 16 && varietyCount >= 3) score += 1

  let strength: 'weak' | 'medium' | 'strong'
  if (score >= 6) strength = 'strong'
  else if (score >= 4) strength = 'medium'
  else strength = 'weak'

  return {
    score,
    strength,
    length,
    hasUpper,
    hasLower,
    hasDigit,
    hasSymbol,
    hasRepetition
  }
}

/**
 * (Utility)
 * Returns a color based on the strength of a password.
 */
export function getStrengthColor(strength: string) {
  switch (strength) {
    case 'strong':
      return 'green'
    case 'medium':
      return 'goldenrod'
    case 'weak':
    default:
      return 'red'
  }
}
