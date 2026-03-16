All tests pass. Here's a summary of what was fixed:

## Summary of Fixes

### Critical Bug Fix
1. **Stale `FinalizeConfig` for remote sandboxes** — `FinalizeConfig` was being built *before* `run_with_lifecycle()` consumed the `RunConfig`, but the engine mutates `config.run_branch`, `config.base_sha`, `config.base_branch`, `config.meta_branch`, and `config.git_checkpoint_enabled` during remote git setup. This meant PR creation, finalize commits, and checkpoint tracking silently broke for remote sandbox runs (Daytona, SSH, exe.dev).
   - **Fix**: Changed `run_with_lifecycle()` to return `(Outcome, RunConfig)` so the caller gets the mutated config back. Moved `FinalizeConfig` construction to *after* the engine call. Added `FinalizeConfig::from_run_config()` and `FinalizeConfig::from_run_config_owned()` constructors to centralize the conversion (eliminating duplicate field-by-field construction in both `run_command` and `run_from_branch`).

### Minor Cleanups
2. **Missing trailing newline in `hooks.mdx`** — restored the standard trailing newline.
3. **Unnecessary `setup_commands.clone()`** — the variable was never used after being passed to `LifecycleConfig`, so replaced `.clone()` with a direct move.
4. **Verbose `sandbox_info_opt` chain** — replaced `.map(|sb| { ... }).unwrap_or(None)` with the idiomatic `.and_then(|sb| { ... })`.

### Findings Not Fixed (false positives or not worth addressing)
- The `deferred_sandbox: Arc<Mutex<Option<Arc<dyn Sandbox>>>>` triple-wrapping is necessary — each layer serves a distinct purpose (trait object sharing, deferred initialization, synchronous closure mutability, cross-closure sharing).
- The devcontainer command vector clones are necessary since `devcontainer_config` is borrowed earlier for environment extraction.
- The `SandboxProvider` → `&str` match doesn't have an existing `Display`/`as_str()` impl on the enum, and adding one would be scope creep.
- The stringly-typed `provider_name == "docker"` comparisons in the event listener are a pre-existing pattern from the original code; not worth refactoring in this PR.