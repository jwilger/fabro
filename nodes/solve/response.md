Implemented the minimal fix in a local Django checkout at `/home/daytona/workspace/django` since the provided setup script couldn't clone into the non-empty workspace root.

Changed:
- `django/db/models/fields/__init__.py`
  - Updated `DurationField`’s invalid-format message from:
    - `[DD] [HH:[MM:]]ss[.uuuuuu]`
  - to:
    - `[DD] [[HH:]MM:]ss[.uuuuuu]`
- `tests/model_fields/test_durationfield.py`
  - Updated the expected error string accordingly.

Validation:
- `python tests/runtests.py model_fields.test_durationfield.TestValidation`
- `python tests/runtests.py model_fields.test_durationfield`

Both passed.