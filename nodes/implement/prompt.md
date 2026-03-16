Goal: # Style SVG output from `fabro graph` and API

## Context

SVGs from `fabro graph` use raw Graphviz defaults (black strokes, white bg, Times-serif). The docs SVGs look much better: teal nodes, gray edges, Helvetica font, transparent bg, dark mode CSS. The goal is to make `fabro graph` and the API `get_graph` endpoint produce the same styled output without building a layout engine.

## Approach: DOT defaults + SVG post-processing

### Step 1: `inject_dot_style_defaults(source: &str) -> String` in `graph.rs`

Find first `{`, insert after it:
```dot
    bgcolor="transparent"
    node [color="#357f9e", fontname="Helvetica", fontsize=12, fontcolor="#1a1a1a"]
    edge [color="#666666", fontname="Helvetica", fontsize=10, fontcolor="#666666"]
```
These are DOT defaults — per-node/edge attrs override them. Applies to both SVG and PNG.

### Step 2: `postprocess_svg(raw: Vec<u8>) -> Vec<u8>` in `graph.rs`

SVG-only. Two string operations:
1. Remove line containing `<polygon fill="white" stroke="none"` (the white background)
2. Insert dark mode `<style>` block after the `<svg ...>` closing `>`

### Step 3: Wire into `render_dot()` in `graph.rs`

- Call `inject_dot_style_defaults` on source before piping to `dot`
- Call `postprocess_svg` on output when format is SVG

### Step 4: Wire into `get_graph()` in `server.rs` (line ~1440, ~1467, ~1472)

- Wrap `dot_source` with `inject_dot_style_defaults`
- Wrap `output.stdout` with `postprocess_svg`

### Step 5: Wire into `get_run_graph()` in `demo/mod.rs` (line ~227, ~248, ~252)

Same two wraps.

### Step 6: Tests in `graph.rs`

- `inject_dot_style_defaults` inserts expected attrs
- `inject_dot_style_defaults` returns unchanged if no `{`
- `postprocess_svg` removes white background
- `postprocess_svg` injects dark mode style block
- Update `graph_valid_workflow_svg` to assert styled output

## Files to modify

1. `lib/crates/fabro-workflows/src/cli/graph.rs` — add 2 pub fns, modify `render_dot`, add tests
2. `lib/crates/fabro-api/src/server.rs` — 2-line change in `get_graph`
3. `lib/crates/fabro-api/src/demo/mod.rs` — 2-line change in `get_run_graph`

## Verification

1. `cargo test -p fabro-workflows -- graph` — unit tests pass
2. `cargo test -p fabro-api` — API tests pass
3. `cargo clippy --workspace -- -D warnings` — no warnings
4. `fabro graph fabro/workflows/implement/workflow.fabro -o /tmp/test.svg` — visually inspect:
   - Teal node strokes, gray edges, Helvetica font
   - No white background
   - Dark mode `<style>` block present
   - Compare with `docs/images/tutorial-plan-implement.svg`


## Completed stages
- **toolchain**: success
  - Script: `command -v cargo >/dev/null || { curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && sudo ln -sf $HOME/.cargo/bin/* /usr/local/bin/; }; cargo --version 2>&1`
  - Stdout:
    ```
    cargo 1.94.0 (85eff7c80 2026-01-15)
    ```
  - Stderr: (empty)
- **preflight_compile**: success
  - Script: `cargo check -q --workspace 2>&1`
  - Stdout: (empty)
  - Stderr: (empty)
- **preflight_lint**: success
  - Script: `cargo clippy -q --workspace -- -D warnings 2>&1`
  - Stdout: (empty)
  - Stderr: (empty)


Read the plan file referenced in the goal and implement every step. Make all the code changes described in the plan. Use red/green TDD.