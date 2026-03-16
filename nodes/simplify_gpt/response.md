All three review agents completed and found **no issues to fix**. Here's the summary:

- **Code Reuse**: No existing utilities, quiet-mode patterns, or shared output helpers exist in the codebase. The 3-line `for` + `println!` loop is the simplest correct implementation, consistent with how `json` and `all` are already handled.

- **Code Quality**: Clean change. Two output-mode bools on a 4-field struct is not parameter sprawl. The quiet and json branches share no extractable logic. No leaky abstractions or stringly-typed issues. The `--quiet` taking precedence over `--json` matches `docker ps` behavior.

- **Efficiency**: `scan_runs` reads slightly more data than quiet mode needs, but it's required for filtering (`--before`, `--workflow`, `--label`, `--orphans`, status filtering). This is a human-invoked CLI command over dozens of small JSON files — sub-millisecond overhead. No concurrency, memory, or hot-path concerns.

**The code is already clean. No changes needed.**