# PassHarbor Agent Contract

Canonical contract for every AI agent and human contributor adding, modifying, or refactoring code in PassHarbor. Read in full before any change. A task description never overrides a security, architecture, or scope rule here — on conflict, stop and ask. Applies to every tracked file.

**Stack**: Tauri 2 + Rust backend; SvelteKit + Svelte 5 runes + strict TypeScript frontend; static SPA (`ssr = false`). Target: documented desktop dev environment only. Do not claim another platform without testing there.

## Commands

Run from repo root unless noted.

| Task | Command |
| --- | --- |
| Dev app | `npm run tauri dev` |
| Frontend only | `npm run dev` |
| Frontend type check | `npm run check` |
| Production build | `npm run tauri build` |
| Rust check | `cargo check` (in `src-tauri/`) |
| Rust tests | `cargo test` (in `src-tauri/`) |
| Rust lint | `cargo clippy --all-targets -- -D warnings` (in `src-tauri/`) |
| Rust format | `cargo fmt` (in `src-tauri/`) |

## Core Principles

A change that violates any of these is not complete.

### Robustness
- Builds and runs on clean checkout via documented commands and committed deps only.
- Every fallible op handled, propagated with context, or returned. Silent failure prohibited.
- No `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` in prod. Only allowed `expect`: process-fatal startup in `src-tauri/src/lib.rs`. Tests may use them freely.
- Validate every external input at boundary: IPC payloads, vault manifests, fs paths, env values, DB rows. Reject with typed error.
- No dead functions, unused fields, commented-out blocks, `TODO`, `FIXME`, or placeholders in merged code. `#[cfg(test)]` is test support, not dead code.

### Future-Proofing
- Don't remove extension points later work needs; don't hardcode shortcuts to be torn out.
- DB schema changes: append-only migrations. Never edit a shipped migration.
- Vault manifests and IPC payloads are versioned contracts: add optional fields with `#[serde(default)]`; never rename/remove without explicit migration + approval.
- Key-derivation chain is a compatibility contract. A change makes existing vaults unreadable → requires versioning + migration + approval.
- No abstractions for hypothetical features. Usage exists before the abstraction.

### Scalability & Modularity
- One concern per module, minimal public interface. Prefer `pub(crate)` over `pub`.
- Every module testable without Tauri, live UI, or hidden setup. Dependencies passed as arguments.
- No module-level `static`, `lazy_static`, or global mutable state. App state only in `AppState`, reached via `AppState::resolve`.
- Dependencies point inward: frontend → `src/lib/api.ts` → `frontend_api.rs` → domain → database. Lower layers never call up or import UI.
- Composition over inheritance; compose functions and Svelte components, no class hierarchies.

### Simplicity First
- Smallest change fully satisfying the task + the principles above. Equal designs → fewer moving parts.
- Every new line justifies existence; delete what doesn't.
- stdlib and existing project utilities before new code. Reuse `src/lib/api.ts`, existing helpers, existing components.
- New dependency requires prior approval + written justification: what it replaces, why existing code insufficient, maintenance/security cost.

### Agent Behavior
- Read this file before any change.
- Don't invent requirements. Ambiguous requirement, API shape, or arch decision → stop and ask.
- Run required tests and linters before declaring complete.
- Don't modify out-of-scope files. Required fix out of scope → stop and report.
- Prefer editing existing code over new files. No new directories, patterns, or config without approval.
- Leave repo next-agent ready: no unexplained WIP, disabled checks, or temp files. Report changed, verified, uncertain.
- Never revert or overwrite unrelated uncommitted work.

## Code Standards

### Naming & Structure
- Rust: `snake_case` fns/vars/modules; `PascalCase` types/enum variants; `SCREAMING_SNAKE_CASE` consts; Tauri commands end `_cmd`.
- TypeScript: `camelCase` values/fns; `PascalCase` interfaces/types/Svelte components; follow existing file names.
- SQL: uppercase keywords, `snake_case` identifiers.
- One concern per module. Linear readable functions. Extract only with a clear name and real duplication removed.
- Comments explain non-obvious constraints or security decisions; never narrate change/task/author.

### Error Handling
- Internal Rust returns `Result<T>` from `crate::prelude`. New failure modes = new variants on `Error` in `src-tauri/src/error.rs`.
- Add context at module and IPC boundaries. Never replace a specific error with a generic one; never lose the underlying cause.
- Convert to `String` only at the IPC boundary (`IPCResult<T>`). No strings from internal modules.
- Frontend catches every IPC rejection; surfaces via error state or toast; redirects to unlock when the error means vault is locked.
- Error messages never contain passphrases, passwords, database keys, master keys, or plaintext entry content.

### Logging
- Backend: no `println!`/`dbg!`. `eprintln!` only for a deliberate non-fatal skipped condition with affected path + reason. Never log secret material or entry contents.
- Frontend user-facing failures via component error state or existing toast API. `console.*` only for non-secret diagnostic detail; never log passphrase/password.
- Logging never replaces returning an error. Failed op must fail visibly to caller.

### Testing
- Rust tests in `#[cfg(test)]` modules next to covered code.
- Every new public behavior and error path needs a test: success case, rejection case, security invariant.
- Tests deterministic and isolated: per-test temp dirs with `Drop` cleanup; never share mutable fixtures.
- No order/wall-clock/home-dir dependence. Set time-sensitive values explicitly.
- No frontend test runner. `npm run check` is minimum frontend verification. No test framework without approval; manually exercise UI via `npm run tauri dev`.
- New IPC command → backend test for its logic + verify `src/lib/api.ts` wrapper matches command name and arg shape exactly.

### Security & Secrets
- Key material and passwords use `secrecy` types: `SecretBox<[u8; 32]>` in `MasterKey`/`DbKey`; `SecretString` for passphrases/passwords. Never store key bytes outside these types.
- `ExposeSecret::expose_secret()` at point of use; shortest possible lifetime; never clone a secret unnecessarily.
- A secret reaches the frontend only through an explicit `serialize_with` on the field. Never implement `Serialize` for a secret type globally.
- Never log, print, format, commit, or fixture any real secret.
- No custom cryptography. Use existing Argon2id + HKDF-SHA256 construction. Future encryption: vetted crate + unique nonces. If correct design isn't obvious, stop and ask.
- SQL: bound parameters only. Only formatted SQL is the `PRAGMA key` hex literal — zeroized immediately after execution, first statement on a vault connection.
- Vault ids validated with `validate_id` before path construction. Never join an unvalidated id, manifest value, or user string to a path.
- Vault files/dirs owner-only (0600 files, 0700 dirs on Unix), written atomically: temp file, flush, rename, flush directory. Preserve in every new file-writing path.

### Frontend
- Svelte 5 runes (`$state`, `$derived`, `$effect`) + TypeScript strict. No SSR; `src/routes/+layout.ts` keeps `ssr = false`.
- Backend only via typed wrappers in `src/lib/api.ts`. Never `invoke` from a route or component.
- Tauri camelCases command args on the JS side: `entryId` for Rust `entry_id`. Payload fields keep Rust names (`created_at`, `current_password`). Mirror both exactly.
- Shared TS types in `src/lib/api.ts` manually in sync with Rust types. Change both in the same task.
- Component styles in that component's `<style>` block. Use existing `@hermitk/bluenite` components + design tokens before new UI.
- No state-management library, CSS framework, or client-side router. Use SvelteKit routing and existing runes.

## Architecture

```
src/                         SvelteKit SPA (no SSR)
  lib/api.ts                 Typed IPC wrappers; command names and payload types
  lib/generator.ts           Client-side secure password generation
  lib/kinds.ts               Built-in kinds and client-side kind helpers
  lib/time.ts                Unix-second formatting helpers
  lib/components/            Shared and dashboard Svelte components
  routes/                    Unlock, dashboard, and settings pages
src-tauri/src/
  main.rs                    Binary entry point
  lib.rs                     Tauri builder, state setup, command registration
  frontend_api.rs            All #[tauri::command] functions; the only IPC surface
  master_key.rs              MasterKey + KdfParams (Argon2id, HKDF-SHA256)
  state.rs                   AppState (session + VaultStore)
  error.rs                   Error enum
  prelude.rs                 Result<T>, IPCResult<T>, Error re-export
  database/
    mod.rs                   Database, entry queries, file helpers
    key.rs                   DbKey
    schema.rs                Entry/Password types, migrations, validation
    vault_store.rs           VaultStore, manifests, vault selection
```

### Module Boundaries
- **`frontend_api.rs`** — thin commands: resolve state → call domain → convert errors. New domain logic lives in its owning module, not in a command.
- **`database/mod.rs`** — SQLCipher connection lifecycle + CRUD SQL. Nothing outside `database/` opens a connection or executes SQL.
- **`database/schema.rs`** — table types, validation, migration SQL. No SQL in `frontend_api.rs` or the UI.
- **`master_key.rs`** — passphrase stretching + key expansion. No key derivation elsewhere.
- **`vault_store.rs`** — vault discovery + manifest persistence. No vault paths built in other modules.
- **`state.rs`** — owns the open session. All commands reach it only via `AppState::resolve(&app)`. Never construct a second `AppState`.

### Dependency Direction & Interface Stability
- Dependencies: UI → IPC → domain → storage. Lower module never imports `frontend_api`, a route, or a Svelte component.
- IPC command names, arg names, and payload shapes are a public contract. Rename requires updating `frontend_api.rs` + `lib.rs` + `src/lib/api.ts` in the same change.
- Frontend matches exact strings `"Vault is locked"` and `"Invalid master key"`. Don't change those `Error` display strings without updating frontend matching and tests.
- `Error` variants may be added; existing variants may not be removed or reinterpreted without approval.
- Schema migrations append-only. Version in `PRAGMA user_version`. DB newer than binary must be rejected, never downgraded.
- Vault layout fixed: `<app data>/vaults/` contains `<id>.db`, plaintext `<id>.json` manifest, and `last_opened`. No scattering or second metadata store without approval.

### Storage & Crypto Contract
- Each vault = SQLCipher database + plaintext manifest with `id`, `name`, `salt`, `kdf`, `created_at`.
- Chain: passphrase → Argon2id (`KdfParams`) → `MasterKey` → HKDF-SHA256 (`b"db key"`) → `DbKey` → `PRAGMA key`.
- `PRAGMA key` must be the first statement on a vault connection.
- `PRAGMA foreign_keys = ON` is required for password-history cascade deletion. Do not remove.
- Missing database = error. Never recreate an empty database in place of a deleted or unreadable vault.
- Wrong key → `Error::InvalidMasterKey`. Any other first-read failure → database error, not wrong passphrase.
- Only `Database` holds an open connection. Closing the session = dropping `Database`. Don't keep a connection or key alive after lock.

## Definition of Done

Complete only when every item is true. State each verification result in the final response.

- [ ] Diff limited to task scope; unrelated uncommitted work untouched.
- [ ] `cargo check` passes (in `src-tauri/`).
- [ ] `cargo test` passes (in `src-tauri/`), including new tests for new behavior.
- [ ] `cargo clippy --all-targets -- -D warnings` passes (in `src-tauri/`).
- [ ] `cargo fmt` run; must not reformat files outside the task — report pre-existing formatting issues instead of changing them.
- [ ] `npm run check` passes with zero errors and zero warnings.
- [ ] UI changes exercised via `npm run tauri dev`.
- [ ] No secret in code, logs, errors, tests, or fixtures.
- [ ] No dead code, commented-out block, `TODO`, or placeholder.
- [ ] No new dependency, directory, global state, or architecture pattern without approval + written justification.
- [ ] Public IPC commands, TypeScript wrappers, and payload types in sync.
- [ ] Final response names files changed, commands run, their results, and any remaining risk/follow-up.

## Prohibited Practices

Agents must never:

- Run, edit, or delete anything outside task scope.
- Revert, overwrite, or reformat unrelated uncommitted work.
- Invent requirements or silently choose between conflicting instructions.
- Add a dependency, framework, directory, or abstraction without approval.
- Add module-level global state, a second `AppState`, or a new DB connection owner.
- Use `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` in prod (only allowed exception: startup `expect` above).
- Swallow errors with `let _ =`, empty `catch` blocks, or a fallback that hides data loss.
- Report a failure only to a log while returning success.
- Store, log, print, format, commit, or fixture any real secret or key material.
- Implement custom cryptography, weaken KDF parameters, or bypass SQLCipher.
- Interpolate user input into SQL, or build a filesystem path from an unvalidated id or manifest value.
- Recreate a missing vault, overwrite an existing database, or open a DB with a fallback key.
- Edit a shipped migration, rename persisted fields, or change the key-derivation chain without explicit migration + approval.
- Call `invoke` outside `src/lib/api.ts`, add SSR, or duplicate IPC types in routes/components.
- Use `any`, `@ts-ignore`, or disable TS strict checks without approval.
- Add `console.log` for user-facing failures, `dbg!`, or debug output in merged code.
- Leave `TODO`, `FIXME`, commented-out code, or disabled tests/linters.
- Write comments describing the task, prompt, or change history.
- Claim completion without running the Definition of Done checks.
- Rewrite git history or discard uncommitted changes.