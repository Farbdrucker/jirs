mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_display_name(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .put("/api/users/me")
        .add_header(k, v)
        .json(&json!({ "display_name": "Alice Wonderland" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["display_name"].as_str().unwrap(), "Alice Wonderland");
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_set_avatar(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .put("/api/users/me")
        .add_header(k, v)
        .json(&json!({ "avatar_url": "https://example.com/avatar.png" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(
        body["avatar_url"].as_str().unwrap(),
        "https://example.com/avatar.png"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_clear_avatar(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    // First set an avatar
    let (k, v) = common::auth_header(&token);
    s.put("/api/users/me")
        .add_header(k, v)
        .json(&json!({ "avatar_url": "https://example.com/avatar.png" }))
        .await
        .assert_status_ok();

    // Then clear it
    let (k, v) = common::auth_header(&token);
    let res = s
        .put("/api/users/me")
        .add_header(k, v)
        .json(&json!({ "avatar_url": null }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["avatar_url"].is_null());
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_empty_body(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .put("/api/users/me")
        .add_header(k, v)
        .json(&json!({}))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    // display_name should be unchanged (COALESCE preserves existing value)
    assert_eq!(body["display_name"].as_str().unwrap(), "alice");
}

#[sqlx::test(migrations = "./migrations")]
async fn change_password_happy_path(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/users/me/password")
        .add_header(k, v)
        .json(&json!({
            "current_password": "password123",
            "new_password": "newpassword456"
        }))
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn change_password_wrong_current(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/users/me/password")
        .add_header(k, v)
        .json(&json!({
            "current_password": "wrongpassword",
            "new_password": "newpassword456"
        }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn change_password_then_login(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    s.post("/api/users/me/password")
        .add_header(k, v)
        .json(&json!({
            "current_password": "password123",
            "new_password": "newpassword456"
        }))
        .await
        .assert_status_ok();

    let res = s
        .post("/api/auth/login")
        .json(&json!({ "email": "alice@example.com", "password": "newpassword456" }))
        .await;
    res.assert_status_ok();
}
