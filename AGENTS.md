# AGENTS.md

PassHarbor is a local-first password manager built with **Tauri 2** (Rust backend) and **SvelteKit** (Svelte 5 + TypeScript frontend). The project is in early development: prioritize **speed of iteration, clean structure, and readability** over completeness.

## Commands

| Task | Command |
| --- | --- |
| Dev app (frontend + Rust) | `npm run tauri dev` |
| Frontend only | `npm run dev` |
| Frontend type check | `npm run check` |
| Build | `npm run tauri build` |
| Rust checks | `cargo check` / `cargo clippy` (run in `src-tauri/`) |
| Rust tests | `cargo test` (run in `src-tauri/`) |

The Rust side has unit tests next to the code they cover. There is no frontend test setup yet. Before finishing a task: `cargo check` + `cargo test` (backend changes), `npm run check` (frontend changes). Run all of them when in doubt.

## Architecture

```
src/                     SvelteKit frontend (SPA, ssr = false)
  lib/api.ts             Typed wrappers around the IPC commands
  routes/                Svelte routes; +layout.ts disables SSR for Tauri
src-tauri/src/
  main.rs                Binary entry point; calls passharbor_lib::run()
  lib.rs                 Tauri builder, command registration, state setup
  frontend_api.rs        All #[tauri::command] functions — the only IPC surface
  master_key.rs          MasterKey + KdfParams: Argon2id passphrase stretching,
                         HKDF-SHA256 expansion
  state.rs               AppState: the open session and the vault store
  error.rs               Error enum
  prelude.rs             Shared Result<T>/IPCResult<T> aliases, Error re-export
  database/
    mod.rs               Database: SQLCipher connection, entry queries, file helpers
    key.rs               DbKey: the per-vault database key
    schema.rs            Entry/Password types, migrations, input validation
    vault_store.rs       VaultStore: manifests, vault selection, last-opened pointer
```

### Storage layout

Each vault lives in `<app data>/vaults/` as a pair of files:

- `<id>.db` — the SQLCipher-encrypted database (`id` is 16 lowercase hex chars).
- `<id>.json` — the plaintext manifest: `id`, `name`, `salt`, `kdf` (Argon2id
  parameters), `created_at`. Manifests are not secret, but they are still
  written owner-only.

`<app data>/vaults/last_opened` records the id of the vault that was opened
last. On unlock the app opens that vault; failing that the oldest vault by
`created_at`; failing that a new vault is created with the passphrase that was
entered.

The database is encrypted by SQLCipher alone — there is no second
application-level cipher. The chain is: passphrase → Argon2id (`KdfParams`) →
`MasterKey` → HKDF-SHA256 (`b"db key"`) → `DbKey` → `PRAGMA key`.
`PRAGMA key` must be the first statement on a vault connection.

### Backend conventions (follow these exactly)

- **Module boundaries**: each concern lives in one module with a minimal `pub` interface. Modules communicate only through explicit function/struct interfaces — no hidden global state. Shared state goes in `AppState` (defined in `state.rs`) and is accessed exclusively via `AppState::resolve(&app)`.
- **State**: `AppState` holds the open session as `Mutex<Option<Database>>` — `None` means the vault is locked — plus the `VaultStore`. Do not add module-level `static`/`lazy_static` state.
- **Commands**: declared in `frontend_api.rs`, registered in `lib.rs` via `tauri::generate_handler![]`. Commands take `app: tauri::AppHandle` when they need state, are marked `#[tauri::command(async)]` so they never block the UI thread, and return `IPCResult<T>` (errors as `String`). Failures are returned to the caller, not emitted as events.
- **Errors**: internal code returns `Result<T>` (`prelude` alias over `error::Error`). Convert to strings only at the IPC boundary. Add new failure modes as variants on `Error` in `error.rs`.
- **Secrets**: key material and passwords live in `secrecy` types — `SecretBox<[u8; 32]>` inside `MasterKey`/`DbKey`, `SecretString` for passphrases and passwords. These redact themselves in `Debug`/`Display` and wipe themselves on drop, so a secret can never be printed by accident. Access the value through `ExposeSecret::expose_secret()` and keep that call close to the use. `secrecy` deliberately refuses to serialize a secret: a field that has to reach the frontend needs an explicit `serialize_with` that exposes it. Derived keys come from `MasterKey::derive_with_salt` using HKDF-SHA256 with a distinct `info` label per key (currently `b"db key"`). Never log, clone unnecessarily, or store key bytes outside these types.
- **Files**: vault files are created owner-only (`0700` directories, `0600` files) and written atomically — write to a temp file, fsync it, rename it into place, fsync the directory.
- **Imports**: modules use `use crate::prelude::*;` for `Error`/`Result`, plus explicit imports for everything else. Note that the `Result` alias shadows `std::result::Result`: spell out `std::result::Result` where two type parameters are needed.
- Edition 2024; crypto crates already available: `hkdf`, `sha2`, `argon2`, `zeroize`, `secrecy`, `rusqlite` (SQLCipher), `serde`, `thiserror`.

### Frontend conventions

- Svelte 5 runes (`$state`, `$derived`, `$effect`); TypeScript; no SSR (`+layout.ts` sets `ssr = false`).
- Talk to the backend only through the typed wrappers in `src/lib/api.ts`, which call `invoke("command_name", {...})` from `@tauri-apps/api/core`. Mirror command names and arg shapes exactly; keep any shared types in sync manually for now.
- Command arguments are camelCase on the JavaScript side (Tauri converts Rust's `snake_case`): `entry_id` is passed as `entryId`. Command *payloads* keep their Rust field names (`created_at`, `current_password`, ...).
- Component styles are colocated in `<style>` blocks.

## Working rules

- **Simplest solution first**: the minimal change that fully solves the problem. No speculative abstraction, no extra layers, no premature config. Prefer linear, readable code over clever code.
- **No hidden bugs**: crypto/security code must be obviously correct — correct nonces, no key duplication, explicit error handling. If a simple approach would compromise correctness or security, stop and ask.
- **Dead code is acceptable right now**; don't spend time removing it unless it blocks the task.
- **Match existing style**: naming, formatting, module layout, and the conventions above take precedence over personal preference. Do not introduce new dependencies, patterns, or directory structures without asking.
- **Ask, don't assume**: before making any non-obvious decision — new dependency, architecture change, ambiguous requirement, conflicting instructions — ask the user. Surface confusion early rather than guessing.
- **Include no conversational context in generated code**: the generated code must not inlucde context from the user and model such as " // The changed F() function with the features you asked" and such.
