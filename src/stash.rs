//! Lightweight GraphQL client for Stash API.
//!
//! # Example
//!
//! ```ignore
//! let client = stash::StashClient::new("https://your-stash/graphql");
//! let query = r#"query { performer(id: $id) { name aliases } }"#;
//! let vars = serde_json::json!({ "id": "uuid-here" });
//! let data: PerformerData = client.query(query, Some(vars)).await?;
//! ```

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

// stash-box gql client
#[derive(Clone)]
pub struct StashClient {
    client: Client,
    base_url: String,
    apikey: String,
}

impl StashClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            apikey: std::env::var("STASH_APIKEY").unwrap_or_default(),
        }
    }

    pub async fn query<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T, StashError> {
        let body = match variables {
            Some(v) => serde_json::json!({ "query": query, "variables": v }),
            None => serde_json::json!({ "query": query }),
        };

        let resp = self
            .client
            .post(&self.base_url)
            .header("ApiKey", &self.apikey)
            .json(&body)
            .send()
            .await
            .map_err(|e| StashError::Request(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(StashError::Request(format!("HTTP {}", resp.status())));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| StashError::Request(e.to_string()))?;

        if let Some(errors) = json.get("errors") {
            let msg = errors
                .as_array()
                .and_then(|a| a.first())
                .and_then(|e| e.get("message").and_then(|m| m.as_str()))
                .unwrap_or("GraphQL error");
            return Err(StashError::GraphQL(msg.to_string()));
        }

        let data = json
            .get("data")
            .ok_or_else(|| StashError::GraphQL("Missing 'data' in response".to_string()))?;
        Ok(serde_json::from_value(data.clone())?)
    }
}

#[derive(Debug)]
pub enum StashError {
    Request(String),
    GraphQL(String),
    Parse(String),
    NotFound(String),
}

impl std::fmt::Display for StashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StashError::Request(s) => write!(f, "Request failed: {}", s),
            StashError::GraphQL(s) => write!(f, "GraphQL error: {}", s),
            StashError::Parse(s) => write!(f, "Parse error: {}", s),
            StashError::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for StashError {}

impl From<sqlx::Error> for StashError {
    fn from(e: sqlx::Error) -> Self {
        StashError::Request(e.to_string())
    }
}

impl From<uuid::Error> for StashError {
    fn from(e: uuid::Error) -> Self {
        StashError::Parse(e.to_string())
    }
}

impl From<serde_json::Error> for StashError {
    fn from(e: serde_json::Error) -> Self {
        StashError::Parse(e.to_string())
    }
}
