mod db;
mod fetch;
mod lookup;
mod models;
mod routes;
mod stash;

use axum::Router;
use routes::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:lookup.db?mode=rwc".to_string());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let stash_url = std::env::var("STASH_URL").unwrap_or_else(|_| "https://stashdb.org/graphql".to_string());
    let stash = stash::StashClient::new(&stash_url);

    let app = Router::new()
        .route("/health", axum::routing::get(routes::health))
        .route("/api/lookup/{type}/{name}", axum::routing::get(routes::lookup_by_type))
        .route("/api/id/{type}/{id}", axum::routing::get(routes::lookup_by_id))
        .with_state(AppState { pool, stash });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3053));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
