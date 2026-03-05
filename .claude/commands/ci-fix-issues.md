# Fix CI: Automated CI Failure Diagnosis and Resolution

Fetch CI failure logs from GitHub Actions, diagnose the root cause, apply fixes, verify locally, and report results.

---

## Step 1: Identify the CI Run

Use `$ARGUMENTS` to determine which CI run to inspect:

- **If `$ARGUMENTS` is a small number** (likely a PR number): run `gh pr checks $ARGUMENTS --json name,state,link`
  - If any checks failed, get the run ID from: `gh pr checks $ARGUMENTS --json link --jq '.[] | select(.state == "FAILURE") | .link'`
  - Extract the run ID from the URL (the numeric segment after `/actions/runs/`)
- **If `$ARGUMENTS` is a large number** (likely a run ID): run `gh run view $ARGUMENTS`
- **If no argument**: run `gh run list --branch $(git branch --show-current) --limit 5` and pick the most recent run

**You must actually run these commands and read the output. Do not guess.**

### Early exits

- **No CI runs found** → Output "No CI runs found for this branch." and stop.
- **Run still in progress** → Output "CI run `<id>` is still in progress. Wait for it to finish." and stop.
- **Run already passed** → Output "CI is green on run `<id>`. Nothing to fix." and stop.

---

## Step 2: Identify Failed Jobs

Run:

```bash
gh run view <run-id>
```

Note which jobs failed (e.g., `rust-tests`, `web-tests`, `ios-tests`). Only failed jobs need attention.

---

## Step 3: Fetch Failure Logs

Run:

```bash
gh run view <run-id> --log-failed
```

Read the full output. Identify the specific error messages, file paths, and line numbers from the log.

---

## Step 4: Diagnose and Fix

For each failure, follow the appropriate decision path:

### Rust failures

- **`cargo fmt` failure** → Run `cd backend && cargo fmt` and stage the changes.
- **`cargo clippy` failure** → Run `cd backend && cargo clippy --all-targets` locally, read each warning, fix the source code.
- **`cargo test` failure** → Run the specific failing test: `cd backend && cargo test <test_name> -- --nocapture`. Read the assertion/panic message. Read the relevant source files. Apply the minimal fix.

### Web failures

- **`npm run check` failure** → Run `cd web && npm run check`, fix TypeScript/Svelte errors.
- **`npm run build` failure** → Run `cd web && npm run build`, fix build errors.

### iOS failures

- **Build/test failure** → Read `.github/workflows/ci.yml` for the exact xcodebuild command. Run it locally. Fix Swift errors.

### Fix rules

- Read the relevant source files before changing anything. Understand the error.
- Apply the **minimal fix**. Do not refactor surrounding code.
- If a fix requires a design decision (ambiguous intent, multiple valid approaches), mark it as `[UNFIXABLE]` and explain why.
- If a fix causes a new failure, attempt up to **3 fix-verify cycles**. After 3 cycles, mark remaining issues as `[UNFIXABLE]`.

---

## Step 5: Verify Locally

After all fixes, re-run **only the commands that correspond to failed jobs**:

- **Rust:** `cd backend && cargo fmt -- --check && cargo clippy --all-targets && cargo test`
- **Web:** `cd web && npm run check && npm run build`
- **iOS:** the xcodebuild command from `.github/workflows/ci.yml`

Do not run verification for jobs that passed in CI.

**You must actually run these commands and read the output. Do not assume they pass.**

---

## Step 6: Report Results

Output using this exact format:

```
## CI Fix Report

### Run: <run-id> (<branch>)

### Fixes Applied
- [FIXED] `file:line` — Description of what was wrong and what was changed
- [UNFIXABLE] Description — why this can't be auto-fixed

### Local Verification

| Check | Result |
|-------|--------|
| cargo fmt | pass/FAIL |
| cargo clippy | pass/FAIL |
| cargo test | pass/FAIL |
| npm run check | pass/FAIL |
| npm run build | pass/FAIL |

(Only include rows for checks that were actually run.)

### Status: ALL FIXED / PARTIALLY FIXED / UNFIXABLE
```

### Status definitions

- **ALL FIXED** — Every failure was resolved and local verification passes.
- **PARTIALLY FIXED** — Some failures were fixed but at least one remains as `[UNFIXABLE]`.
- **UNFIXABLE** — No failures could be automatically resolved.

---

## Final reminders

- Every `[FIXED]` entry **must** cite `file:line`.
- Do not commit or push. The user decides when to commit.
- If the CI log is ambiguous, say so. Do not guess at fixes.
