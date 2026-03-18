CREATE TABLE sprints (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    goal       TEXT,
    start_date DATE,
    end_date   DATE,
    status     TEXT NOT NULL DEFAULT 'planning'
                   CHECK (status IN ('planning', 'active', 'completed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
