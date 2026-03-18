use axum::{
    extract::{Extension, Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    activity,
    auth::middleware::CurrentUser,
    error::{AppError, AppResult},
    projects::handlers::ensure_member,
    tickets::handlers::fetch_ticket_by_slug,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub author_avatar_url: Option<String>,
    pub parent_id: Option<Uuid>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub body: String,
}

pub async fn list_comments(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<Comment>>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let comments = sqlx::query_as!(
        Comment,
        r#"
        SELECT c.id, c.ticket_id, c.author_id, u.username AS author_username,
               u.display_name AS author_display_name, u.avatar_url AS author_avatar_url,
               c.parent_id, c.body, c.created_at, c.updated_at
        FROM comments c
        JOIN users u ON u.id = c.author_id
        WHERE c.ticket_id = $1
        ORDER BY c.created_at
        "#,
        ticket.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(comments))
}

pub async fn create_comment(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCommentRequest>,
) -> AppResult<Json<Comment>> {
    if req.body.trim().is_empty() {
        return Err(AppError::BadRequest("Comment body cannot be empty".to_string()));
    }

    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let comment = sqlx::query_as!(
        Comment,
        r#"
        WITH inserted AS (
            INSERT INTO comments (ticket_id, author_id, parent_id, body)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        )
        SELECT i.id, i.ticket_id, i.author_id, u.username AS author_username,
               u.display_name AS author_display_name, u.avatar_url AS author_avatar_url,
               i.parent_id, i.body, i.created_at, i.updated_at
        FROM inserted i
        JOIN users u ON u.id = i.author_id
        "#,
        ticket.id,
        current_user.id,
        req.parent_id,
        req.body,
    )
    .fetch_one(&state.pool)
    .await?;

    activity::log(&state.pool, ticket.id, current_user.id, "comment_added",
        None, Some(&req.body[..req.body.len().min(100)])).await.ok();

    Ok(Json(comment))
}

pub async fn update_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateCommentRequest>,
) -> AppResult<Json<Comment>> {
    let comment = sqlx::query_as!(
        Comment,
        r#"
        WITH updated AS (
            UPDATE comments SET body = $1, updated_at = NOW()
            WHERE id = $2 AND author_id = $3
            RETURNING *
        )
        SELECT u2.id, u2.ticket_id, u2.author_id, u.username AS author_username,
               u.display_name AS author_display_name, u.avatar_url AS author_avatar_url,
               u2.parent_id, u2.body, u2.created_at, u2.updated_at
        FROM updated u2
        JOIN users u ON u.id = u2.author_id
        "#,
        req.body,
        id,
        current_user.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Comment not found or not authorized".to_string()))?;

    Ok(Json(comment))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let deleted = sqlx::query!(
        "DELETE FROM comments WHERE id = $1 AND author_id = $2 RETURNING id",
        id,
        current_user.id
    )
    .fetch_optional(&state.pool)
    .await?;

    if deleted.is_none() {
        return Err(AppError::NotFound("Comment not found or not authorized".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
