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

There is no test suite yet. Before finishing a task: `cargo check` (backend changes), `npm run check` (frontend changes). Run both when in doubt.

## Architecture

```
src/                     SvelteKit frontend (SPA, ssr = false)
  routes/                Svelte routes; +layout.ts disables SSR for Tauri
src-tauri/src/
  main.rs                Binary entry point; calls passharbor_lib::run()
  lib.rs                 Tauri builder, AppState definition + management,
                         command registration
  frontend_api.rs        All #[tauri::command] functions — the only IPC surface
  master_key.rs          MasterKey: holds user key, HKDF-SHA256 derivation
  encryptor.rs           EncKey + ChaCha20-Poly1305 encrypt/decrypt
  database.rs            DbKey and (future) SQLite persistence
  vault.rs               (stub) VaultState
  error.rs               Error enum, emit_error event helper
  prelude.rs             Shared Result<T>/IPCResult<T> aliases, Error re-export
  utils.rs               resolve_app_state helper
```

### Backend conventions (follow these exactly)

- **Module boundaries**: each concern lives in one module with a minimal `pub` interface. Modules communicate only through explicit function/struct interfaces — no hidden global state. Shared state goes in `AppState` (defined in `lib.rs`) and is accessed exclusively via `resolve_app_state(&app)`.
- **State**: `AppState` holds `Mutex<T>` fields; keys are stored as `Option<...>` and are `None` when unset. Do not add module-level `static`/`lazy_static` state.
- **Commands**: declared in `frontend_api.rs`, registered in `lib.rs` via `tauri::generate_handler![]`. Commands take `app: tauri::AppHandle` when they need state, return `IPCResult<T>` (errors as `String`), and use `emit_error(&app, &Error)` to surface failures as `"error"` events to the frontend.
- **Errors**: internal code returns `Result<T>` (`prelude` alias over `error::Error`). Convert to strings only at the IPC boundary. Add new failure modes as variants on `Error` in `error.rs`.
- **Secrets**: any key material is a newtype over `[u8; 32]` (or wraps one) deriving `Zeroize, ZeroizeOnDrop`. Derived keys come from `MasterKey::derive_*` using HKDF-SHA256 with a distinct `info` label per key (e.g. `b"db key"`, `b"enc key"`). Never log, clone unnecessarily, or store key bytes outside these types.
- **Imports**: modules use `use crate::prelude::*;` for `Error`/`Result`, plus explicit imports for everything else.
- Edition 2024; crypto crates already available: `chacha20poly1305`, `hkdf`, `sha2`, `argon2`, `zeroize`, `secrecy`, `rusqlite`, `serde`, `thiserror`, `anyhow`.

### Frontend conventions

- Svelte 5 runes (`$state`, `$derived`, `$effect`); TypeScript; no SSR (`+layout.ts` sets `ssr = false`).
- Talk to the backend only via `invoke("command_name", {...})` from `@tauri-apps/api/core` and listen for `"error"` events. Mirror command names/arg shapes exactly; keep any shared types in sync manually for now.
- Component styles are colocated in `<style>` blocks.

## Working rules

- **Simplest solution first**: the minimal change that fully solves the problem. No speculative abstraction, no extra layers, no premature config. Prefer linear, readable code over clever code.
- **No hidden bugs**: crypto/security code must be obviously correct — correct nonces, no key duplication, explicit error handling. If a simple approach would compromise correctness or security, stop and ask.
- **Dead code is acceptable right now** (e.g. unused stubs like `vault.rs`); don't spend time removing it unless it blocks the task.
- **Match existing style**: naming, formatting, module layout, and the conventions above take precedence over personal preference. Do not introduce new dependencies, patterns, or directory structures without asking.
- **Ask, don't assume**: before making any non-obvious decision — new dependency, architecture change, ambiguous requirement, conflicting instructions — ask the user. Surface confusion early rather than guessing.
- **Include no conversational context in generated code**: the generated code must not inlucde context from the user and model such as " // The changed F() function with the features you asked" and such.
