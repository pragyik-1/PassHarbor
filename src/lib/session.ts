const VAULT_NAME_KEY = "passharbor.vault-name";

/** Record the vault name for the current webview session. */
export function rememberVault(name: string): void {
  sessionStorage.setItem(VAULT_NAME_KEY, name);
}

/** Read the vault name for the current webview session, if any. */
export function recallVault(): string | null {
  return sessionStorage.getItem(VAULT_NAME_KEY);
}

/** Clear the vault name when the vault is locked. */
export function forgetVault(): void {
  sessionStorage.removeItem(VAULT_NAME_KEY);
}
