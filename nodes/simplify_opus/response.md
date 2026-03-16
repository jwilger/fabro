All changes are clean. Here's a summary of what was fixed:

### Fixes Applied

1. **Regex now uses `static LazyLock<Regex>`** — Matches the established codebase convention (used in `fabro-cli/src/doctor.rs`, `fabro-workflows/src/error.rs`, `fabro-util/src/redact/entropy.rs`, etc.). The regex is compiled once instead of on every call.

2. **`apply_direction` returns `Cow<'_, str>` instead of `String`** — Avoids an unnecessary heap allocation when `direction` is `None` (the common case). The `Regex::replace_all` already returns `Cow<str>`, so the `Some` branch also avoids an extra `.into_owned()` when there are no matches.

3. **Replacement string uses static `&str` match** — Instead of `format!("rankdir={dir}")`, uses a direct match to static strings `"rankdir=LR"` / `"rankdir=TB"`, eliminating a small heap allocation.

4. **Restored trailing newline in `cli.mdx`** — The previous commit accidentally stripped the final newline (POSIX text file convention).

5. **Fixed misleading doc** — Changed from `"lr (default) or tb"` to `"lr or tb. If omitted, uses the DOT file's own rankdir."` — the flag is optional with no default; when omitted, the DOT file's existing `rankdir` is preserved as-is.