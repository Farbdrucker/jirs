mod common;

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;

fn server(pool: PgPool) -> TestServer {
    common::build_test_app(pool)
}

#[sqlx::test(migrations = "./migrations")]
async fn kanban_returns_5_columns(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s.get("/api/projects/TEST/board").add_header(k, v).await;
    res.assert_status_ok();
    let body: Value = res.json();
    let columns = body["columns"].as_array().unwrap();
    assert_eq!(columns.len(), 5);

    let expected = ["backlog", "todo", "in_progress", "in_review", "done"];
    for (i, col) in columns.iter().enumerate() {
        assert_eq!(col["status"].as_str().unwrap(), expected[i]);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn kanban_empty_columns_present(pool: PgPool) {
    let s = server(pool);
    let (token, _) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let res = s.get("/api/projects/TEST/board").add_header(k, v).await;
    res.assert_status_ok();
    let body: Value = res.json();
    let columns = body["columns"].as_array().unwrap();
    for col in columns {
        assert!(col["tickets"].is_array());
        assert_eq!(col["tickets"].as_array().unwrap().len(), 0);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn scrum_board_active_sprint_only(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;

    let (k, v) = common::auth_header(&token);
    let planning_sprint: Value = s
        .post("/api/projects/TEST/sprints")
        .add_header(k, v)
        .json(&json!({ "name": "Planning Sprint" }))
        .await
        .json();
    let (k, v) = common::auth_header(&token);
    let active_sprint: Value = s
        .post("/api/projects/TEST/sprints")
        .add_header(k, v)
        .json(&json!({ "name": "Active Sprint" }))
        .await
        .json();

    let active_sprint_id = active_sprint["id"].as_str().unwrap();

    // Create a ticket in the planning sprint
    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({
            "title": "Planning ticket",
            "ticket_type": "task",
            "sprint_id": planning_sprint["id"]
        }))
        .await
        .assert_status_ok();

    // Create a ticket in the active sprint
    let (k, v) = common::auth_header(&token);
    s.post(&format!("/api/projects/{key}/tickets"))
        .add_header(k, v)
        .json(&json!({
            "title": "Active sprint ticket",
            "ticket_type": "task",
            "sprint_id": active_sprint_id
        }))
        .await
        .assert_status_ok();

    // Start the active sprint
    let (k, v) = common::auth_header(&token);
    s.patch(&format!("/api/sprints/{active_sprint_id}/start"))
        .add_header(k, v)
        .await
        .assert_status_ok();

    let (k, v) = common::auth_header(&token);
    let board: Value = s
        .get("/api/projects/TEST/board/scrum")
        .add_header(k, v)
        .await
        .json();
    let columns = board["columns"].as_array().unwrap();
    let total_tickets: usize = columns
        .iter()
        .map(|c| c["tickets"].as_array().unwrap().len())
        .sum();
    assert_eq!(total_tickets, 1, "Only active sprint tickets should appear");
}

#[sqlx::test(migrations = "./migrations")]
async fn move_ticket(pool: PgPool) {
    let s = server(pool);
    let (token, key) = common::setup_project(&s).await;
    let slug = common::create_ticket(&s, &token, &key, "Moveable ticket").await;

    let (k, v) = common::auth_header(&token);
    let res = s
        .patch("/api/board/move")
        .add_header(k, v)
        .json(&json!({ "ticket_slug": slug, "to_status": "in_progress" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["status"].as_str().unwrap(), "in_progress");
}

#[sqlx::test(migrations = "./migrations")]
async fn board_non_member(pool: PgPool) {
    let s = server(pool);
    let (_token, _) = common::setup_project(&s).await;

    let other_token = common::register_user(&s, "bob@example.com", "bob").await;
    let (k, v) = common::auth_header(&other_token);
    let res = s.get("/api/projects/TEST/board").add_header(k, v).await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}
