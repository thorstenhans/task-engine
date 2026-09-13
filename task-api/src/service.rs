use crate::{
    classifier::{self, Classifier},
    models::domain::{self, Notification, Status},
    notifier::{self, Notifier},
    repository::{self, Repository},
};
use tracing::debug;
use uuid::Uuid;

pub(crate) async fn get_task_by_id(id: Uuid) -> anyhow::Result<Option<domain::Task>> {
    let repo = repository::new();
    repo.get_by_id(id).await
}

pub(crate) async fn delete_task_by_id(id: Uuid) -> anyhow::Result<()> {
    let mut repo = repository::new();
    repo.delete_task_by_id(id).await
}
pub(crate) async fn update_task_by_id(
    id: Uuid,
    title: Option<String>,
    status: Option<domain::Status>,
    notification_url: &str,
) -> anyhow::Result<Option<domain::Task>> {
    let mut repo = repository::new();
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
    let task_repository = repository::new();
    let domain_models = task_repository.get_all().await?;
    debug!("Retrieved {} tasks from repository", domain_models.len());
    Ok(domain_models)
}

pub(crate) async fn create_task(
    new_task: domain::Task,
    notification_url: &str,
) -> anyhow::Result<domain::Task> {
    let mut new_task = new_task;
    let classification_svc = classifier::new();

    new_task.classification = classification_svc.classify(&new_task.title);
    let mut repo = repository::new();
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{handlers::get_all_tasks, repository::MOCK_TASK_ID, service::get_task_by_id};

    #[tokio::test]
    async fn test_get_non_existing_by_id() {
        // arrange
        let id = Uuid::new_v4();

        // act
        let actual = get_task_by_id(id).await;

        // assert
        match actual {
            Ok(None) => {}
            other => panic!("Expected Ok(None), got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_existing_by_id() {
        // arrange
        let id = Uuid::parse_str(MOCK_TASK_ID).unwrap();

        // act
        let actual = get_task_by_id(id).await;

        // assert
        match actual {
            Ok(Some(t)) => assert_eq!(t.id, id),
            other => panic!("Expeceted Task with Id {id}, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_all_should_return_seed() {
        // arrange

        // act
        let actual = get_all_tasks().await;

        // assert
        match actual {
            Ok(tasks) => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].id, MOCK_TASK_ID);
            },
            other => panic!("Expected 1 Task, got {:?}", other)
        }
    }
}
