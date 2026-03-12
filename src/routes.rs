use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;

use crate::db;
use crate::lookup;
use crate::stash::{StashClient, StashError};
use uuid::Uuid;

const MAX_NAME_LEN: usize = 256;

pub async fn health() -> &'static str {
    "ok"
}

#[derive(Clone)]
#[must_use]
pub struct AppState {
    pub pool: SqlitePool,
    pub stash: StashClient,
}

fn error_response(status: StatusCode, message: &str) -> axum::response::Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}

pub async fn lookup_by_type(
    State(state): State<AppState>,
    Path((entity_type, name)): Path<(String, String)>,
) -> axum::response::Response {
    if name.len() > MAX_NAME_LEN {
        return error_response(
            StatusCode::BAD_REQUEST,
            &format!("Name must be at most {} characters", MAX_NAME_LEN),
        );
    }

    let result = match entity_type.to_lowercase().as_str() {
        "tags" => db::lookup_tags_by_name(&state.pool, &name).await.map(LookupResponse::Tags),
        "studios" => db::lookup_studios_by_name(&state.pool, &name).await.map(LookupResponse::Studios),
        "performers" => db::lookup_performers_by_name(&state.pool, &name).await.map(LookupResponse::Performers),
        _ => return error_response(StatusCode::BAD_REQUEST, "Invalid entity type. Use: tags, studios, performers"),
    };

    match result {
        Ok(resp) if resp.is_empty() => error_response(StatusCode::NOT_FOUND, resp.empty_message()),
        Ok(resp) => resp.into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn lookup_by_id(
    State(state): State<AppState>,
    Path((entity_type, id)): Path<(String, String)>,
) -> axum::response::Response {
    if Uuid::parse_str(&id).is_err() {
        return error_response(StatusCode::BAD_REQUEST, "Invalid UUID format");
    }

    let result = match entity_type.to_lowercase().as_str() {
        "tags" => lookup::tag_by_id(&state.pool, &state.stash, &id)
            .await
            .map(|t| LookupResponse::Tags(vec![t])),
        "studios" => lookup::studio_by_id(&state.pool, &state.stash, &id)
            .await
            .map(|s| LookupResponse::Studios(vec![s])),
        "performers" => lookup::performer_by_id(&state.pool, &state.stash, &id)
            .await
            .map(|p| LookupResponse::Performers(vec![p])),
        _ => return error_response(StatusCode::BAD_REQUEST, "Invalid entity type. Use: tags, studios, performers"),
    };

    match result {
        Ok(resp) => resp.into_response(),
        Err(e) => {
            let (status, msg) = match &e {
                StashError::NotFound(s) => (StatusCode::NOT_FOUND, s.clone()),
                StashError::GraphQL(s) if s.to_lowercase().contains("not found") => (StatusCode::NOT_FOUND, s.clone()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            };
            error_response(status, &msg)
        }
    }
}

enum LookupResponse {
    Tags(Vec<crate::models::TagResponse>),
    Studios(Vec<crate::models::StudioResponse>),
    Performers(Vec<crate::models::PerformerResponse>),
}

impl LookupResponse {
    fn is_empty(&self) -> bool {
        match self {
            LookupResponse::Tags(v) => v.is_empty(),
            LookupResponse::Studios(v) => v.is_empty(),
            LookupResponse::Performers(v) => v.is_empty(),
        }
    }
    fn empty_message(&self) -> &'static str {
        match self {
            LookupResponse::Tags(_) => "No tags found",
            LookupResponse::Studios(_) => "No studios found",
            LookupResponse::Performers(_) => "No performers found",
        }
    }
}

impl IntoResponse for LookupResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            LookupResponse::Tags(v) => (StatusCode::OK, Json(v)).into_response(),
            LookupResponse::Studios(v) => (StatusCode::OK, Json(v)).into_response(),
            LookupResponse::Performers(v) => (StatusCode::OK, Json(v)).into_response(),
        }
    }
}
