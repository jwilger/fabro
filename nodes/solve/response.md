The changes are complete. Here's a summary of what was done:

## Changes Made

### 1. `django/core/management/commands/sqlmigrate.py` (line 59)

**Before:**
```python
self.output_transaction = migration.atomic
```

**After:**
```python
self.output_transaction = migration.atomic and connection.features.can_rollback_ddl
```

This ensures that `sqlmigrate` only wraps output in `BEGIN`/`COMMIT` when **both** conditions are true:
- The migration is atomic (`migration.atomic`)
- The database supports transactional DDL (`connection.features.can_rollback_ddl`)

This matches the behavior of the migration executor itself, which checks both conditions before wrapping migrations in transactions.

### 2. `tests/migrations/test_commands.py` — New test `test_sqlmigrate_for_non_transactional_databases`

Added a new test that uses `mock.patch.object(connection.features, 'can_rollback_ddl', False)` to simulate a database that doesn't support transactional DDL, and verifies that transaction wrappers (`BEGIN`/`COMMIT`) are not included in the `sqlmigrate` output. The test uses the standard atomic `test_migrations` (via `@override_settings`), so the only reason transaction wrappers are omitted is the mocked `can_rollback_ddl=False`.