mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

async fn create_sprint(s: &TestServer, token: &str, name: &str) -> Value {
    let (k, v) = common::auth_header(token);
    s.post("/api/projects/TEST/sprints")
        .add_header(k, v)
        .json(&json!({ "name": name }))
        .await
        .json()
}

#[sqlx::test(migrations = "./migrations")]
async fn create_sprint_defaults_to_planning(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let sprint = create_sprint(&s, &token, "Sprint 1").await;
    assert_eq!(sprint["status"].as_str().unwrap(), "planning");
}

#[sqlx::test(migrations = "./migrations")]
async fn start_sprint(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let sprint = create_sprint(&s, &token, "Sprint 1").await;
    let sprint_id = sprint["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/sprints/{sprint_id}/start"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["status"].as_str().unwrap(), "active");
}

#[sqlx::test(migrations = "./migrations")]
async fn start_second_sprint_conflicts(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let s1 = create_sprint(&s, &token, "Sprint 1").await;
    let s2 = create_sprint(&s, &token, "Sprint 2").await;
    let id1 = s1["id"].as_str().unwrap();
    let id2 = s2["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/sprints/{id1}/start"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/sprints/{id2}/start"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn complete_sprint(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let sprint = create_sprint(&s, &token, "Sprint 1").await;
    let sprint_id = sprint["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/sprints/{sprint_id}/start"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/sprints/{sprint_id}/complete"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["status"].as_str().unwrap(), "completed");
}

#[sqlx::test(migrations = "./migrations")]
async fn complete_then_start_new(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let s1 = create_sprint(&s, &token, "Sprint 1").await;
    let s2 = create_sprint(&s, &token, "Sprint 2").await;
    let id1 = s1["id"].as_str().unwrap();
    let id2 = s2["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/sprints/{id1}/start"))
        .add_header(k, v)
        .await
        .assert_status_ok();
    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/sprints/{id1}/complete"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/sprints/{id2}/start"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn sprint_not_found(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let fake_id = Uuid::new_v4();
    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/sprints/{fake_id}/start"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn sprint_non_member(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let sprint = create_sprint(&s, &token, "Sprint 1").await;
    let sprint_id = sprint["id"].as_str().unwrap();

    let other_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&other_token);
    let res = s
        .patch(&format!("/api/sprints/{sprint_id}/start"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}
