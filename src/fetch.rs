use crate::models::{PerformerResponse, StudioResponse, TagResponse};
use crate::stash::{StashClient, StashError};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct StashPerformer {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Deserialize)]
struct FindPerformerData {
    #[serde(rename = "findPerformer")]
    find_performer: Option<StashPerformer>,
}

pub async fn get_performer(
    client: &StashClient,
    id: &str,
) -> Result<PerformerResponse, StashError> {
    let query = r#"query ($id: ID!) { findPerformer(id: $id) { id name aliases } }"#;
    let vars = serde_json::json!({ "id": id });
    let data: FindPerformerData = client.query(query, Some(vars)).await?;
    let p = data
        .find_performer
        .ok_or_else(|| StashError::NotFound("Performer not found".to_string()))?;
    let uuid = Uuid::parse_str(&p.id)?;
    Ok(PerformerResponse {
        uuid,
        name: p.name,
        aliases: p.aliases,
    })
}

#[derive(Deserialize)]
struct StashStudioParent {
    id: String,
}

#[derive(Deserialize)]
struct StashStudio {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    parent: Option<StashStudioParent>,
}

#[derive(Deserialize)]
struct FindStudioData {
    #[serde(rename = "findStudio")]
    find_studio: Option<StashStudio>,
}

pub async fn get_studio(
    client: &StashClient,
    id: &str,
) -> Result<StudioResponse, StashError> {
    let query = r#"query ($id: ID!) { findStudio(id: $id) { id name aliases parent { id } } }"#;
    let vars = serde_json::json!({ "id": id });
    let data: FindStudioData = client.query(query, Some(vars)).await?;
    let s = data
        .find_studio
        .ok_or_else(|| StashError::NotFound("Studio not found".to_string()))?;
    let uuid = Uuid::parse_str(&s.id)?;
    let parent = s.parent.map(|p| Uuid::parse_str(&p.id)).transpose()?;
    Ok(StudioResponse {
        uuid,
        name: s.name,
        aliases: s.aliases,
        parent,
    })
}

#[derive(Deserialize)]
struct StashTagCategory {
    id: String,
}

#[derive(Deserialize)]
struct StashTag {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    category: Option<StashTagCategory>,
}

#[derive(Deserialize)]
struct FindTagData {
    #[serde(rename = "findTag")]
    find_tag: Option<StashTag>,
}

pub async fn get_tag(
    client: &StashClient,
    id: &str,
) -> Result<TagResponse, StashError> {
    let query = r#"query ($id: ID!) { findTag(id: $id) { id name aliases category { id }} }"#;
    let vars = serde_json::json!({ "id": id });
    let data: FindTagData = client.query(query, Some(vars)).await?;
    let t = data
        .find_tag
        .ok_or_else(|| StashError::NotFound("Tag not found".to_string()))?;
    let uuid = Uuid::parse_str(&t.id)?;
    let category = t.category.map(|p| Uuid::parse_str(&p.id)).transpose()?;
    Ok(TagResponse {
        uuid,
        name: t.name,
        aliases: t.aliases,
        category
    })
}