All clean. Here's a summary of what was fixed:

### Changes Made

1. **Extracted duplicated StageRetrying hook code into a helper method** (`engine.rs`): The two identical 14-line blocks that built a `HookContext` for `StageRetrying` and called `run_hooks()` were extracted into a new `stage_retrying_hook()` method. This follows the existing pattern established by `run_failed_hook()` (line 963). Each call site is now a single 2-line method call instead of a 14-line block.

2. **Restored trailing newline in `hooks.mdx`**: The previous commit accidentally stripped the trailing newline from the docs file.

### Issues Reviewed and Deemed Clean

- **parallel.rs hook pattern**: The `if let Some(ref runner) = services.hook_runner` pattern is correct — handlers don't have access to the engine's `run_hooks()` wrapper, so directly calling `runner.run()` is the appropriate approach.
- **Type conversions**: `usize::try_from(u32)` matches the existing convention throughout the file. Not worth deviating.
- **Cloning**: `graph.name.clone()`, `node.id.clone()`, `services.sandbox.clone()` — all are necessary since `HookContext` owns its strings, and sandbox is `Arc`-wrapped (cheap clone).