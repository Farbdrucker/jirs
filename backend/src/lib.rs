pub mod activity;
pub mod auth;
pub mod boards;
pub mod comments;
pub mod config;
pub mod db;
pub mod error;
pub mod links;
pub mod projects;
pub mod sprints;
pub mod tags;
pub mod tickets;
pub mod users;

use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use config::Config;
use sqlx::PgPool;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        // Auth
        .route("/api/auth/me", get(auth::handlers::me))
        // Users
        .route("/api/users", get(users::handlers::list_users))
        .route("/api/users/me", put(users::handlers::update_profile))
        .route("/api/users/me/password", post(users::handlers::change_password))
        // Projects
        .route("/api/projects", get(projects::handlers::list_projects))
        .route("/api/projects", post(projects::handlers::create_project))
        .route("/api/projects/{key}", get(projects::handlers::get_project))
        .route("/api/projects/{key}", put(projects::handlers::update_project))
        .route("/api/projects/{key}/members", get(projects::handlers::get_members))
        .route("/api/projects/{key}/members", post(projects::handlers::add_member))
        .route("/api/projects/{key}/members/{user_id}", delete(projects::handlers::remove_member))
        // Tickets
        .route("/api/projects/{key}/tickets", get(tickets::handlers::list_tickets))
        .route("/api/projects/{key}/tickets", post(tickets::handlers::create_ticket))
        .route("/api/tickets/{slug}", get(tickets::handlers::get_ticket))
        .route("/api/tickets/{slug}", put(tickets::handlers::update_ticket))
        .route("/api/tickets/{slug}", delete(tickets::handlers::delete_ticket))
        .route("/api/tickets/{slug}/status", patch(tickets::handlers::patch_status))
        .route("/api/tickets/{slug}/assign", patch(tickets::handlers::patch_assign))
        .route("/api/tickets/{slug}/children", get(tickets::handlers::get_children))
        .route("/api/tickets/{slug}/tags", get(tags::handlers::get_ticket_tags))
        // Comments
        .route("/api/tickets/{slug}/comments", get(comments::handlers::list_comments))
        .route("/api/tickets/{slug}/comments", post(comments::handlers::create_comment))
        .route("/api/comments/{id}", put(comments::handlers::update_comment))
        .route("/api/comments/{id}", delete(comments::handlers::delete_comment))
        // Tags
        .route("/api/projects/{key}/tags", get(tags::handlers::list_tags))
        .route("/api/projects/{key}/tags", post(tags::handlers::create_tag))
        .route("/api/projects/{key}/tags/{id}", delete(tags::handlers::delete_tag))
        .route("/api/tickets/{slug}/tags/{tag_id}", post(tags::handlers::add_tag_to_ticket))
        .route("/api/tickets/{slug}/tags/{tag_id}", delete(tags::handlers::remove_tag_from_ticket))
        // Links
        .route("/api/tickets/{slug}/links", post(links::handlers::create_link))
        .route("/api/tickets/{slug}/links/{id}", delete(links::handlers::delete_link))
        .route("/api/tickets/{slug}/repos", post(links::handlers::create_repo_link))
        .route("/api/tickets/{slug}/repos/{id}", delete(links::handlers::delete_repo_link))
        // Sprints
        .route("/api/projects/{key}/sprints", get(sprints::handlers::list_sprints))
        .route("/api/projects/{key}/sprints", post(sprints::handlers::create_sprint))
        .route("/api/sprints/{id}", put(sprints::handlers::update_sprint))
        .route("/api/sprints/{id}/start", patch(sprints::handlers::start_sprint))
        .route("/api/sprints/{id}/complete", patch(sprints::handlers::complete_sprint))
        // Boards
        .route("/api/projects/{key}/board", get(boards::handlers::get_kanban_board))
        .route("/api/projects/{key}/board/scrum", get(boards::handlers::get_scrum_board))
        .route("/api/board/move", patch(boards::handlers::move_ticket))
        // Activity
        .route("/api/tickets/{slug}/activity", get(activity::handlers::get_activity))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::auth_middleware,
        ));

    let public = Router::new()
        .route("/api/auth/register", post(auth::handlers::register))
        .route("/api/auth/login", post(auth::handlers::login))
        .route("/api/auth/refresh", post(auth::handlers::refresh));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(CompressionLayer::new())
        .with_state(state)
}
