import { page } from '$app/state'

let manualLock = false

export function isNotLoginPage() {
  return page.route.id != '/'
}

/**
 * Records that the vault was locked explicitly so the unlock screen can skip
 * its automatic keychain attempt.
 */
export function markManualLock() {
  manualLock = true
}

/** True until the vault is unlocked or the app session ends. */
export function isManualLockPending() {
  return manualLock
}

/** Clears the explicit-lock hint after a successful unlock. */
export function clearManualLock() {
  manualLock = false
}
