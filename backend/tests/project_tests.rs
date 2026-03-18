mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn create_project_happy_path(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["key"].as_str().unwrap(), "PROJ");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_project_auto_admin(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let members_res = s
        .get("/api/projects/PROJ/members")
        .add_header(k, v)
        .await;
    members_res.assert_status_ok();
    let members: Value = members_res.json();
    let members = members.as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["role"].as_str().unwrap(), "admin");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_project_key_lowercase(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "proj", "name": "My Project" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_project_key_too_short(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "A", "name": "My Project" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_project_key_too_long(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "TOOLONGKEYY", "name": "My Project" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_project_duplicate_key(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "First" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "Second" }))
        .await;
    res.assert_status(axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_project_non_member(pool: PgPool) {
    let s = server(pool);
    let owner_token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let other_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&other_token);
    let res = s.get("/api/projects/PROJ").add_header(k, v).await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn update_project_as_admin(pool: PgPool) {
    let s = server(pool);
    let token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "Old Name" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let res = s
        .put("/api/projects/PROJ")
        .add_header(k, v)
        .json(&json!({ "name": "New Name" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["name"].as_str().unwrap(), "New Name");
}

#[sqlx::test(migrations = "./migrations")]
async fn update_project_as_viewer(pool: PgPool) {
    let s = server(pool);
    let owner_token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let viewer_token = common::register_user(&s, "bob@example.com", "bob").await;

    // Get bob's ID
    let (k, v) = common::auth_header(&viewer_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let bob_id = me["id"].as_str().unwrap();

    // Owner adds bob as viewer
    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects/PROJ/members")
        .add_header(k, v)
        .json(&json!({ "user_id": bob_id, "role": "viewer" }))
        .await
        .assert_status_ok();

    // Bob tries to update
    let (k, v) = common::auth_header(&viewer_token);
    let res = s
        .put("/api/projects/PROJ")
        .add_header(k, v)
        .json(&json!({ "name": "Hacked Name" }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_projects_filtered_by_membership(pool: PgPool) {
    let s = server(pool);
    let alice_token = common::register_user(&s, "alice@example.com", "alice").await;
    let bob_token = common::register_user(&s, "bob@example.com", "bob").await;

    // Alice creates 2 projects
    let (k, v) = common::auth_header(&alice_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "ALCA", "name": "Alice A" }))
        .await
        .assert_status_ok();
    let (k, v) = common::auth_header(&alice_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "ALCB", "name": "Alice B" }))
        .await
        .assert_status_ok();

    // Bob creates 1 project
    let (k, v) = common::auth_header(&bob_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "BOB", "name": "Bob's Project" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&alice_token);
    let alice_projects: Value = s.get("/api/projects").add_header(k, v).await.json();
    assert_eq!(alice_projects.as_array().unwrap().len(), 2);

    let (k, v) = common::auth_header(&bob_token);
    let bob_projects: Value = s.get("/api/projects").add_header(k, v).await.json();
    assert_eq!(bob_projects.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn add_member_happy_path(pool: PgPool) {
    let s = server(pool);
    let owner_token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let bob_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&bob_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let bob_id = me["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&owner_token);
    let res = s
        .post("/api/projects/PROJ/members")
        .add_header(k, v)
        .json(&json!({ "user_id": bob_id, "role": "member" }))
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn add_member_invalid_role(pool: PgPool) {
    let s = server(pool);
    let owner_token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let bob_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&bob_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let bob_id = me["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&owner_token);
    let res = s
        .post("/api/projects/PROJ/members")
        .add_header(k, v)
        .json(&json!({ "user_id": bob_id, "role": "superuser" }))
        .await;
    res.assert_status_bad_request();
}

#[sqlx::test(migrations = "./migrations")]
async fn remove_member_happy_path(pool: PgPool) {
    let s = server(pool);
    let owner_token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let bob_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&bob_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let bob_id = me["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects/PROJ/members")
        .add_header(k, v)
        .json(&json!({ "user_id": bob_id, "role": "member" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&owner_token);
    let res = s
        .delete(&format!("/api/projects/PROJ/members/{bob_id}"))
        .add_header(k, v)
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn remove_owner(pool: PgPool) {
    let s = server(pool);
    let owner_token = common::register_user(&s, "alice@example.com", "alice").await;

    let (k, v) = common::auth_header(&owner_token);
    s.post("/api/projects")
        .add_header(k, v)
        .json(&json!({ "key": "PROJ", "name": "My Project" }))
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&owner_token);
    let me: Value = s.get("/api/auth/me").add_header(k, v).await.json();
    let owner_id = me["id"].as_str().unwrap();

    let (k, v) = common::auth_header(&owner_token);
    let res = s
        .delete(&format!("/api/projects/PROJ/members/{owner_id}"))
        .add_header(k, v)
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}
