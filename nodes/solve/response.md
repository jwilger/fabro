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

This ensures that `sqlmigrate` only wraps output in `BEGIN`/`COMMIT` when **both** conditions are true: the migration is atomic **and** the database supports transactional DDL (`can_rollback_ddl`). This matches the behavior of the migration executor itself, which already checks both conditions.

### 2. `tests/migrations/test_commands.py` — New test `test_sqlmigrate_for_non_transactional_databases`

Added a test that mocks `connection.features.can_rollback_ddl` to `False` and verifies that the `BEGIN`/`COMMIT` transaction wrappers are **not** included in the `sqlmigrate` output, even for an atomic migration. The test is modeled after the existing `test_sqlmigrate_for_non_atomic_migration` test.