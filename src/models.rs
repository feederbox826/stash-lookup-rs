use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub uuid: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub category: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct StudioResponse {
    pub uuid: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub parent: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct PerformerResponse {
    pub uuid: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
}
