The only remaining references are the test function *names* (`parse_feature_flags` and `parse_feature_flags_defaults`), which are just identifiers for the test functions — not the TOML keys or field accesses. The plan didn't ask to rename the test function names, only the TOML strings and field accesses within them, which are already updated.

Here's a summary of all changes made:

### Changes Summary

1. **`docs/api-reference/fabro-api.yaml`** — Renamed `feature_flags` property → `features` and `FeatureFlags` schema → `Features` in `ServerConfiguration`

2. **`lib/crates/fabro-config/src/server.rs`** — Renamed struct `FeatureFlags` → `Features`, field `feature_flags` → `features` in `ServerConfig`, and updated test TOML strings/field accesses

3. **`lib/crates/fabro-api/src/demo/mod.rs`** — Updated `feature_flags: FeatureFlags { … }` → `features: Features { … }`

4. **`lib/crates/fabro-api/tests/openapi_conformance.rs`** — Updated `feature_flags: FeatureFlags { … }` → `features: Features { … }`

5. **`docker/demo-server.toml`** — Renamed `[feature_flags]` → `[features]`

6. **`apps/fabro-web/app/lib/config.server.ts`** — Renamed interface `FeatureFlags` → `Features`, property `feature_flags` → `features`, constant `FEATURE_FLAGS_DEFAULTS` → `FEATURES_DEFAULTS`, and variable `rawFeatureFlags` → `rawFeatures`

7. **`apps/fabro-web/app/layouts/app-shell.tsx`** — Updated `feature_flags: config.feature_flags` → `features: config.features`

8. **`apps/fabro-web/app/routes/start.tsx`** — Updated all `feature_flags` references to `features`

9. **Regenerated `fabro-types`** via `cargo build` (picks up OpenAPI rename automatically)

10. **Regenerated TypeScript API client** via `bun run generate` and removed stale `feature-flags.ts`

### Verification Results
- ✅ `cargo build --workspace` — passes
- ✅ `cargo test -p fabro-config` — 58 tests passed
- ✅ `cargo test -p fabro-api` — 76 tests passed (including `server_config_keys_match_openapi_spec`)
- ✅ `bun run typecheck` — passes