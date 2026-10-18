import { invoke } from "@tauri-apps/api/core";

export async function unlock(passphrase: string): Promise<void> {
  await invoke("unlock_cmd", { passphrase });
}
