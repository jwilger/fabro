Everything passes. Here's a summary of what was fixed:

## Summary

### Fixed Issues

1. **Stringly-typed provider comparison** (`cli/run.rs`): Replaced `provider_name == "docker"` and `provider_name == "ssh"` string comparisons with `provider == SandboxProvider::Docker` and `provider == SandboxProvider::Ssh` enum comparisons. This eliminated the intermediate `sandbox_provider_name` match arm and the `provider_name: String` allocation, using the `Copy`-implementing `SandboxProvider` enum directly in the event listener closure.

2. **Test boilerplate** (`engine.rs`): Extracted `test_run_config(run_dir, run_id)` and `test_lifecycle(setup_commands)` helper functions that build default `RunConfig` and `LifecycleConfig` for tests. The 5 new lifecycle tests each had ~20 lines of identical config construction that now reduce to single-line calls. This removed ~95 lines of duplicated test code.

### Findings Not Fixed (by design)

- **`FinalizeConfig`**: While it copies fields from `RunConfig`, it serves a genuine purpose — the error path needs config values when `run_with_lifecycle` fails and doesn't return a `RunConfig`. Eliminating it would require restructuring the entire post-engine control flow, which would be a larger refactor with no functional benefit.
- **`deferred_sandbox` pattern (`Arc<Mutex<Option<Arc<dyn Sandbox>>>>`**: This is inherently needed because event listeners are registered before the sandbox is created, but need access to it later. The sandbox can't be created earlier (it depends on provider config), and the listeners must be registered before the emitter is wrapped in `Arc`. The pattern is local to this scope and the mutex is only locked once (when the event fires).
- **Existing 43 `RunConfig` constructions in tests**: The new helpers only apply to the 5 new tests. Backfilling all existing tests would be scope creep unrelated to this PR.