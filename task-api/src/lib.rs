use anyhow::bail;
use axum::Router;
use axum::routing::{get, post};
use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_service;
use std::sync::{Arc, Once};
use tower_http::trace::TraceLayer;
use tower_service::Service;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::models::config::AppConfig;

mod classifier;
mod models;
mod notifier;
mod repository;
mod service;

spin_sdk::dependencies!();

static INIT: Once = Once::new();

pub fn init_logging(log_level: &str) {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(log_level))
            .try_init();
    });
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub cfg: Arc<AppConfig>,
}

#[http_service]
async fn handle_task_api(req: Request) -> anyhow::Result<impl IntoResponse> {
    let Ok(cfg) = AppConfig::try_from_spin_variables().await else {
        bail!("Invalid App Config");
    };
    let state = AppState { cfg: Arc::new(cfg) };
    init_logging(&state.cfg.log_level);
    debug!(
        "Passing incoming request to the axum router ({} {})",
        req.method(),
        req.uri()
    );
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
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .call(req)
        .await)
}

mod handlers {
    use axum::extract::State;
    use axum::{Json, extract::Path, http::StatusCode};
    use axum_anyhow::{ApiResult, bad_request, internal_error, not_found};
    use tracing::info;
    use uuid::Uuid;

    use crate::AppState;
    use crate::models::api::{CreateTask, Task, UpdateTask};
    use crate::models::domain;
    use crate::service::TaskService;

    #[axum::debug_handler]
    pub(super) async fn get_all_tasks(State(state): State<AppState>) -> ApiResult<Json<Vec<Task>>> {
        let svc = TaskService::new(&state.cfg);
        let tasks = svc.get_all_tasks().await?;
        let tasks = tasks.into_iter().map(Task::from).collect();
        Ok(Json(tasks))
    }
    pub(crate) async fn get_by_id(
        Path(id): Path<String>,
        State(state): State<AppState>,
    ) -> ApiResult<Json<Task>> {
        let id = match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_e) => {
                info!("Callee presented invalid identifier.");
                return Err(bad_request("Bad Request", "Invalid ID"));
            }
        };

        let svc = TaskService::new(&state.cfg);

        match svc.get_task_by_id(id).await? {
            Some(domain_model) => Ok(Json(Task::from(domain_model))),
            None => {
                info!("Task with Id {id} not found");
                Err(not_found("Not Found", "Task not found"))
            }
        }
    }

    pub(crate) async fn create_task(
        State(state): State<AppState>,
        Json(payload): Json<CreateTask>,
    ) -> ApiResult<(StatusCode, Json<Task>)> {
        if payload.title.is_empty() {
            return Err(bad_request("Bad Request", "Empty title presented"));
        }

        let new_task = domain::Task {
            id: Uuid::new_v4(),
            title: payload.title.clone(),
            classification: String::default(),
            status: domain::Status::Open,
        };
        let mut svc = TaskService::new(&state.cfg);
        let task = svc.create_task(new_task).await?;
        Ok((StatusCode::CREATED, Json(task.into())))
    }
    pub(crate) async fn update_task_by_id(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<UpdateTask>,
    ) -> ApiResult<Json<Task>> {
        let id = match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_) => return Err(bad_request("Bad Request", "Invalid ID")),
        };
        if payload.title.clone().is_some_and(|t| t.is_empty()) {
            return Err(bad_request("Bad request", "Title can't be empty"));
        }
        let status = payload.status.map(|s| s.into());
        let mut svc = TaskService::new(&state.cfg);
        let task = match svc.update_task_by_id(id, payload.title, status).await {
            Ok(None) => return Err(not_found("Not Found", "Task not found")),
            Ok(Some(t)) => t,
            Err(_) => {
                return Err(internal_error(
                    "Internal Server Error",
                    "Error updating task",
                ));
            }
        };
        Ok(Json(task.into()))
    }

    pub(crate) async fn delete_task_by_id(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> ApiResult<StatusCode> {
        let id = match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_) => return Err(bad_request("Bad Request", "Invalid ID")),
        };
        let svc = TaskService::new(&state.cfg);
        svc.delete_task_by_id(id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
