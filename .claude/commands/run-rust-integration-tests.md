# Run Rust Integration Tests

Run all integration tests (files in `tests/` directories) across the Rust workspace and present a clean, scannable summary. These tests require a running PostgreSQL instance.

---

## Step 1: Ensure PostgreSQL is Running

Run from the repo root:

```bash
docker compose up postgres -d
```

Then wait for the container to be healthy:

```bash
docker compose exec postgres pg_isready -U otherworlds
```

If `pg_isready` fails, retry up to 5 times with 2-second pauses. If PostgreSQL is still not ready after retries, report the error and stop — do not run tests against a dead database.

**You must actually run these commands. Do not skip this step.**

---

## Step 2: Run Integration Tests

Run from the `backend/` directory with `DATABASE_URL` set (required by `#[sqlx::test]`):

```bash
DATABASE_URL="postgres://otherworlds:otherworlds@localhost:5432/otherworlds" cargo test --test '*' 2>&1
```

`--test '*'` runs only integration test binaries (files in `tests/` directories), skipping inline `#[cfg(test)]` unit tests. These tests use `#[sqlx::test(migrations = "../../migrations")]` which auto-runs migrations per test. The `DATABASE_URL` env var must point to the running PostgreSQL instance — `sqlx::test` will panic at startup if it is missing.

**You must actually run this command and read the full output. Do not summarize from memory.**

---

## Step 3: Parse Output

Read the cargo test output and extract per-crate, per-file results. The output contains blocks like:

```
Running tests/health_test.rs (target/.../health_test-...)
running N tests
test test_name ... ok
test test_name ... FAILED
test result: ok. X passed; Y failed; Z ignored; ...
```

For each test binary, collect:
- **Crate name** (from the binary path, e.g., `otherworlds_api` → `api`)
- **Test file** (from the `Running tests/...` line, e.g., `health_test.rs`)
- **Passed / failed / ignored counts** (from the `test result:` line)
- **Names of any FAILED tests**

Also capture any failure detail output (assertion messages, panic messages) that cargo prints for failing tests.

---

## Step 4: Present Summary

Output using this exact format:

```
## Integration Test Results

### Failing Tests
[List each failing test name with its crate and file, or "None" if all pass]

### Per-Crate Summary

| Crate | Test File | Passed | Failed | Ignored | Status |
|-------|-----------|--------|--------|---------|--------|
| ...   | ...       | ...    | ...    | ...     | pass/FAIL |

### Totals
- **Total passed:** N
- **Total failed:** N
- **Total ignored:** N
- **Overall: PASS / FAIL**
```

### Formatting rules

- **Failures first** — failing test names appear at the top before the table so the user never has to scroll to find them.
- **Test file column** — the API crate has multiple test files; show which file each result row comes from.
- **Strip prefixes** — remove the `otherworlds-` prefix from crate names for readability (e.g., show `event-store`, `api`).
- **Failure details** — if any tests failed, include cargo's failure output (assertion messages, panics) in a collapsed `<details>` block below the summary.
- **Only crates with integration tests** — omit crates that had zero integration tests from the table.
