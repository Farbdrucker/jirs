use axum::{
    extract::{Extension, Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::middleware::CurrentUser,
    error::AppResult,
    projects::handlers::ensure_member,
    tickets::handlers::fetch_ticket_by_slug,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct ActivityEntry {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub actor_id: Uuid,
    pub actor_username: String,
    pub actor_display_name: String,
    pub action: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn get_activity(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<ActivityEntry>>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let entries = sqlx::query_as!(
        ActivityEntry,
        r#"
        SELECT a.id, a.ticket_id, a.actor_id, u.username AS actor_username,
               u.display_name AS actor_display_name,
               a.action, a.old_value, a.new_value, a.created_at
        FROM activity a
        JOIN users u ON u.id = a.actor_id
        WHERE a.ticket_id = $1
        ORDER BY a.created_at DESC
        "#,
        ticket.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(entries))
}
