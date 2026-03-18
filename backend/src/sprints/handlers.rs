use axum::{
    extract::{Extension, Path, State},
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::middleware::CurrentUser,
    error::{AppError, AppResult},
    projects::handlers::{ensure_member, get_project_by_key},
    AppState,
};

#[derive(Debug, Serialize)]
pub struct Sprint {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub goal: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSprintRequest {
    pub name: String,
    pub goal: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSprintRequest {
    pub name: Option<String>,
    pub goal: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

pub async fn list_sprints(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<Sprint>>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let sprints = sqlx::query_as!(
        Sprint,
        "SELECT id, project_id, name, goal, start_date, end_date, status, created_at
         FROM sprints WHERE project_id = $1 ORDER BY created_at DESC",
        project.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(sprints))
}

pub async fn create_sprint(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateSprintRequest>,
) -> AppResult<Json<Sprint>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let sprint = sqlx::query_as!(
        Sprint,
        r#"
        INSERT INTO sprints (project_id, name, goal, start_date, end_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, project_id, name, goal, start_date, end_date, status, created_at
        "#,
        project.id,
        req.name,
        req.goal,
        req.start_date,
        req.end_date,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(sprint))
}

pub async fn update_sprint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateSprintRequest>,
) -> AppResult<Json<Sprint>> {
    let sprint = fetch_sprint(&state.pool, id).await?;
    ensure_member(&state.pool, sprint.project_id, current_user.id).await?;

    let sprint = sqlx::query_as!(
        Sprint,
        r#"
        UPDATE sprints SET
            name = COALESCE($1, name),
            goal = COALESCE($2, goal),
            start_date = COALESCE($3, start_date),
            end_date = COALESCE($4, end_date)
        WHERE id = $5
        RETURNING id, project_id, name, goal, start_date, end_date, status, created_at
        "#,
        req.name, req.goal, req.start_date, req.end_date, id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(sprint))
}

pub async fn start_sprint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Sprint>> {
    let sprint = fetch_sprint(&state.pool, id).await?;
    ensure_member(&state.pool, sprint.project_id, current_user.id).await?;

    let active_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sprints WHERE project_id = $1 AND status = 'active'",
        sprint.project_id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    if active_count > 0 {
        return Err(AppError::Conflict("Another sprint is already active".to_string()));
    }

    let sprint = sqlx::query_as!(
        Sprint,
        r#"
        UPDATE sprints SET status = 'active'
        WHERE id = $1
        RETURNING id, project_id, name, goal, start_date, end_date, status, created_at
        "#,
        id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(sprint))
}

pub async fn complete_sprint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Sprint>> {
    let sprint = fetch_sprint(&state.pool, id).await?;
    ensure_member(&state.pool, sprint.project_id, current_user.id).await?;

    let sprint = sqlx::query_as!(
        Sprint,
        r#"
        UPDATE sprints SET status = 'completed'
        WHERE id = $1
        RETURNING id, project_id, name, goal, start_date, end_date, status, created_at
        "#,
        id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(sprint))
}

async fn fetch_sprint(pool: &PgPool, id: Uuid) -> AppResult<Sprint> {
    sqlx::query_as!(
        Sprint,
        "SELECT id, project_id, name, goal, start_date, end_date, status, created_at
         FROM sprints WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Sprint {id} not found")))
}
