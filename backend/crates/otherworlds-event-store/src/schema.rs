//! Event store database schema.

/// SQL to create the events table.
pub const CREATE_EVENTS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS domain_events (
    event_id        UUID PRIMARY KEY,
    aggregate_id    UUID NOT NULL,
    event_type      VARCHAR(255) NOT NULL,
    payload         JSONB NOT NULL,
    sequence_number BIGINT NOT NULL,
    correlation_id  UUID NOT NULL,
    causation_id    UUID NOT NULL,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (aggregate_id, sequence_number)
);

CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate_id
    ON domain_events (aggregate_id, sequence_number);

CREATE INDEX IF NOT EXISTS idx_domain_events_correlation_id
    ON domain_events (correlation_id);
";
