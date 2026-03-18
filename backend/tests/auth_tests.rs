mod common;

use axum::http::{HeaderName, HeaderValue};
use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn register_happy_path(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert!(body["user"]["id"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn register_email_stored_lowercase(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "ALICE@EXAMPLE.COM",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["user"]["email"].as_str().unwrap(), "alice@example.com");
}

#[sqlx::test(migrations = "./migrations")]
async fn register_empty_email(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn register_empty_password(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": ""
        }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn register_short_password(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "abc12"
        }))
        .await;
    res.assert_status_bad_request();
    let body: Value = res.json();
    let error = body["error"].as_str().unwrap_or("");
    assert!(error.contains("6"), "Expected '6 characters' message, got: {error}");
}

#[sqlx::test(migrations = "./migrations")]
async fn register_exactly_6_chars(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "abc123"
        }))
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_email(pool: PgPool) {
    let s = server(pool);
    s.post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .assert_status_ok();

    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice2",
            "display_name": "Alice2",
            "password": "secret123"
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_username(pool: PgPool) {
    let s = server(pool);
    s.post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .assert_status_ok();

    let res = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice2@example.com",
            "username": "alice",
            "display_name": "Alice2",
            "password": "secret123"
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_happy_path(pool: PgPool) {
    let s = server(pool);
    s.post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .assert_status_ok();

    let res = s
        .post("/api/auth/login")
        .json(&json!({ "email": "alice@example.com", "password": "secret123" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_wrong_password(pool: PgPool) {
    let s = server(pool);
    s.post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .assert_status_ok();

    let res = s
        .post("/api/auth/login")
        .json(&json!({ "email": "alice@example.com", "password": "wrongpassword" }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_unknown_email(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/login")
        .json(&json!({ "email": "nobody@example.com", "password": "secret123" }))
        .await;
    // Must return 401 — must not leak whether the user exists
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_email_case_insensitive(pool: PgPool) {
    let s = server(pool);
    s.post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .assert_status_ok();

    let res = s
        .post("/api/auth/login")
        .json(&json!({ "email": "ALICE@EXAMPLE.COM", "password": "secret123" }))
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_happy_path(pool: PgPool) {
    let s = server(pool);
    let reg: Value = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .json();
    let refresh_token = reg["refresh_token"].as_str().unwrap();

    let res = s
        .post("/api/auth/refresh")
        .json(&json!({ "refresh_token": refresh_token }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_with_access_token(pool: PgPool) {
    let s = server(pool);
    let reg: Value = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .json();
    let access_token = reg["access_token"].as_str().unwrap();

    let res = s
        .post("/api/auth/refresh")
        .json(&json!({ "refresh_token": access_token }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    let body: Value = res.json();
    let error = body["error"].as_str().unwrap_or("");
    assert!(error.contains("Invalid token type"), "Got: {error}");
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_invalid_token(pool: PgPool) {
    let s = server(pool);
    let res = s
        .post("/api/auth/refresh")
        .json(&json!({ "refresh_token": "not.a.valid.token" }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn me_authenticated(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s.get("/api/auth/me").add_header(k, v).await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["id"].is_string());
    assert_eq!(body["email"].as_str().unwrap(), "alice@example.com");
}

#[sqlx::test(migrations = "./migrations")]
async fn me_no_token(pool: PgPool) {
    let s = server(pool);
    let res = s.get("/api/auth/me").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn me_refresh_token_as_access(pool: PgPool) {
    let s = server(pool);
    let reg: Value = s
        .post("/api/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "username": "alice",
            "display_name": "Alice",
            "password": "secret123"
        }))
        .await
        .json();
    let refresh_token = reg["refresh_token"].as_str().unwrap();

    let res = s
        .get("/api/auth/me")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", refresh_token)).unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}
