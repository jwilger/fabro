All tests pass across the entire workspace — zero failures. Here's a summary of all the changes made:

## Summary of Changes

### 1. `Cargo.toml` (workspace root) — Added dependencies
- `sentry = { version = "0.35", default-features = false, features = ["backtrace", "contexts", "ureq", "rustls"] }`
- `fork = "0.2"`
- `exec = "0.3"`

### 2. `lib/crates/fabro-util/Cargo.toml` — Added dependencies
- `sentry.workspace = true`
- `fork.workspace = true`
- `exec.workspace = true`

### 3. `lib/crates/fabro-util/src/telemetry/mod.rs` — Registered new modules
- Added `pub mod panic;`
- Added `pub mod spawn;`

### 4. `lib/crates/fabro-util/src/telemetry/spawn.rs` — **NEW FILE**
Shared `spawn_detached()` function with:
- **Unix**: double-fork pattern (`fork → setsid → close_fd → fork → exec`) via the `fork` and `exec` crates
- **Windows**: `Command::new().creation_flags(DETACHED_PROCESS).spawn()`
- Helper functions: `write_temp_json()`, `build_fabro_argv()`, `current_exe_str()`, `path_exists()`
- **5 tests**: temp file creation, argv building, current exe, empty args no-op, Unix integration test with marker file

### 5. `lib/crates/fabro-util/src/telemetry/sender.rs` — Refactored to use `spawn_detached`
- Replaced `Command::new().spawn()` with `super::spawn::spawn_detached()` and `super::spawn::write_temp_json()`
- Simplified `spawn_sender()` to return `()` (no longer `Result`)
- Existing `send_noops_without_write_key` test continues to pass

### 6. `lib/crates/fabro-util/src/telemetry/panic.rs` — **NEW FILE**
Core panic reporting logic:
- `install_panic_hook()` — chains onto default hook, calls `report_panic()` first
- `build_panic_event()` — builds `sentry::protocol::Event` with exception (type=`"panic"`, mechanism `handled=false`), stacktrace, OS context, Fatal level, release version
- `report_panic()` — checks telemetry level (Off → skip), filters "Broken pipe" panics, builds event, spawns sender
- `spawn_panic_sender()` — serializes event to `~/.fabro/tmp/fabro-panic-{id}.json`, double-forks `fabro __send_panic <path>` with `FABRO_TELEMETRY=off`
- `send_panic_to_sentry()` — reads JSON, inits Sentry client with compile-time `SENTRY_DSN`, captures event, flushes
- **5 tests**: event structure validation, broken pipe filtering, telemetry-off no-op, DSN-missing no-op, JSON round-trip

### 7. `lib/crates/fabro-cli/src/main.rs` — Wired up
- `install_panic_hook()` called at the very start of `main()` (before `main_inner()`)
- Added `SendPanic` hidden subcommand variant (mirrors `SendAnalytics`)
- Added command name mapping `"__send_panic"`
- Added handler that reads the JSON, calls `send_panic_to_sentry()`, and cleans up the temp file