The code is clean. Here's a summary of the review:

**Fixed:**
- **Long format string in `inject_dot_style_defaults`** (graph.rs line 100): Extracted the inline DOT defaults into a `DOT_STYLE_DEFAULTS` const using a `r##"..."##` raw string literal, matching the style of the existing `DARK_MODE_STYLE` const. The `format!` call is now a clean `format!("{before}{DOT_STYLE_DEFAULTS}{after}")` instead of a 200+ character single line with embedded `\n` escapes.

**Already clean (no action needed):**
- **Code reuse**: The `render_dot_svg` helper in `server.rs` properly deduplicates the inline dot rendering that was previously copy-pasted in both `server.rs` and `demo/mod.rs`. No other SVG/DOT utilities exist in the codebase.
- **Color coupling**: `#357f9e` and `#666666` appear in both DOT defaults and dark mode CSS selectors — this is by design (CSS targets the DOT-generated attributes).
- **Efficiency**: The `postprocess_svg` string allocations are negligible for SVG sizes (a few KB). The `spawn_blocking` approach is appropriate since `render_dot` does synchronous I/O + string processing.
- **No copy-paste, no parameter sprawl, no leaky abstractions** — the change is well-structured.