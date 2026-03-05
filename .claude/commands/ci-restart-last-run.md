# CI Restart: Re-run Last Failed CI Run

Re-run the most recent failed GitHub Actions workflow run on the current branch.

---

## Step 1: Find the Most Recent Failed Run

Run:

```bash
gh run list --branch $(git branch --show-current) --limit 5
```

From the output, identify the **most recent run with a `failure` conclusion**. Note its run ID.

### Early exits

- **No runs found** → Output "No CI runs found for this branch." and stop.
- **No failed runs** → Output "No failed CI runs on this branch. Nothing to re-run." and stop.
- **Most recent failed run is already in progress** (re-run was already triggered) → Output "Run `<id>` is already in progress." and stop.

---

## Step 2: Re-run Failed Jobs

Run:

```bash
gh run rerun <run-id> --failed
```

This re-runs only the failed jobs, not the entire workflow.

---

## Step 3: Confirm

Output:

```
Re-triggered failed jobs on run <run-id> (<branch>).
Monitor with: /ci-status
```
