-- Add index on event_type to support list_aggregate_ids queries.
CREATE INDEX IF NOT EXISTS idx_domain_events_event_type ON domain_events (event_type);
