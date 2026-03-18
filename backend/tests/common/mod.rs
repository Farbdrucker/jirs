use axum::http::{HeaderName, HeaderValue};
use axum_test::TestServer;
use jirs::{auth::jwt::create_access_token, config::Config, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub fn build_test_app(pool: PgPool) -> TestServer {
    let config = Config {
        database_url: String::new(),
        jwt_secret: "test_secret".into(),
        jwt_refresh_secret: "test_refresh".into(),
        port: 0,
    };
    let state = AppState { pool, config };
    let router = jirs::build_router(state);
    TestServer::new(router)
}

pub fn auth_header(token: &str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    )
}

/// Register a user and return their access_token.
pub async fn register_user(server: &TestServer, email: &str, username: &str) -> String {
    let res = server
        .post("/api/auth/register")
        .json(&json!({
            "email": email,
            "username": username,
            "display_name": username,
            "password": "password123"
        }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    body["access_token"].as_str().unwrap().to_string()
}

/// Register a user, create project with key "TEST", return (access_token, "TEST").
pub async fn setup_project(server: &TestServer) -> (String, String) {
    let token = register_user(server, "owner@example.com", "owner").await;
    let (k, v) = auth_header(&token);
    let res = server
        .post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "TEST", "name": "Test Project" }))
        .await;
    res.assert_status_ok();
    (token, "TEST".to_string())
}

/// Create a task ticket and return its slug.
pub async fn create_ticket(server: &TestServer, token: &str, key: &str, title: &str) -> String {
    let (k, v) = auth_header(token);
    let res = server
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({ "title": title, "ticket_type": "task" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    body["slug"].as_str().unwrap().to_string()
}

/// Create a Bearer token directly from a user_id without hitting the DB.
pub fn bearer(user_id: Uuid) -> String {
    let token = create_access_token(user_id, "test_secret").unwrap();
    format!("Bearer {}", token)
}
