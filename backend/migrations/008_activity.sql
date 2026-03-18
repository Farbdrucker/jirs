CREATE TABLE activity (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id   UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    actor_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action      TEXT NOT NULL,
    old_value   TEXT,
    new_value   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
