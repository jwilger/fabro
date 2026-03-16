Perfect! The fix is complete and minimal. Let me summarize what was done:

## Summary

**Problem**: The setup stage was failing when trying to execute a `git clone` command into a directory that already has files, resulting in:
```
fatal: destination path '.' already exists and is not an empty directory.
```

**Solution**: Added documentation guidance in the "Run Configuration" page about handling idempotent git clones. The new subsection "Idempotent git clones" under the `[setup]` section demonstrates the correct pattern using shell fallback operators (`||`) to handle the case where a clone directory already exists.

**Changes Made**:
- Modified: `docs/execution/run-configuration.mdx`
- Added: A new subsection "Idempotent git clones" with explanation and example
- The example shows how to use `git clone ... || (cd ... && git pull)` pattern to gracefully handle existing directories

This minimal code change addresses the GitHub issue by:
1. Preventing users from encountering the same error
2. Showing the recommended pattern used elsewhere in the documentation (semantic-port example)
3. Directly mentioning the error message users might encounter to help them find the solution