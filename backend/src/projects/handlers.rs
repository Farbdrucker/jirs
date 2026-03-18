use axum::{
    extract::{Extension, Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::middleware::CurrentUser,
    error::{AppError, AppResult},
    projects::models::*,
    AppState,
};

pub async fn list_projects(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<Project>>> {
    let projects = sqlx::query_as!(
        Project,
        r#"
        SELECT DISTINCT p.id, p.key, p.name, p.description, p.owner_id, p.created_at
        FROM projects p
        LEFT JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = $1
        WHERE p.owner_id = $1 OR pm.user_id IS NOT NULL
        ORDER BY p.created_at DESC
        "#,
        current_user.id
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(projects))
}

pub async fn create_project(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProjectRequest>,
) -> AppResult<Json<Project>> {
    let key_re = regex::Regex::new(r"^[A-Z]{2,10}$").unwrap();
    if !key_re.is_match(&req.key) {
        return Err(AppError::BadRequest(
            "Project key must be 2-10 uppercase letters".to_string(),
        ));
    }

    let mut tx = state.pool.begin().await?;

    let project = sqlx::query_as!(
        Project,
        r#"
        INSERT INTO projects (key, name, description, owner_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, key, name, description, owner_id, created_at
        "#,
        req.key,
        req.name,
        req.description,
        current_user.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint().is_some() {
                return AppError::Conflict("Project key already exists".to_string());
            }
        }
        AppError::Database(e)
    })?;

    sqlx::query!(
        "INSERT INTO project_ticket_counter (project_id) VALUES ($1)",
        project.id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, 'admin')",
        project.id,
        current_user.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(project))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Project>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;
    Ok(Json(project))
}

pub async fn update_project(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateProjectRequest>,
) -> AppResult<Json<Project>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_admin(&state.pool, project.id, current_user.id).await?;

    let project = sqlx::query_as!(
        Project,
        r#"
        UPDATE projects
        SET name = COALESCE($1, name),
            description = COALESCE($2, description)
        WHERE id = $3
        RETURNING id, key, name, description, owner_id, created_at
        "#,
        req.name,
        req.description,
        project.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(project))
}

pub async fn get_members(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<ProjectMember>>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let members = sqlx::query_as!(
        ProjectMember,
        r#"
        SELECT u.id AS user_id, u.username, u.display_name, u.avatar_url,
               pm.role, pm.joined_at
        FROM project_members pm
        JOIN users u ON u.id = pm.user_id
        WHERE pm.project_id = $1
        ORDER BY pm.joined_at
        "#,
        project.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(members))
}

pub async fn add_member(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AddMemberRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_admin(&state.pool, project.id, current_user.id).await?;

    let role = req.role.unwrap_or_else(|| "member".to_string());
    if !["admin", "member", "viewer"].contains(&role.as_str()) {
        return Err(AppError::BadRequest("Invalid role".to_string()));
    }

    sqlx::query!(
        r#"
        INSERT INTO project_members (project_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id, user_id) DO UPDATE SET role = $3
        "#,
        project.id,
        req.user_id,
        role
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn remove_member(
    State(state): State<AppState>,
    Path((key, user_id)): Path<(String, Uuid)>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_admin(&state.pool, project.id, current_user.id).await?;

    if user_id == project.owner_id {
        return Err(AppError::Forbidden("Cannot remove project owner".to_string()));
    }

    sqlx::query!(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2",
        project.id,
        user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn get_project_by_key(pool: &PgPool, key: &str) -> AppResult<Project> {
    sqlx::query_as!(
        Project,
        "SELECT id, key, name, description, owner_id, created_at FROM projects WHERE key = $1",
        key
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Project {key} not found")))
}

pub async fn ensure_member(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> AppResult<()> {
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM projects WHERE id = $1 AND owner_id = $2
            UNION
            SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2
        ) AS "exists!"
        "#,
        project_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    if !exists {
        return Err(AppError::Forbidden("Not a project member".to_string()));
    }
    Ok(())
}

pub async fn ensure_admin(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> AppResult<()> {
    let project_owner = sqlx::query_scalar!(
        "SELECT owner_id FROM projects WHERE id = $1",
        project_id
    )
    .fetch_one(pool)
    .await?;

    if project_owner == user_id {
        return Ok(());
    }

    let role = sqlx::query_scalar!(
        "SELECT role FROM project_members WHERE project_id = $1 AND user_id = $2",
        project_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    match role.as_deref() {
        Some("admin") => Ok(()),
        _ => Err(AppError::Forbidden("Project admin required".to_string())),
    }
}
