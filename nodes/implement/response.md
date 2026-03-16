All checks pass cleanly. Here's a summary of all changes made:

### 1. `lib/crates/fabro-workflows/src/cli/graph.rs`
- **Added `GraphDirection` enum** (lines 35–51) with `Lr` and `Tb` variants, deriving `Debug, Clone, Copy, ValueEnum` and implementing `Display` (outputs `"LR"` / `"TB"`)
- **Added `--direction` option** to `GraphArgs` (lines 66–68): `#[arg(short = 'd', long)] pub direction: Option<GraphDirection>`
- **Added `apply_direction` function** (lines 102–112) that uses `regex::Regex` to replace `rankdir\s*=\s*\w+` in DOT source when a direction is provided
- **Wired it into `graph_command`** (line 84): `let source = apply_direction(&source, args.direction);`
- **Updated all 6 existing tests** to include `direction: None` in `GraphArgs` construction
- **Added 2 new unit tests**: `apply_direction_rewrites_rankdir` and `apply_direction_none_preserves_source`

### 2. `docs/reference/cli.mdx` (line 377)
- Added row: `-d, --direction <DIR>` / `Graph direction: \`lr\` (default) or \`tb\``

### Verification
- `cargo test -p fabro-workflows -- graph` — **106 passed**, including both new tests
- `cargo clippy --workspace -- -D warnings` — **clean, no warnings**