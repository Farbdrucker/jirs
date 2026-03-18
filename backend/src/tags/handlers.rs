use axum::{
    extract::{Extension, Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::middleware::CurrentUser,
    error::{AppError, AppResult},
    projects::handlers::{ensure_member, get_project_by_key},
    tickets::handlers::fetch_ticket_by_slug,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
}

pub async fn list_tags(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<Tag>>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let tags = sqlx::query_as!(
        Tag,
        "SELECT id, project_id, name, color, created_at FROM tags WHERE project_id = $1 ORDER BY name",
        project.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tags))
}

pub async fn create_tag(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateTagRequest>,
) -> AppResult<Json<Tag>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let color = req.color.unwrap_or_else(|| "#6366f1".to_string());
    let tag = sqlx::query_as!(
        Tag,
        r#"
        INSERT INTO tags (project_id, name, color) VALUES ($1, $2, $3)
        RETURNING id, project_id, name, color, created_at
        "#,
        project.id,
        req.name,
        color
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint().is_some() {
                return AppError::Conflict("Tag already exists".to_string());
            }
        }
        AppError::Database(e)
    })?;

    Ok(Json(tag))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path((key, id)): Path<(String, Uuid)>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    sqlx::query!("DELETE FROM tags WHERE id = $1 AND project_id = $2", id, project.id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn get_ticket_tags(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<Tag>>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let tags = sqlx::query_as!(
        Tag,
        r#"
        SELECT t.id, t.project_id, t.name, t.color, t.created_at
        FROM tags t
        JOIN ticket_tags tt ON tt.tag_id = t.id
        WHERE tt.ticket_id = $1
        ORDER BY t.name
        "#,
        ticket.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tags))
}

pub async fn add_tag_to_ticket(
    State(state): State<AppState>,
    Path((slug, tag_id)): Path<(String, Uuid)>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    sqlx::query!(
        "INSERT INTO ticket_tags (ticket_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        ticket.id,
        tag_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn remove_tag_from_ticket(
    State(state): State<AppState>,
    Path((slug, tag_id)): Path<(String, Uuid)>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    sqlx::query!(
        "DELETE FROM ticket_tags WHERE ticket_id = $1 AND tag_id = $2",
        ticket.id,
        tag_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
