use tracing::debug;
use uuid::Uuid;

use crate::{classifier::{self, Classifier}, models::domain::{self, Notification, Status}, notifier::{self, Notifier}, task_repository::{self, Repository}};

pub(crate) async fn get_task_by_id(id: Uuid) -> anyhow::Result<Option<domain::Task>> {
    let repo = task_repository::new();
    repo.get_by_id(id).await
}

pub(crate) async fn delete_task_by_id(id: Uuid) -> anyhow::Result<()> {
    let mut repo = task_repository::new();
    repo.delete_task_by_id(id).await

}
pub(crate)async fn update_task_by_id(id: Uuid, title: Option<String>, status: Option<domain::Status>, notification_url: &str) -> anyhow::Result<Option<domain::Task>> {
    let mut repo = task_repository::new();
    let mut task = match repo.get_by_id(id).await {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
    };
    if let Some(updated_title) = title {
        println!("Updating task title to: {}", updated_title);
        task.title = updated_title.clone();
        let classification_svc = classifier::new();
        task.classification = classification_svc.classify(&updated_title);
    }
    if let Some(updated_status) = status {
        println!("Updating task status to: {:?}", updated_status);
        task.status = updated_status;
    }

    repo.update_task(task.clone()).await?;
    if task.status == Status::Done {
        let notification_svc = notifier::new(notification_url);
        let notification = Notification::TaskCompleted(task.title.clone());
        if let Err(e) = notification_svc.send(notification).await {
            eprintln!("Could not send notification ({e})");
        };
    }
    Ok(Some(task))

}

pub(crate) async fn get_all_tasks() -> anyhow::Result<Vec<domain::Task>> {
    let task_repository = task_repository::new();
    let domain_models = task_repository.get_all().await?;
    debug!("Retrieved {} tasks from repository", domain_models.len());
    Ok(domain_models)
}

pub(crate) async fn create_task(new_task: domain::Task, notification_url: &str) -> anyhow::Result<domain::Task> {
    let mut new_task = new_task;
    let classification_svc = classifier::new();

    new_task.classification = classification_svc.classify(&new_task.title);
    let mut repo = task_repository::new();
    repo.create(new_task.clone()).await?;
    println!("Task created with ID: {}", new_task.id);
    let notification_svc = notifier::new(notification_url);
    if let Err(e) = notification_svc
        .send(Notification::TaskCreated(new_task.title.clone()))
        .await
    {
        eprintln!("Could not sent notification ({e})");
    }
    Ok(new_task)
}
