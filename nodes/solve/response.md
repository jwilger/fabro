The fix works perfectly. Here's a summary of the minimal change made:

## Changes Made

**File: `astropy/io/ascii/rst.py`** (2 changes in the `RST` class)

1. **`__init__` method**: Added `header_rows=None` parameter and passed it through to the parent `FixedWidth.__init__()`:
   - Before: `def __init__(self):` → `super().__init__(delimiter_pad=None, bookend=False)`
   - After: `def __init__(self, header_rows=None):` → `super().__init__(delimiter_pad=None, bookend=False, header_rows=header_rows)`

2. **`write` method**: Made the position line index dynamic instead of hardcoded to `1`:
   - Before: `lines = [lines[1]] + lines + [lines[1]]`
   - After: `idx = len(self.header.header_rows)` then `lines = [lines[idx]] + lines + [lines[idx]]`

The hardcoded `lines[1]` only worked when there was exactly 1 header row (the "name" row), because the position/separator line (`=====`) was always at index 1. With multiple header rows (e.g., `["name", "unit"]`), the separator line moves to a later index — specifically `len(header_rows)`.