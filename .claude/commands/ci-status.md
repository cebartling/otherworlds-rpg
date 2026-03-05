# CI Status: Check Latest GitHub Actions Workflow

Check the current status of CI workflow runs and present a concise dashboard.

---

## Step 1: Determine Which Run to Check

Use `$ARGUMENTS` to determine scope:

- **If `$ARGUMENTS` is a PR number** (small number): run `gh pr checks $ARGUMENTS`
- **If `$ARGUMENTS` is a run ID** (large number): run `gh run view $ARGUMENTS`
- **If `$ARGUMENTS` is a branch name**: run `gh run list --branch $ARGUMENTS --limit 5`
- **If no argument**: run `gh run list --branch $(git branch --show-current) --limit 5` and pick the most recent run

**You must actually run these commands and read the output. Do not guess.**

---

## Step 2: Get Run Details

For the identified run, fetch full details:

```bash
gh run view <run-id>
```

Extract:
- **Run ID**
- **Trigger** (push, pull_request, etc.)
- **Branch**
- **Commit message** (first line)
- **Overall status** (completed, in_progress, queued)
- **Overall conclusion** (success, failure, cancelled, or pending if still running)
- **Per-job status** (job name, status, duration)

If the run is still **in progress**, also run:

```bash
gh run watch <run-id> --exit-status 2>&1 &
```

Do **not** wait for it. Just note it is in progress.

---

## Step 3: If Failed, Preview Errors

If the run failed, run:

```bash
gh run view <run-id> --log-failed 2>&1 | tail -30
```

Extract the last few lines of error output to give a preview of what went wrong. Do not attempt to fix anything — that is what `/fix-ci` is for.

---

## Step 4: Present Dashboard

Output using this exact format:

```
## CI Status

### Run: <run-id>
- **Branch:** <branch>
- **Trigger:** <push|pull_request|...>
- **Commit:** <first line of commit message>
- **Started:** <relative time, e.g., "12 minutes ago">

### Jobs

| Job | Status | Duration |
|-----|--------|----------|
| ... | pass/FAIL/in_progress/skipped | Xs |

### Result: PASS / FAIL / IN PROGRESS / CANCELLED
```

If the run **failed**, append:

```
### Error Preview
<last few lines of failure output from --log-failed>

Run `/fix-ci` to diagnose and fix.
```

If the run **passed**:

```
### Result: PASS
All checks green.
```

If the run is **in progress**:

```
### Result: IN PROGRESS
<list which jobs are still running>
Re-run `/ci-status` to check again later.
```

---

## Final reminders

- This skill is **read-only**. Do not fix, commit, or push anything.
- Keep output short. The goal is a quick glance at CI health.
- If `gh` is not authenticated or the repo has no CI workflows, say so and stop.
