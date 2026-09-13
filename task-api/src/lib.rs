use anyhow::bail;
use axum::Router;
use axum::routing::{get, post};
use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::{http_service};
use tower_service::Service;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::models::config::AppConfig;

mod handlers;
mod models;
mod services;
mod task_repository;

spin_sdk::dependencies!();

#[http_service]
async fn handle_task_api(req: Request) -> anyhow::Result<impl IntoResponse> {
    let Ok(cfg) = AppConfig::try_from_spin_variables().await else {
        bail!("Invalid App Config");
    };
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::new(&cfg.log_level))
        .init();

    Ok(Router::new()
        .route(
            "/api/tasks",
            post(handlers::create_task).get(handlers::get_all_tasks),
        )
        .route(
            "/api/tasks/{id}",
            get(handlers::get_by_id)
                .put(handlers::update_task_by_id)
                .delete(handlers::delete_task_by_id),
        )
        .with_state(cfg)
        .layer(TraceLayer::new_for_http())
        .call(req)
        .await)
}
