Everything looks good. Let me provide a summary of all changes made:

## Summary

### Changes made across 3 files:

**1. `lib/crates/fabro-workflows/src/cli/graph.rs`** — Core implementation + tests

- Added `DARK_MODE_STYLE` constant — CSS `<style>` block with `@media (prefers-color-scheme: dark)` that adjusts text fill, stroke colors, and polygon fills for dark mode viewing
- Added `pub fn inject_dot_style_defaults(source: &str) -> String` — Inserts DOT graph-level defaults (`bgcolor="transparent"`, teal node styling with `#357f9e`, gray edge styling with `#666666`, Helvetica font) after the first `{` in DOT source. Returns unchanged if no `{` found.
- Added `pub fn postprocess_svg(raw: Vec<u8>) -> Vec<u8>` — Removes the white background `<polygon>` line and inserts the dark mode `<style>` block after the opening `<svg ...>` tag
- Modified `render_dot()` — Calls `inject_dot_style_defaults` on source before piping to `dot`, and calls `postprocess_svg` on output when format is SVG
- Added 4 new unit tests: `inject_dot_style_defaults_inserts_attrs`, `inject_dot_style_defaults_no_brace`, `postprocess_svg_removes_white_bg`, `postprocess_svg_injects_dark_mode`
- Updated `graph_valid_workflow_svg` test to assert dark mode style block is present and white background is removed

**2. `lib/crates/fabro-api/src/server.rs`** — API `get_graph` endpoint

- Wrapped `dot_source` with `inject_dot_style_defaults` before piping to `dot`
- Wrapped `output.stdout` with `postprocess_svg` in the success response

**3. `lib/crates/fabro-api/src/demo/mod.rs`** — Demo `get_run_graph` endpoint

- Same two wraps: `inject_dot_style_defaults` on the DOT source, `postprocess_svg` on the output

### Verification results:
- ✅ `cargo test -p fabro-workflows -- graph` — 108 tests pass (including 4 new + 1 updated)
- ✅ `cargo test -p fabro-api` — 76 tests pass (62 unit + 14 integration)
- ✅ `cargo clippy --workspace -- -D warnings` — zero warnings