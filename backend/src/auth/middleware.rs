use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{
    auth::jwt::verify_token,
    error::AppError,
    AppState,
};

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: Uuid,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(&request)?;
    let claims = verify_token(&token, &state.config.jwt_secret)?;
    if claims.token_type != "access" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }
    request.extensions_mut().insert(CurrentUser { id: claims.sub });
    Ok(next.run(request).await)
}

fn extract_token(request: &Request) -> Result<String, AppError> {
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(value) = auth_header.to_str() {
            if let Some(token) = value.strip_prefix("Bearer ") {
                return Ok(token.to_string());
            }
        }
    }
    Err(AppError::Unauthorized("Missing or invalid Authorization header".to_string()))
}
