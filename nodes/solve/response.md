The fix is complete. Here's a summary:

## Fix for Django #32134 — Crash of `__range` lookup with namedtuple

**Problem:** `resolve_lookup_value` in `django/db/models/sql/query.py` attempts to reconstruct a tuple/list value by calling `type(value)(generator)`. When `value` is a `namedtuple`, this crashes because namedtuples expect individual positional arguments, not a single iterable argument.

**Root cause:** The code `type(value)(generator_expression)` works for regular `tuple` and `list`, but `namedtuple` subclasses require `type(value)(*args)` instead.

**Fix:** In the `resolve_lookup_value` method, check if the value's type has a `_make` attribute (the standard indicator of a namedtuple). If so, use `type_(*values)` to unpack the resolved values as positional arguments. Otherwise, use the original `type_(values)` approach for regular lists and tuples.

**File changed:** `django/db/models/sql/query.py` (lines 1077-1087)