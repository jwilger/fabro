The fix is complete. Here's a summary of the change:

**File changed:** `django/db/models/fields/__init__.py`

**Change:** In the `DurationField.default_error_messages['invalid']` string, the format description was corrected from:
- `[DD] [HH:[MM:]]ss[.uuuuuu]` (incorrect — implied hours were optional but minutes depended on hours)

to:
- `[DD] [[HH:]MM:]ss[.uuuuuu]` (correct — seconds are mandatory, minutes are optional, and hours are optional only when minutes are provided)

**Rationale:** Looking at the `standard_duration_re` regex in `django/utils/dateparse.py`, the parsing logic is:
1. `(?P<seconds>\d+)` — seconds are **mandatory**
2. `(?:(?P<minutes>\d+):)?` — minutes are **optional** (preceding the seconds)
3. `((?:(?P<hours>\d+):)(?=\d+:\d+))?` — hours are **optional**, but only when followed by both minutes and seconds (the lookahead `(?=\d+:\d+)` enforces this)

So `14:00` parses as 14 minutes and 0 seconds (not 14 hours), which matches the corrected format `[[HH:]MM:]ss`.