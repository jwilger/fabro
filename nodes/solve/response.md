Implemented the Django fix for `sqlmigrate` wrapping output in `BEGIN/COMMIT` on databases without transactional DDL.

What changed:
- In `django/core/management/commands/sqlmigrate.py`
  - `self.output_transaction` now requires both:
    - `migration.atomic`
    - `connection.features.can_rollback_ddl`

- In `tests/migrations/test_commands.py`
  - Added a regression test:
    - `test_sqlmigrate_for_non_transactional_databases`
    - mocks `connection.features.can_rollback_ddl = False`
    - asserts no transaction wrappers are emitted
  - Updated the existing forward/backward `sqlmigrate` assertions so they only require `BEGIN/COMMIT` on backends that support transactional DDL.

Validation:
- Ran:
  - `python -m pip install -e .`
  - `python tests/runtests.py migrations.test_commands --parallel 1 --verbosity 2`

Result:
- All tests in `migrations.test_commands` passed.