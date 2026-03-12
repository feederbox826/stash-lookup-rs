use crate::db;
use crate::fetch;
use crate::models::{PerformerResponse, StudioResponse, TagResponse};
use crate::stash::{StashClient, StashError};
use sqlx::SqlitePool;

pub async fn performer_by_id(
    pool: &SqlitePool,
    client: &StashClient,
    id: &str,
) -> Result<PerformerResponse, StashError> {
    if let Some(p) = db::lookup_performer_by_id(pool, id).await? {
        return Ok(p);
    }
    let fetched = fetch::get_performer(client, id).await?;
    db::add_performer(pool, &fetched.uuid, &fetched.name, &fetched.aliases).await?;
    Ok(fetched)
}

pub async fn studio_by_id(
    pool: &SqlitePool,
    client: &StashClient,
    id: &str,
) -> Result<StudioResponse, StashError> {
    if let Some(s) = db::lookup_studio_by_id(pool, id).await? {
        return Ok(s);
    }
    let fetched = fetch::get_studio(client, id).await?;
    db::add_studio(pool, &fetched.uuid, &fetched.name, &fetched.aliases, fetched.parent).await?;
    Ok(fetched)
}

pub async fn tag_by_id(
    pool: &SqlitePool,
    client: &StashClient,
    id: &str,
) -> Result<TagResponse, StashError> {
    if let Some(t) = db::lookup_tag_by_id(pool, id).await? {
        return Ok(t);
    }
    let fetched = fetch::get_tag(client, id).await?;
    db::add_tag(pool, &fetched.uuid, &fetched.name, &fetched.aliases, fetched.category).await?;
    Ok(fetched)
}