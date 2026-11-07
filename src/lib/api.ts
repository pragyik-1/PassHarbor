import { invoke } from "@tauri-apps/api/core";

/**
 * Typed wrappers around the Rust commands in `src-tauri/src/frontend_api.rs`.
 *
 * Tauri converts command arguments from Rust's `snake_case` to `camelCase`
 * on the JavaScript side, so `entry_id` is passed as `entryId`. Command
 * *payloads* keep their Rust field names.
 */

export interface SessionInfo {
  unlocked: boolean;
  vault_name: string | null;
}

export interface Entry {
  id: number;
  kind: string;
  name: string;
  username: string | null;
  email: string | null;
  url: string | null;
  note: string | null;
  created_at: number;
  updated_at: number;
}

export interface EntrySummary {
  id: number;
  kind: string;
  name: string;
  username: string | null;
  email: string | null;
  url: string | null;
  created_at: number;
  updated_at: number;
}

export interface KindCount {
  kind: string;
  count: number;
}

export interface EntryQuery {
  search?: string | null;
  kind?: string | null;
  sort?: "name" | "updated" | "created" | null;
}

export interface EntryQueryResult {
  entries: EntrySummary[];
  kinds: KindCount[];
}

export interface Password {
  id: number;
  entry_id: number;
  password: string;
  created_at: number;
}

export interface EntryDetail {
  entry: Entry;
  current_password: string | null;
  history: Password[];
}

export interface NewEntry {
  kind: string;
  name: string;
  username?: string | null;
  email?: string | null;
  url?: string | null;
  note?: string | null;
  password?: string | null;
}

export interface UpdateEntry {
  kind: string;
  name: string;
  username?: string | null;
  email?: string | null;
  url?: string | null;
  note?: string | null;
}

/** Open the vault and return session information. */
export async function unlock(passphrase: string): Promise<SessionInfo> {
  return await invoke<SessionInfo>("unlock_cmd", { passphrase });
}

/** Check the current session status without modifying state. */
export async function getSession(): Promise<SessionInfo> {
  return await invoke<SessionInfo>("get_session_cmd");
}

/** Lock the vault: close the database and release the key material. */
export async function lock(): Promise<void> {
  await invoke<void>("lock_cmd");
}

export async function addEntry(entry: NewEntry): Promise<number> {
  return await invoke<number>("add_entry_cmd", { entry });
}

export async function listEntries(): Promise<Entry[]> {
  return await invoke<Entry[]>("list_entries_cmd");
}

export async function queryEntries(
  query: EntryQuery = {},
): Promise<EntryQueryResult> {
  return await invoke<EntryQueryResult>("query_entries_cmd", { query });
}

export async function getEntry(id: number): Promise<EntryDetail> {
  return await invoke<EntryDetail>("get_entry_cmd", { id });
}

export async function updateEntry(
  id: number,
  entry: UpdateEntry,
): Promise<void> {
  await invoke<void>("update_entry_cmd", { id, entry });
}

export async function deleteEntry(id: number): Promise<void> {
  await invoke<void>("delete_entry_cmd", { id });
}

export async function setPassword(
  entryId: number,
  password: string,
): Promise<void> {
  await invoke<void>("set_password_cmd", { entryId, password });
}

export async function addCustomKind(kind: string): Promise<void> {
  await invoke<void>("add_custom_kind_cmd", { kind });
}

export async function removeCustomKind(kind: string): Promise<void> {
  await invoke<void>("remove_custom_kind_cmd", { kind });
}

export async function resetCustomKinds(): Promise<void> {
  await invoke<void>("reset_custom_kinds_cmd");
}

export async function listCustomKinds(): Promise<string[]> {
  return await invoke<string[]>("list_custom_kinds_cmd");
}

/** Whether an IPC failure means the vault session has been closed. */
export function isVaultLockedError(error: unknown): boolean {
  return String(error).includes("Vault is locked");
}

/** Whether an IPC failure indicates an invalid master passphrase. */
export function isInvalidMasterKeyError(error: unknown): boolean {
  return String(error).includes("Invalid master key");
}

/** Whether an IPC failure indicates the target entry was not found. */
export function isEntryNotFoundError(error: unknown): boolean {
  return String(error).includes("not found");
}
