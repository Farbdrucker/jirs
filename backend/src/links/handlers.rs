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
    projects::handlers::ensure_member,
    tickets::handlers::fetch_ticket_by_slug,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct TicketLink {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub target_slug: String,
    pub target_title: String,
    pub link_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RepoLink {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub repo_url: String,
    pub branch_name: Option<String>,
    pub pr_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub target_slug: String,
    pub link_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoLinkRequest {
    pub repo_url: String,
    pub branch_name: Option<String>,
    pub pr_url: Option<String>,
}

const VALID_LINK_TYPES: &[&str] = &[
    "blocks", "is_blocked_by", "relates_to", "duplicates", "is_duplicated_by"
];

fn inverse_link_type(t: &str) -> &str {
    match t {
        "blocks" => "is_blocked_by",
        "is_blocked_by" => "blocks",
        "duplicates" => "is_duplicated_by",
        "is_duplicated_by" => "duplicates",
        _ => "relates_to",
    }
}

pub async fn create_link(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateLinkRequest>,
) -> AppResult<Json<TicketLink>> {
    if !VALID_LINK_TYPES.contains(&req.link_type.as_str()) {
        return Err(AppError::BadRequest(format!("Invalid link type: {}", req.link_type)));
    }

    let source = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, source.project_id, current_user.id).await?;
    let target = fetch_ticket_by_slug(&state.pool, &req.target_slug).await?;

    if source.id == target.id {
        return Err(AppError::BadRequest("Cannot link ticket to itself".to_string()));
    }

    let mut tx = state.pool.begin().await?;

    sqlx::query!(
        "INSERT INTO ticket_links (source_id, target_id, link_type) VALUES ($1, $2, $3)",
        source.id, target.id, req.link_type
    )
    .execute(&mut *tx)
    .await?;

    let inverse = inverse_link_type(&req.link_type);
    sqlx::query!(
        "INSERT INTO ticket_links (source_id, target_id, link_type) VALUES ($1, $2, $3)",
        target.id, source.id, inverse
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let link = sqlx::query_as!(
        TicketLink,
        r#"
        SELECT tl.id, tl.source_id, tl.target_id, t.slug AS target_slug,
               t.title AS target_title, tl.link_type, tl.created_at
        FROM ticket_links tl
        JOIN tickets t ON t.id = tl.target_id
        WHERE tl.source_id = $1 AND tl.target_id = $2
        ORDER BY tl.created_at DESC LIMIT 1
        "#,
        source.id, target.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(link))
}

pub async fn delete_link(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, Uuid)>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let link = sqlx::query!(
        "SELECT source_id, target_id FROM ticket_links WHERE id = $1",
        id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    sqlx::query!("DELETE FROM ticket_links WHERE id = $1", id)
        .execute(&state.pool)
        .await?;

    sqlx::query!(
        "DELETE FROM ticket_links WHERE source_id = $1 AND target_id = $2",
        link.target_id, link.source_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn create_repo_link(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateRepoLinkRequest>,
) -> AppResult<Json<RepoLink>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let link = sqlx::query_as!(
        RepoLink,
        r#"
        INSERT INTO repo_links (ticket_id, repo_url, branch_name, pr_url)
        VALUES ($1, $2, $3, $4)
        RETURNING id, ticket_id, repo_url, branch_name, pr_url, created_at
        "#,
        ticket.id,
        req.repo_url,
        req.branch_name,
        req.pr_url,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(link))
}

pub async fn delete_repo_link(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, Uuid)>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    sqlx::query!(
        "DELETE FROM repo_links WHERE id = $1 AND ticket_id = $2",
        id, ticket.id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
