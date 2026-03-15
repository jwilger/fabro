All verification steps pass. Let me provide a summary of everything that was implemented:

## Summary

All steps from the plan have been fully implemented. Here's what's in place:

### Step 1: `lib/crates/fabro-beastie/` crate (6 files)

| File | Description |
|------|-------------|
| `Cargo.toml` | Crate manifest with platform-conditional deps (`core-foundation` on macOS, `libc` on Linux) |
| `src/lib.rs` | Public API: `guard(enabled: bool) -> Option<SleepInhibitorGuard>` with `cfg`-based platform dispatch + 3 unit tests |
| `src/dummy.rs` | No-op backend for unsupported platforms |
| `src/macos.rs` | IOKit `IOPMAssertionCreateWithName("PreventUserIdleSystemSleep")` on acquire, `IOPMAssertionRelease` on drop |
| `src/iokit_bindings.rs` | Raw FFI bindings for IOKit power management |
| `src/linux.rs` | Spawns `systemd-inhibit` (fallback `gnome-session-inhibit`), kills child on drop, uses `PR_SET_PDEATHSIG` |

### Step 2: Config field in `fabro-config`
- `lib/crates/fabro-config/src/cli.rs`: Added `#[serde(default)] pub prevent_idle_sleep: bool` to `CliConfig` + 2 tests (`parse_prevent_idle_sleep_true`, `parse_prevent_idle_sleep_defaults_to_false`)

### Step 3: Wiring into `fabro-cli` and `fabro-workflows`

| File | Change |
|------|--------|
| `fabro-cli/Cargo.toml` | Feature `sleep_inhibitor = ["dep:fabro-beastie", "fabro-workflows/sleep_inhibitor"]` + optional dep |
| `fabro-cli/src/main.rs` | `#[cfg(feature = "sleep_inhibitor")] let _sleep_guard` in `Exec` path; passes `prevent_idle_sleep` to `run_command` |
| `fabro-workflows/Cargo.toml` | Feature `sleep_inhibitor = ["dep:fabro-beastie"]` + optional dep |
| `fabro-workflows/src/cli/run.rs` | `run_command` accepts `prevent_idle_sleep: bool`, `#[cfg(feature = "sleep_inhibitor")] let _sleep_guard` at top |

### Verification results
1. ✅ `cargo test -p fabro-beastie` — 3 tests pass
2. ✅ `cargo test -p fabro-config` — 60 tests pass
3. ✅ `cargo build --workspace` — compiles without the feature
4. ✅ `cargo build --workspace --features fabro-cli/sleep_inhibitor` — compiles with the feature
5. ✅ `cargo clippy --workspace -- -D warnings` — clean (both with and without feature)
6. ✅ `cargo fmt --check --all` — clean