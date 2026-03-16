All 364 query tests pass. The fix is minimal and correct:

**The Problem:** The `ordering_parts` regex `r'(.*)\s(ASC|DESC)(.*)'` uses `.*` which by default doesn't match newline characters. When multiline `RawSQL` expressions are used in `order_by()`, the regex only matches the last line containing ASC/DESC. If two different multiline expressions happen to have the same last line (e.g., `else null end`), they appear identical in the `seen` set, causing the second one to be incorrectly removed as a "duplicate".

**The Fix:** Adding `re.MULTILINE | re.DOTALL` flags to the regex compilation on line 35 of `compiler.py`. The `re.DOTALL` flag makes `.` match any character including newlines, so `(.*)` captures the entire multiline SQL expression before ASC/DESC, correctly distinguishing between different multiline expressions.

**Change made:**
```python
# Before:
self.ordering_parts = re.compile(r'(.*)\s(ASC|DESC)(.*)')

# After:
self.ordering_parts = re.compile(r'(.*)\s(ASC|DESC)(.*)', re.MULTILINE | re.DOTALL)
```