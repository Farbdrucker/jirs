mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn create_comment_happy_path(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Commented ticket").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug}/comments"))
        .add_header(k, v)
        .json(&json!({ "body": "This is a comment" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["body"].as_str().unwrap(), "This is a comment");
    assert!(body["author_id"].is_string());
    assert!(body["author_username"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn create_comment_empty_body(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug}/comments"))
        .add_header(k, v)
        .json(&json!({ "body": "   " }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_comment_non_member(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket").await;

    let other_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&other_token);
    let res = s
        .post(&format!("/api/tickets/{slug}/comments"))
        .add_header(k, v)
        .json(&json!({ "body": "Intruder comment" }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn update_comment_wrong_user(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket").await;

    // Create a comment as alice (the owner)
    let (k, v) = common::auth_header(&token);
    let comment: Value = s
        .post(&format!("/api/tickets/{slug}/comments"))
        .add_header(k, v)
        .json(&json!({ "body": "Alice's comment" }))
        .await
        .json();
    let comment_id = comment["id"].as_str().unwrap();

    // Add bob as member so he can access the ticket
    let bob_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&bob_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let bob_id = me["id"].as_str().unwrap();
    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/projects/{key}/members"))
        .add_header(k, v)
        .json(&json!({ "user_id": bob_id, "role": "member" }))
        .await
        .assert_status_ok();

    // Bob tries to update Alice's comment — should 404 (WHERE author_id = bob not matching)
    let (k, v) = common::auth_header(&bob_token);
    let res = s
        .put(&format!("/api/comments/{comment_id}"))
        .add_header(k, v)
        .json(&json!({ "body": "Bob modified it" }))
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_comment_wrong_user(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket").await;

    let (k, v) = common::auth_header(&token);
    let comment: Value = s
        .post(&format!("/api/tickets/{slug}/comments"))
        .add_header(k, v)
        .json(&json!({ "body": "Alice's comment" }))
        .await
        .json();
    let comment_id = comment["id"].as_str().unwrap();

    let bob_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&bob_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let bob_id = me["id"].as_str().unwrap();
    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/projects/{key}/members"))
        .add_header(k, v)
        .json(&json!({ "user_id": bob_id, "role": "member" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&bob_token);
    let res = s
        .delete(&format!("/api/comments/{comment_id}"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}
