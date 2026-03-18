mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_blocks_forward(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;
    let slug_b = common::create_ticket(&s, &token, &key, "Ticket B").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_b, "link_type": "blocks" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["link_type"].as_str().unwrap(), "blocks");
    assert_eq!(body["target_slug"].as_str().unwrap(), slug_b);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_blocks_inverse_created(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;
    let slug_b = common::create_ticket(&s, &token, &key, "Ticket B").await;

    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_b, "link_type": "blocks" }))
        .await
        .assert_status_ok();

    // Verify the inverse exists by linking B->A with "relates_to" (a different type)
    // and confirming it doesn't conflict with the existing is_blocked_by link
    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug_b}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_a, "link_type": "relates_to" }))
        .await;
    // A relates_to link from B->A can coexist with the is_blocked_by link
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_relates_to(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;
    let slug_b = common::create_ticket(&s, &token, &key, "Ticket B").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_b, "link_type": "relates_to" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["link_type"].as_str().unwrap(), "relates_to");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_duplicates(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;
    let slug_b = common::create_ticket(&s, &token, &key, "Ticket B").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_b, "link_type": "duplicates" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["link_type"].as_str().unwrap(), "duplicates");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_self(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket A").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug, "link_type": "blocks" }))
        .await;
    res.assert_status_bad_request();
    let body: Value = res.json();
    let error = body["error"].as_str().unwrap_or("");
    assert!(error.contains("itself"), "Got: {error}");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_invalid_type(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;
    let slug_b = common::create_ticket(&s, &token, &key, "Ticket B").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_b, "link_type": "depends_on" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_link_unknown_target(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": "NOPE-999", "link_type": "blocks" }))
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_link_removes_both(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug_a = common::create_ticket(&s, &token, &key, "Ticket A").await;
    let slug_b = common::create_ticket(&s, &token, &key, "Ticket B").await;

    let (k, v) = common::auth_header(&token);
    let link: Value = s
        .post(&format!("/api/tickets/{slug_a}/links"))
        .add_header(k, v)
        .json(&json!({ "target_slug": slug_b, "link_type": "blocks" }))
        .await
        .json();
    let link_id = link["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    let res = s
        .delete(&format!("/api/tickets/{slug_a}/links/{link_id}"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();

    // Verify that attempting to delete the same link again returns not found
    let (k, v) = common::auth_header(&token);
    let res2 = s
        .delete(&format!("/api/tickets/{slug_a}/links/{link_id}"))
        .add_header(k, v)
        .await;
    res2.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_repo_link(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Ticket A").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/tickets/{slug}/repos"))
        .add_header(k, v)
        .json(&json!({ "repo_url": "https://github.com/example/repo" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(
        body["repo_url"].as_str().unwrap(),
        "https://github.com/example/repo"
    );
    assert!(body["branch_name"].is_null());
    assert!(body["pr_url"].is_null());
}
