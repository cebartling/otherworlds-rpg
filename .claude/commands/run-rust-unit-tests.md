# Run Rust Unit Tests

Run all inline `#[cfg(test)]` unit tests across the Rust workspace and present a clean, scannable summary.

---

## Step 1: Run Tests

Run from the `backend/` directory:

```bash
cargo test --lib 2>&1
```

`--lib` runs only inline unit tests — no integration tests, no database required.

**You must actually run this command and read the full output. Do not summarize from memory.**

---

## Step 2: Parse Output

Read the cargo test output and extract per-crate results. The output contains blocks like:

```
Running unittests src/lib.rs (target/.../otherworlds_core-...)
running N tests
test module::test_name ... ok
test module::test_name ... FAILED
test result: ok. X passed; Y failed; Z ignored; ...
```

For each crate, collect:
- **Crate name** (from the `Running unittests` line or binary path)
- **Passed / failed / ignored counts** (from the `test result:` line)
- **Names of any FAILED tests**

Also capture any failure detail output (assertion messages, panic messages) that cargo prints for failing tests.

---

## Step 3: Present Summary

Output using this exact format:

```
## Unit Test Results

### Failing Tests
[List each failing test name with its crate, or "None" if all pass]

### Per-Crate Summary

| Crate | Passed | Failed | Ignored | Status |
|-------|--------|--------|---------|--------|
| ...   | ...    | ...    | ...     | pass/FAIL |

### Totals
- **Total passed:** N
- **Total failed:** N
- **Total ignored:** N
- **Overall: PASS / FAIL**
```

### Formatting rules

- **Failures first** — failing test names appear at the top before the table so the user never has to scroll to find them.
- **Strip prefixes** — remove the `otherworlds-` prefix from crate names for readability (e.g., show `core`, `api`, `narrative`).
- **Failure details** — if any tests failed, include cargo's failure output (assertion messages, panics) in a collapsed `<details>` block below the summary.
- **Only crates with tests** — omit crates that had zero tests from the table.
