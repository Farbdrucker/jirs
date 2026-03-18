mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn create_tag_default_color(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "bug" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["color"].as_str().unwrap(), "#6366f1");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_tag_custom_color(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "urgent", "color": "#ff0000" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["color"].as_str().unwrap(), "#ff0000");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_tag_duplicate_name_same_project(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    s.post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "bug" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "bug" }))
        .await;
    res.assert_status(axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_tag_same_name_different_project(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "Other Project" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    s.post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "bug" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects/PROJ/tags")
        .add_header(k, v)
        .json(&json!({ "name": "bug" }))
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn add_tag_idempotent(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Tagged ticket").await;

    let (k, v) = common::auth_header(&token);
    let tag: Value = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "feature" }))
        .await
        .json();
    let tag_id = tag["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/tickets/{slug}/tags/{tag_id}"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/tickets/{slug}/tags/{tag_id}"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let tags: Value = s
        .get(&format!("/api/tickets/{slug}/tags"))
        .add_header(k, v)
        .await
        .json();
    assert_eq!(tags.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn remove_tag(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let tag: Value = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "removable" }))
        .await
        .json();
    let tag_id = tag["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.delete(&format!("/api/projects/TEST/tags/{tag_id}"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let tags: Value = s
        .get("/api/projects/TEST/tags")
        .add_header(k, v)
        .await
        .json();
    assert_eq!(tags.as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_ticket_tags(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Tagged ticket").await;

    let (k, v) = common::auth_header(&token);
    let tag: Value = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "feature" }))
        .await
        .json();
    let tag_id = tag["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/tickets/{slug}/tags/{tag_id}"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/tickets/{slug}/tags"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let tags: Value = res.json();
    let arr = tags.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"].as_str().unwrap(), "feature");
}

#[sqlx::test(migrations = "./migrations")]
async fn ticket_detail_includes_tags(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Tagged ticket").await;

    let (k, v) = common::auth_header(&token);
    let tag: Value = s
        .post("/api/projects/TEST/tags")
        .add_header(k, v)
        .json(&json!({ "name": "feature" }))
        .await
        .json();
    let tag_id = tag["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/tickets/{slug}/tags/{tag_id}"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let detail: Value = s
        .get(&format!("/api/tickets/{slug}"))
        .add_header(k, v)
        .await
        .json();
    let tags = detail["tags"].as_array().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0]["name"].as_str().unwrap(), "feature");
}
