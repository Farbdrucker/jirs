CREATE INDEX IF NOT EXISTS idx_tickets_parent ON tickets(parent_id) WHERE parent_id IS NOT NULL;
