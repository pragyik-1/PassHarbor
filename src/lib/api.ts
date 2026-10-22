import { invoke } from "@tauri-apps/api/core";

/**
 * Typed wrappers around the Rust commands in `src-tauri/src/frontend_api.rs`.
 *
 * Tauri converts command arguments from Rust's `snake_case` to `camelCase`
 * on the JavaScript side, so `entry_id` is passed as `entryId`. Command
 * *payloads* keep their Rust field names.
 */

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

/** Open the vault and return its name. */
export async function unlock(passphrase: string): Promise<string> {
  return await invoke<string>("unlock_cmd", { passphrase });
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

export async function getEntry(id: number): Promise<EntryDetail | null> {
  return await invoke<EntryDetail | null>("get_entry_cmd", { id });
}

export async function updateEntry(id: number, entry: UpdateEntry): Promise<boolean> {
  return await invoke<boolean>("update_entry_cmd", { id, entry });
}

export async function deleteEntry(id: number): Promise<boolean> {
  return await invoke<boolean>("delete_entry_cmd", { id });
}

export async function setPassword(entryId: number, password: string): Promise<void> {
  await invoke<void>("set_password_cmd", { entryId, password });
}
