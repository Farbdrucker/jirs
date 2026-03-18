pub mod handlers;

use sqlx::PgPool;
use uuid::Uuid;

pub async fn log(
    pool: &PgPool,
    ticket_id: Uuid,
    actor_id: Uuid,
    action: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO activity (ticket_id, actor_id, action, old_value, new_value) VALUES ($1, $2, $3, $4, $5)",
        ticket_id,
        actor_id,
        action,
        old_value,
        new_value,
    )
    .execute(pool)
    .await?;
    Ok(())
}
