The fix is minimal and clean. Here's a summary of what was changed:

**File: `astropy/io/ascii/rst.py`**

Two changes in the `RST` class:

1. **`__init__`**: Added `header_rows=None` parameter and passed it through to the parent `FixedWidth.__init__()`. This allows the RST writer to accept the `header_rows` keyword argument (e.g., `header_rows=["name", "unit"]`).

2. **`write`**: Changed the hardcoded `lines[1]` index to `lines[idx]` where `idx = len(self.header.header_rows)`. Previously it assumed the `===` separator line was always at index 1 (valid only for the default single header row `["name"]`). With multiple header rows, the separator line shifts to a later index — it's always right after all header rows, so its index equals the number of header rows.