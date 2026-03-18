mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn status_change_logs_activity(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Activity ticket").await;

    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/tickets/{slug}/status"))
        .add_header(k, v)
        .json(&json!({ "status": "in_progress" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/tickets/{slug}/activity"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let entries: Value = res.json();
    let arr = entries.as_array().unwrap();
    assert!(!arr.is_empty(), "Expected at least one activity entry");

    let entry = arr.iter().find(|e| e["action"] == "status_changed");
    assert!(entry.is_some(), "Expected status_changed action in activity");
    let entry = entry.unwrap();
    assert_eq!(entry["old_value"].as_str().unwrap(), "backlog");
    assert_eq!(entry["new_value"].as_str().unwrap(), "in_progress");
}

#[sqlx::test(migrations = "./migrations")]
async fn assignee_change_logs_activity(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Activity ticket").await;

    let (k, v) = common::auth_header(&token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let user_id = me["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/tickets/{slug}/assign"))
        .add_header(k, v)
        .json(&json!({ "assignee_id": user_id }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/tickets/{slug}/activity"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let entries: Value = res.json();
    let arr = entries.as_array().unwrap();

    let entry = arr.iter().find(|e| e["action"] == "assignee_changed");
    assert!(entry.is_some(), "Expected assignee_changed action in activity");
}

#[sqlx::test(migrations = "./migrations")]
async fn activity_empty_for_new_ticket(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Fresh ticket").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/tickets/{slug}/activity"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let entries: Value = res.json();
    assert_eq!(entries.as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn activity_non_member(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket").await;

    let other_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&other_token);
    let res = s
        .get(&format!("/api/tickets/{slug}/activity"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}
