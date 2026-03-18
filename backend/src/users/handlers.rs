use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::middleware::CurrentUser,
    error::{AppError, AppResult},
    users::models::UserResponse,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn list_users(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<UserSummary>>> {
    let users = sqlx::query_as!(
        UserSummary,
        "SELECT id, username, display_name, avatar_url FROM users ORDER BY display_name"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(users))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = sqlx::query_as!(
        UserResponse,
        r#"
        UPDATE users SET
            display_name = COALESCE($1, display_name),
            avatar_url = CASE WHEN $2::boolean THEN $3 ELSE avatar_url END
        WHERE id = $4
        RETURNING id, email, username, display_name, avatar_url, created_at
        "#,
        req.display_name,
        req.avatar_url.is_some() as bool,
        req.avatar_url.flatten(),
        current_user.id,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.new_password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let row = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1",
        current_user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let parsed_hash = PasswordHash::new(&row.password_hash)
        .map_err(|_| AppError::PasswordHash)?;
    Argon2::default()
        .verify_password(req.current_password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Current password is incorrect".to_string()))?;

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        new_hash,
        current_user.id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
