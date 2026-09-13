use axum::extract::State;
use axum::{Json, extract::Path, http::StatusCode};
use axum_anyhow::{ApiResult, bad_request, internal_error, not_found};
use tracing::{info};
use uuid::Uuid;

use crate::models::config::AppConfig;
use crate::models::{
    api::{CreateTask, Task, UpdateTask},
    domain,
};
use crate::service;

pub(crate) async fn get_all_tasks() -> ApiResult<Json<Vec<Task>>> {
    let tasks = service::get_all_tasks().await?;
    let tasks = tasks.into_iter().map(Task::from).collect();
    Ok(Json(tasks))
}

pub(crate) async fn get_by_id(Path(id): Path<String>) -> ApiResult<Json<Task>> {
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_e) => {
            info!("Callee presented invalid identifier.");
            return Err(bad_request("Bad Request", "Invalid ID"));
        },
    };

    match service::get_task_by_id(id).await? {
        Some(domain_model) => Ok(Json(Task::from(domain_model))),
        None => {
            info!("Task with Id {id} not found");
            Err(not_found("Not Found", "Task not found"))
        },
    }
}

pub(crate) async fn create_task(
    State(cfg): State<AppConfig>,
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
    let task = service::create_task(new_task, &cfg.notification_url).await?;
    Ok((StatusCode::CREATED, Json(task.into())))
}

pub(crate) async fn update_task_by_id(
    State(cfg): State<AppConfig>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTask>,
) -> ApiResult<Json<Task>> {
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err(bad_request("Bad Request", "Invalid ID")),
    };
    if payload.title.clone().is_some_and(|t| t.is_empty()) {
        return Err(bad_request("Bad request", "Title can't be empty" ));
    }
    let status = payload.status.map(|s|s.into());
    let task = match service::update_task_by_id(id, payload.title, status, &cfg.notification_url).await {
        Ok(None) => return Err(not_found("Not Found", "Task not found")),
        Ok(Some(t)) => t,
        Err(_) => return Err(internal_error("Internal Server Error", "Error updating task"))
    };
    Ok(Json(task.into()))
}

pub(crate) async fn delete_task_by_id(Path(id): Path<String>) -> ApiResult<StatusCode> {
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err(bad_request("Bad Request", "Invalid ID")),
    };
    service::delete_task_by_id(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::{extract::Path, http::StatusCode};
    use uuid::Uuid;

    use crate::task_repository::MOCK_TASK_ID;

    use super::get_by_id;

    #[tokio::test]
    async fn test_get_non_existing_by_id() {
        let id = Path(Uuid::new_v4().to_string());

        let actual = get_by_id(id).await;
        assert!(actual.is_err());
        let err = actual.unwrap_err();
        assert_eq!(err.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_existing_by_id() {
        let id = Uuid::parse_str(MOCK_TASK_ID).unwrap().to_string();
        let path_id = Path(id.clone());

        let actual = get_by_id(path_id).await;
        assert!(actual.is_ok());
        let task_json = actual.unwrap();
        assert_eq!(task_json.id, id);
    }
}
