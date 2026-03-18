use uuid::Uuid;

use crate::error::AppResult;

/// Atomically increment the project ticket counter and return the new slug.
pub async fn next_slug(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    project_key: &str,
) -> AppResult<(String, i64)> {
    let counter = sqlx::query_scalar!(
        "UPDATE project_ticket_counter SET counter = counter + 1 WHERE project_id = $1 RETURNING counter",
        project_id
    )
    .fetch_one(&mut **tx)
    .await?;

    let slug = format!("{}-{}", project_key, counter);
    Ok((slug, counter))
}
