CREATE TABLE ticket_links (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id  UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    target_id  UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    link_type  TEXT NOT NULL CHECK (link_type IN (
                   'blocks', 'is_blocked_by', 'relates_to', 'duplicates', 'is_duplicated_by'
               )),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE repo_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id   UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    repo_url    TEXT NOT NULL,
    branch_name TEXT,
    pr_url      TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
