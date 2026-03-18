use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tags::handlers::Tag;

#[derive(Debug, Serialize, Clone)]
pub struct Ticket {
    pub id: Uuid,
    pub slug: String,
    pub ticket_number: i64,
    pub project_id: Uuid,
    pub ticket_type: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub story_points: Option<i32>,
    pub sprint_id: Option<Uuid>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserStub {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TicketSummary {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub ticket_type: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TicketDetail {
    pub id: Uuid,
    pub slug: String,
    pub ticket_number: i64,
    pub project_id: Uuid,
    pub ticket_type: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub story_points: Option<i32>,
    pub sprint_id: Option<Uuid>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Enriched fields
    pub assignee: Option<UserStub>,
    pub reporter: UserStub,
    pub parent: Option<TicketSummary>,
    pub tags: Vec<Tag>,
    pub children: Vec<TicketSummary>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub ticket_type: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub story_points: Option<i32>,
    pub sprint_id: Option<Uuid>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub story_points: Option<i32>,
    pub sprint_id: Option<Uuid>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdateRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignRequest {
    pub assignee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct TicketFilters {
    pub status: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub sprint_id: Option<Uuid>,
    pub ticket_type: Option<String>,
    pub tag_id: Option<Uuid>,
    pub search: Option<String>,
}
