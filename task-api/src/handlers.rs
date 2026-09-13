use axum::extract::State;
use axum::{Json, extract::Path, http::StatusCode};
use axum_anyhow::{ApiResult, bad_request, internal_error, not_found};
use tracing::{debug, info};
use uuid::Uuid;

use crate::classifier::{self, Classifier};
use crate::models::config::AppConfig;
use crate::models::domain::{Notification, Status};
use crate::models::{
    api::{CreateTask, Task, UpdateTask},
    domain,
};
use crate::notifier::{self, Notifier};
use crate::service;
use crate::task_repository::{self, Repository};

pub(crate) async fn get_all_tasks() -> ApiResult<Json<Vec<Task>>> {
    let task_repository = task_repository::new();
    let domain_models = task_repository.get_all().await?;
    debug!("Retrieved {} tasks from repository", domain_models.len());
    let tasks = domain_models.into_iter().map(Task::from).collect();
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
    let classification_svc = classifier::new();

    let domain_model = domain::Task {
        id: Uuid::new_v4(),
        title: payload.title.clone(),
        classification: classification_svc.classify(&payload.title),
        status: domain::Status::Open,
    };
    let mut task_repo = task_repository::new();
    task_repo.create(domain_model.clone()).await?;
    println!("Task created with ID: {}", domain_model.id);
    let notification_svc = notifier::new(&cfg.notification_url);
    if let Err(e) = notification_svc
        .send(Notification::TaskCreated(domain_model.title.clone()))
        .await
    {
        eprintln!("Could not sent notification ({e})")
    }
    Ok((StatusCode::CREATED, Json(domain_model.into())))
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
    let mut task_repo = task_repository::new();
    let mut task = match task_repo.get_by_id(id).await {
        Ok(Some(t)) => t,
        Ok(None) => return Err(not_found("Not Found", "Task not found")),
        Err(_) => return Err(internal_error("Internal Error", "Could not load task")),
    };
    if let Some(updated_title) = payload.title {
        println!("Updating task title to: {}", updated_title);
        task.title = updated_title.clone();
        let classification_svc = classifier::new();
        task.classification = classification_svc.classify(&updated_title);
    }
    if let Some(updated_status) = payload.status.map(|s| s.into()) {
        println!("Updating task status to: {:?}", updated_status);
        task.status = updated_status;
    }

    task_repo.update_task(task.clone()).await?;
    if task.status == Status::Done {
        let notification_svc = notifier::new(&cfg.notification_url);
        let notification = Notification::TaskCompleted(task.title.clone());
        if let Err(e) = notification_svc.send(notification).await {
            eprintln!("Could not send notification ({e})");
        };
    }
    Ok(Json(task.into()))
}

pub(crate) async fn delete_task_by_id(Path(id): Path<String>) -> ApiResult<StatusCode> {
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err(bad_request("Bad Request", "Invalid ID")),
    };
    let mut task_repository = task_repository::new();
    task_repository.delete_task_by_id(id).await?;
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
