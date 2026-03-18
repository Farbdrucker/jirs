use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{
        jwt::{create_access_token, create_refresh_token, verify_token},
        middleware::CurrentUser,
    },
    error::{AppError, AppResult},
    users::models::UserResponse,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    if req.email.is_empty() || req.password.is_empty() || req.username.is_empty() {
        return Err(AppError::BadRequest("All fields are required".to_string()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    let user = sqlx::query_as!(
        UserResponse,
        r#"
        INSERT INTO users (email, username, display_name, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, username, display_name, avatar_url, created_at
        "#,
        req.email.to_lowercase(),
        req.username,
        req.display_name,
        hash,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint().is_some() {
                return AppError::Conflict("Email or username already exists".to_string());
            }
        }
        AppError::Database(e)
    })?;

    let access_token = create_access_token(user.id, &state.config.jwt_secret)?;
    let refresh_token = create_refresh_token(user.id, &state.config.jwt_refresh_secret)?;

    Ok(Json(AuthResponse { access_token, refresh_token, user }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let row = sqlx::query!(
        r#"SELECT id, email, username, display_name, avatar_url, created_at, password_hash
           FROM users WHERE email = $1"#,
        req.email.to_lowercase()
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let parsed_hash = PasswordHash::new(&row.password_hash)
        .map_err(|_| AppError::PasswordHash)?;
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let user = UserResponse {
        id: row.id,
        email: row.email,
        username: row.username,
        display_name: row.display_name,
        avatar_url: row.avatar_url,
        created_at: row.created_at,
    };

    let access_token = create_access_token(user.id, &state.config.jwt_secret)?;
    let refresh_token = create_refresh_token(user.id, &state.config.jwt_refresh_secret)?;

    Ok(Json(AuthResponse { access_token, refresh_token, user }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = verify_token(&req.refresh_token, &state.config.jwt_refresh_secret)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }
    let access_token = create_access_token(claims.sub, &state.config.jwt_secret)?;
    let refresh_token = create_refresh_token(claims.sub, &state.config.jwt_refresh_secret)?;
    Ok(Json(serde_json::json!({ "access_token": access_token, "refresh_token": refresh_token })))
}

pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Json<UserResponse>> {
    let user = sqlx::query_as!(
        UserResponse,
        "SELECT id, email, username, display_name, avatar_url, created_at FROM users WHERE id = $1",
        current_user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}
