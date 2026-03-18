use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use sqlx::PgPool;

use crate::{
    activity,
    auth::middleware::CurrentUser,
    error::{AppError, AppResult},
    projects::handlers::{ensure_member, get_project_by_key},
    tickets::{models::*, slug::next_slug},
    AppState,
};

pub async fn list_tickets(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Query(filters): Query<TicketFilters>,
) -> AppResult<Json<Vec<Ticket>>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    let tickets = sqlx::query_as!(
        Ticket,
        r#"
        SELECT DISTINCT t.id, t.slug, t.ticket_number, t.project_id, t.ticket_type,
               t.title, t.description, t.status, t.priority,
               t.assignee_id, t.reporter_id, t.parent_id,
               t.story_points, t.sprint_id, t.due_date, t.created_at, t.updated_at
        FROM tickets t
        LEFT JOIN ticket_tags tt ON tt.ticket_id = t.id
        WHERE t.project_id = $1
          AND ($2::text IS NULL OR t.status = $2)
          AND ($3::uuid IS NULL OR t.assignee_id = $3)
          AND ($4::uuid IS NULL OR t.sprint_id = $4)
          AND ($5::text IS NULL OR t.ticket_type = $5)
          AND ($6::uuid IS NULL OR tt.tag_id = $6)
          AND ($7::text IS NULL OR t.title ILIKE '%' || $7 || '%')
        ORDER BY t.created_at DESC
        "#,
        project.id,
        filters.status,
        filters.assignee_id,
        filters.sprint_id,
        filters.ticket_type,
        filters.tag_id,
        filters.search,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tickets))
}

pub async fn create_ticket(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateTicketRequest>,
) -> AppResult<Json<Ticket>> {
    let project = get_project_by_key(&state.pool, &key).await?;
    ensure_member(&state.pool, project.id, current_user.id).await?;

    validate_ticket_type(&req.ticket_type)?;
    let status = req.status.as_deref().unwrap_or("backlog");
    let priority = req.priority.as_deref().unwrap_or("medium");
    validate_status(status)?;
    validate_priority(priority)?;

    let mut tx = state.pool.begin().await?;
    let (slug, ticket_number) = next_slug(&mut tx, project.id, &project.key).await?;

    let ticket = sqlx::query_as!(
        Ticket,
        r#"
        INSERT INTO tickets (
            slug, ticket_number, project_id, ticket_type, title, description,
            status, priority, assignee_id, reporter_id, parent_id,
            story_points, sprint_id, due_date
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id, slug, ticket_number, project_id, ticket_type, title, description,
                  status, priority, assignee_id, reporter_id, parent_id,
                  story_points, sprint_id, due_date, created_at, updated_at
        "#,
        slug,
        ticket_number,
        project.id,
        req.ticket_type,
        req.title,
        req.description,
        status,
        priority,
        req.assignee_id,
        current_user.id,
        req.parent_id,
        req.story_points,
        req.sprint_id,
        req.due_date,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(ticket))
}

pub async fn get_ticket(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<TicketDetail>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let reporter = sqlx::query_as!(
        UserStub,
        "SELECT id, username, display_name, avatar_url FROM users WHERE id = $1",
        ticket.reporter_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Reporter not found".to_string()))?;

    let assignee = if let Some(assignee_id) = ticket.assignee_id {
        sqlx::query_as!(
            UserStub,
            "SELECT id, username, display_name, avatar_url FROM users WHERE id = $1",
            assignee_id
        )
        .fetch_optional(&state.pool)
        .await?
    } else {
        None
    };

    let parent = if let Some(parent_id) = ticket.parent_id {
        sqlx::query_as!(
            TicketSummary,
            "SELECT id, slug, title, ticket_type, status FROM tickets WHERE id = $1",
            parent_id
        )
        .fetch_optional(&state.pool)
        .await?
    } else {
        None
    };

    let tags = sqlx::query_as!(
        crate::tags::handlers::Tag,
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

    let children = sqlx::query_as!(
        TicketSummary,
        "SELECT id, slug, title, ticket_type, status FROM tickets WHERE parent_id = $1 ORDER BY created_at",
        ticket.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(TicketDetail {
        id: ticket.id,
        slug: ticket.slug,
        ticket_number: ticket.ticket_number,
        project_id: ticket.project_id,
        ticket_type: ticket.ticket_type,
        title: ticket.title,
        description: ticket.description,
        status: ticket.status,
        priority: ticket.priority,
        assignee_id: ticket.assignee_id,
        reporter_id: ticket.reporter_id,
        parent_id: ticket.parent_id,
        story_points: ticket.story_points,
        sprint_id: ticket.sprint_id,
        due_date: ticket.due_date,
        created_at: ticket.created_at,
        updated_at: ticket.updated_at,
        assignee,
        reporter,
        parent,
        tags,
        children,
    }))
}

pub async fn get_children(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<Vec<TicketSummary>>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let children = sqlx::query_as!(
        TicketSummary,
        "SELECT id, slug, title, ticket_type, status FROM tickets WHERE parent_id = $1 ORDER BY created_at",
        ticket.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(children))
}

pub async fn update_ticket(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateTicketRequest>,
) -> AppResult<Json<Ticket>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    if let Some(ref s) = req.status { validate_status(s)?; }
    if let Some(ref p) = req.priority { validate_priority(p)?; }

    let old_status = ticket.status.clone();
    let old_assignee = ticket.assignee_id;

    let updated = sqlx::query_as!(
        Ticket,
        r#"
        UPDATE tickets SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            assignee_id = CASE WHEN $5::boolean THEN $6 ELSE assignee_id END,
            parent_id = COALESCE($7, parent_id),
            story_points = COALESCE($8, story_points),
            sprint_id = COALESCE($9, sprint_id),
            due_date = COALESCE($10, due_date),
            updated_at = NOW()
        WHERE id = $11
        RETURNING id, slug, ticket_number, project_id, ticket_type, title, description,
                  status, priority, assignee_id, reporter_id, parent_id,
                  story_points, sprint_id, due_date, created_at, updated_at
        "#,
        req.title,
        req.description,
        req.status,
        req.priority,
        req.assignee_id.is_some() as bool,
        req.assignee_id,
        req.parent_id,
        req.story_points,
        req.sprint_id,
        req.due_date,
        ticket.id,
    )
    .fetch_one(&state.pool)
    .await?;

    if let Some(ref new_status) = req.status {
        if *new_status != old_status {
            activity::log(&state.pool, ticket.id, current_user.id, "status_changed",
                Some(&old_status), Some(new_status)).await.ok();
        }
    }
    if req.assignee_id.is_some() && req.assignee_id != old_assignee {
        let new_val = req.assignee_id.map(|id| id.to_string());
        activity::log(&state.pool, ticket.id, current_user.id, "assignee_changed",
            old_assignee.map(|id| id.to_string()).as_deref(),
            new_val.as_deref()).await.ok();
    }

    Ok(Json(updated))
}

pub async fn delete_ticket(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<serde_json::Value>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    sqlx::query!("DELETE FROM tickets WHERE id = $1", ticket.id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn patch_status(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<StatusUpdateRequest>,
) -> AppResult<Json<Ticket>> {
    validate_status(&req.status)?;
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let old_status = ticket.status.clone();
    let updated = sqlx::query_as!(
        Ticket,
        r#"
        UPDATE tickets SET status = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, slug, ticket_number, project_id, ticket_type, title, description,
                  status, priority, assignee_id, reporter_id, parent_id,
                  story_points, sprint_id, due_date, created_at, updated_at
        "#,
        req.status,
        ticket.id
    )
    .fetch_one(&state.pool)
    .await?;

    activity::log(&state.pool, ticket.id, current_user.id, "status_changed",
        Some(&old_status), Some(&req.status)).await.ok();

    Ok(Json(updated))
}

pub async fn patch_assign(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignRequest>,
) -> AppResult<Json<Ticket>> {
    let ticket = fetch_ticket_by_slug(&state.pool, &slug).await?;
    ensure_member(&state.pool, ticket.project_id, current_user.id).await?;

    let old_assignee = ticket.assignee_id;
    let updated = sqlx::query_as!(
        Ticket,
        r#"
        UPDATE tickets SET assignee_id = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, slug, ticket_number, project_id, ticket_type, title, description,
                  status, priority, assignee_id, reporter_id, parent_id,
                  story_points, sprint_id, due_date, created_at, updated_at
        "#,
        req.assignee_id,
        ticket.id
    )
    .fetch_one(&state.pool)
    .await?;

    activity::log(&state.pool, ticket.id, current_user.id, "assignee_changed",
        old_assignee.map(|id| id.to_string()).as_deref(),
        req.assignee_id.map(|id| id.to_string()).as_deref()).await.ok();

    Ok(Json(updated))
}

pub async fn fetch_ticket_by_slug(pool: &PgPool, slug: &str) -> AppResult<Ticket> {
    sqlx::query_as!(
        Ticket,
        r#"
        SELECT id, slug, ticket_number, project_id, ticket_type, title, description,
               status, priority, assignee_id, reporter_id, parent_id,
               story_points, sprint_id, due_date, created_at, updated_at
        FROM tickets WHERE slug = $1
        "#,
        slug
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Ticket {slug} not found")))
}

fn validate_ticket_type(t: &str) -> AppResult<()> {
    match t {
        "epic" | "story" | "task" | "subtask" | "bug" => Ok(()),
        _ => Err(AppError::BadRequest(format!("Invalid ticket type: {t}"))),
    }
}

fn validate_status(s: &str) -> AppResult<()> {
    match s {
        "backlog" | "todo" | "in_progress" | "in_review" | "done" => Ok(()),
        _ => Err(AppError::BadRequest(format!("Invalid status: {s}"))),
    }
}

fn validate_priority(p: &str) -> AppResult<()> {
    match p {
        "low" | "medium" | "high" | "critical" => Ok(()),
        _ => Err(AppError::BadRequest(format!("Invalid priority: {p}"))),
    }
}
