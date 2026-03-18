use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    auth::middleware::CurrentUser,
    error::AppResult,
    projects::handlers::{ensure_member, get_project_by_key},
    tickets::{handlers::fetch_ticket_by_slug, models::Ticket},
    AppState,
};

pub const STATUS_ORDER: &[&str] = &["backlog", "todo", "in_progress", "in_review", "done"];

#[derive(Debug, Serialize)]
pub struct BoardColumn {
    pub status: String,
    pub tickets: Vec<Ticket>,
}

#[derive(Debug, Serialize)]
pub struct Board {
    pub columns: Vec<BoardColumn>,
}

#[derive(Debug, Deserialize)]
pub struct MoveTicketRequest {
    pub ticket_slug: String,
    pub to_status: String,
}

pub async fn get_kanban_board(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Board>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let tickets = sqlx::query_as!(
        Ticket,
        r#"
        SELECT id, slug, ticket_number, project_id, ticket_type, title, description,
               status, priority, assignee_id, reporter_id, parent_id,
               story_points, sprint_id, due_date, created_at, updated_at
        FROM tickets WHERE project_id = $1
        ORDER BY ticket_number
        "#,
        project.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(build_board(tickets)))
}

pub async fn get_scrum_board(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Board>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let tickets = sqlx::query_as!(
        Ticket,
        r#"
        SELECT t.id, t.slug, t.ticket_number, t.project_id, t.ticket_type, t.title, t.description,
               t.status, t.priority, t.assignee_id, t.reporter_id, t.parent_id,
               t.story_points, t.sprint_id, t.due_date, t.created_at, t.updated_at
        FROM tickets t
        JOIN sprints s ON s.id = t.sprint_id
        WHERE t.project_id = $1 AND s.status = 'active'
        ORDER BY t.ticket_number
        "#,
        project.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(build_board(tickets)))
}

pub async fn move_ticket(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<MoveTicketRequest>,
) -> AppResult<Json<Ticket>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &req.ticket_slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let updated = sqlx::query_as!(
        Ticket,
        r#"
        UPDATE tickets SET status = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, slug, ticket_number, project_id, ticket_type, title, description,
                  status, priority, assignee_id, reporter_id, parent_id,
                  story_points, sprint_id, due_date, created_at, updated_at
        "#,
        req.to_status,
        ticket.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}

fn build_board(tickets: Vec<Ticket>) -> Board {
    let mut map: HashMap<String, Vec<Ticket>> = HashMap::new();
    for t in tickets {
        map.entry(t.status.clone()).or_default().push(t);
    }

    let columns = STATUS_ORDER
        .iter()
        .map(|&status| BoardColumn {
            status: status.to_string(),
            tickets: map.remove(status).unwrap_or_default(),
        })
        .collect();

    Board { columns }
}
