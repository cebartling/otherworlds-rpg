# ADR-0024: Proactive Optimistic Concurrency Control in Event Store

## Status

Accepted

## Context

The event store (ADR-0001) must prevent concurrent writes from corrupting an aggregate's event stream. If two command handlers load the same aggregate at version N and both try to append events starting at N+1, one must fail.

The standard approach relies on a unique constraint on `(aggregate_id, sequence_number)` in the database. The write that violates the constraint fails with a database error, which is caught and mapped to a concurrency conflict.

This reactive approach has two drawbacks:

1. **Opaque errors** — the database error message does not include the expected vs. actual version, making diagnostics harder.
2. **Wasted work** — the INSERT is attempted even when the conflict is detectable beforehand, consuming a database round-trip for the write and potentially for the constraint violation error.

## Decision

`PgEventRepository::append_events` performs a **proactive version check** before inserting events:

1. Within the same transaction, execute `SELECT COALESCE(MAX(sequence_number), 0)` for the aggregate.
2. Compare the result against the caller's `expected_version`.
3. If they don't match, return `DomainError::ConcurrencyConflict { aggregate_id, expected, actual }` immediately — no INSERT is attempted.
4. If they match, proceed with the batch INSERT using `UNNEST` arrays.
5. As a safety net, if the INSERT still fails due to a unique constraint violation (race between the SELECT and INSERT), a second diagnostic query retrieves the actual version for the error message.

The proactive check and the INSERT occur within a single database transaction, minimizing the race window.

## Consequences

### Easier

- Concurrency conflict errors always include the expected and actual version numbers, enabling meaningful error messages to API consumers.
- The common case (no conflict) adds only one lightweight SELECT per append — the same query that would be needed to detect the conflict reactively.
- The two-phase approach (proactive check + constraint safety net) provides defense in depth.

### More Difficult

- The implementation is slightly more complex than relying solely on the unique constraint.
- Under extreme concurrency, the SELECT-then-INSERT pattern has a narrow race window where two transactions could both pass the version check. The unique constraint catches this, but it means the proactive check is not a complete replacement — both mechanisms are needed.

### Unchanged

- The append-only event model is preserved.
- The `EventRepository` trait contract is unchanged — callers pass `expected_version` as before.
- Domain crate isolation is unaffected — this is an infrastructure concern in `otherworlds-event-store`.
