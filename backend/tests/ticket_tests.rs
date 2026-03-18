mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashSet;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_slug_first(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let slug = common::create_ticket(&s, &token, &key, "First ticket").await;
    assert_eq!(slug, "TEST-1");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_slug_sequential(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let s1 = common::create_ticket(&s, &token, &key, "Ticket 1").await;
    let s2 = common::create_ticket(&s, &token, &key, "Ticket 2").await;
    let s3 = common::create_ticket(&s, &token, &key, "Ticket 3").await;

    assert_eq!(s1, "TEST-1");
    assert_eq!(s2, "TEST-2");
    assert_eq!(s3, "TEST-3");
}

/// Verifies that 10 rapid sequential creates all produce unique slugs.
/// True concurrent verification is covered by the atomic UPDATE...RETURNING in slug.rs.
#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_slug_unique_ten(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let mut slugs = HashSet::new();
    for i in 0..10 {
        let slug = common::create_ticket(&s, &token, &key, &format!("Ticket {i}")).await;
        slugs.insert(slug);
    }

    assert_eq!(slugs.len(), 10, "All 10 slugs must be unique");
    for slug in &slugs {
        assert!(slug.starts_with("TEST-"), "Unexpected slug: {slug}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_invalid_type(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({ "title": "Bad ticket", "ticket_type": "invalid" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_invalid_status(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({ "title": "Bad ticket", "ticket_type": "task", "status": "wip" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_invalid_priority(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({ "title": "Bad ticket", "ticket_type": "task", "priority": "urgent" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_ticket_defaults(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({ "title": "Default ticket", "ticket_type": "task" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["status"].as_str().unwrap(), "backlog");
    assert_eq!(body["priority"].as_str().unwrap(), "medium");
}

#[sqlx::test(migrations = "./migrations")]
async fn get_ticket_enriched(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Enriched ticket").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/tickets/{slug}"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["reporter"].is_object());
    assert!(body["tags"].is_array());
    assert!(body["children"].is_array());
    assert_eq!(body["tags"].as_array().unwrap().len(), 0);
    assert_eq!(body["children"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_ticket_with_assignee(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let user_id = me["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    let res = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({
            "title": "Assigned ticket",
            "ticket_type": "task",
            "assignee_id": user_id
        }))
        .await;
    res.assert_status_ok();
    let ticket: Value = res.json();
    let slug = ticket["slug"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    let detail: Value = s
        .get(&format!("/api/tickets/{slug}"))
        .add_header(k, v)
        .await
        .json();
    assert!(detail["assignee"].is_object());
    assert_eq!(detail["assignee"]["id"].as_str().unwrap(), user_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_ticket_with_parent(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let parent: Value = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({ "title": "Parent epic", "ticket_type": "epic" }))
        .await
        .json();
    let parent_id = parent["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    let child: Value = s
        .post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({
            "title": "Child story",
            "ticket_type": "story",
            "parent_id": parent_id
        }))
        .await
        .json();
    let child_slug = child["slug"].as_str().unwrap();

    let (k, v) = common::auth_header(&token);
    let detail: Value = s
        .get(&format!("/api/tickets/{child_slug}"))
        .add_header(k, v)
        .await
        .json();
    assert!(detail["parent"].is_object());
    assert_eq!(detail["parent"]["id"].as_str().unwrap(), parent_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_ticket_non_member(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Private ticket").await;

    let other_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&other_token);
    let res = s
        .get(&format!("/api/tickets/{slug}"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_ticket_not_found(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s.get("/api/tickets/NOPE-999").add_header(k, v).await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_tickets_filter_status(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let t1 = common::create_ticket(&s, &token, &key, "Done ticket").await;
    let _t2 = common::create_ticket(&s, &token, &key, "Backlog ticket").await;

    // Move t1 to done
    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/tickets/{t1}/status"))
        .add_header(k, v)
        .json(&json!({ "status": "done" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/projects/{key}/tickets?status=done"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let tickets: Value = res.json();
    let arr = tickets.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["status"].as_str().unwrap(), "done");
}

#[sqlx::test(migrations = "./migrations")]
async fn list_tickets_search(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    common::create_ticket(&s, &token, &key, "Important feature").await;
    common::create_ticket(&s, &token, &key, "Minor bug fix").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .get(&format!("/api/projects/{key}/tickets?search=Important"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
    let tickets: Value = res.json();
    let arr = tickets.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert!(arr[0]["title"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("important"));
}

#[sqlx::test(migrations = "./migrations")]
async fn patch_status_valid(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Status test").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/tickets/{slug}/status"))
        .add_header(k, v)
        .json(&json!({ "status": "in_progress" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["status"].as_str().unwrap(), "in_progress");
}

#[sqlx::test(migrations = "./migrations")]
async fn patch_status_invalid(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Status test").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch(&format!("/api/tickets/{slug}/status"))
        .add_header(k, v)
        .json(&json!({ "status": "archived" }))
        .await;
    res.assert_status_bad_request();
}
