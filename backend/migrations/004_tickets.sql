CREATE TABLE tickets (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug          TEXT UNIQUE NOT NULL,
    ticket_number BIGINT NOT NULL,
    project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ticket_type   TEXT NOT NULL CHECK (ticket_type IN ('epic', 'story', 'task', 'subtask', 'bug')),
    title         TEXT NOT NULL,
    description   TEXT,
    status        TEXT NOT NULL DEFAULT 'backlog'
                      CHECK (status IN ('backlog', 'todo', 'in_progress', 'in_review', 'done')),
    priority      TEXT NOT NULL DEFAULT 'medium'
                      CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    assignee_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    reporter_id   UUID NOT NULL REFERENCES users(id),
    parent_id     UUID REFERENCES tickets(id) ON DELETE SET NULL,
    story_points  INTEGER,
    sprint_id     UUID REFERENCES sprints(id) ON DELETE SET NULL,
    due_date      DATE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_id, ticket_number)
);
