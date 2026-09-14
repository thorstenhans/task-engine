use crate::{
    classifier::{self, Classifier},
    config::AppConfig,
    models::domain::{self, Notification, Status},
    notifier::{self, Notifier},
    repository::{self, Repository},
};
use tracing::{debug, info, warn};
use uuid::Uuid;

pub(crate) struct TaskService {
    notifier: Box<dyn Notifier + Send + Sync>,
    classifier: Box<dyn Classifier + Send + Sync>,
    repo: Box<dyn Repository + Send + Sync>,
}
impl TaskService {
    pub(crate) fn new(cfg: &AppConfig) -> Self {
        Self {
            notifier: Box::new(notifier::new(cfg.notification_url.clone())),
            classifier: Box::new(classifier::new()),
            repo: Box::new(repository::new()),
        }
    }
    pub(crate) async fn get_task_by_id(&self, id: Uuid) -> anyhow::Result<Option<domain::Task>> {
        self.repo.get_by_id(id).await
    }

    pub(crate) async fn delete_task_by_id(&self, id: Uuid) -> anyhow::Result<()> {
        let mut repo = repository::new();
        repo.delete_task_by_id(id).await
    }

    pub(crate) async fn update_task_by_id(
        &mut self,
        id: Uuid,
        title: Option<String>,
        status: Option<domain::Status>,
    ) -> anyhow::Result<Option<domain::Task>> {
        let mut task = match self.repo.get_by_id(id).await {
            Ok(Some(t)) => t,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };
        if let Some(updated_title) = title {
            println!("Updating task title to: {}", updated_title);
            task.title = updated_title.clone();
            task.classification = self.classifier.classify(&updated_title);
        }
        let old_status = task.status.clone();
        if let Some(updated_status) = status {
            println!("Updating task status to: {:?}", updated_status);
            task.status = updated_status;
        }

        self.repo.update_task(task.clone()).await?;
        if old_status == Status::Open && task.status == Status::Done {
            let notification = Notification::TaskCompleted(task.title.clone());
            if let Err(e) = self.notifier.send(notification).await {
                eprintln!("Could not send notification ({e})");
            };
        }
        Ok(Some(task))
    }

    pub(crate) async fn get_all_tasks(&self) -> anyhow::Result<Vec<domain::Task>> {
        let domain_models = self.repo.get_all().await?;
        debug!("Retrieved {} tasks from repository", domain_models.len());
        Ok(domain_models)
    }

    pub(crate) async fn create_task(&mut self, title: String) -> anyhow::Result<domain::Task> {
        let new_task_id = Uuid::new_v4();
        info!("Generated new ID {new_task_id}");
        let new_task = domain::Task {
            id: new_task_id,
            title: title.clone(),
            classification: self.classifier.classify(&title),
            status: domain::Status::Open,
        };

        self.repo.create(new_task.clone()).await?;
        debug!("Task created with ID: {}", new_task.id);
        if let Err(e) = self
            .notifier
            .send(Notification::TaskCreated(new_task.title.clone()))
            .await
        {
            warn!("Could not sent notification ({e})");
        }
        Ok(new_task)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{config::AppConfig, repository::MOCK_TASK_ID, service::TaskService};

    #[tokio::test]
    async fn test_get_non_existing_by_id() {
        // arrange
        let id = Uuid::new_v4();
        let sut = TaskService::new(&AppConfig::default());

        // act
        let actual = sut.get_task_by_id(id).await;

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
        let sut = TaskService::new(&AppConfig::default());

        // act
        let actual = sut.get_task_by_id(id).await;

        // assert
        match actual {
            Ok(Some(t)) => assert_eq!(t.id, id),
            other => panic!("Expeceted Task with Id {id}, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_all_should_return_seed() {
        // arrange
        let sut = TaskService::new(&AppConfig::default());
        let expected_id = Uuid::parse_str(MOCK_TASK_ID).unwrap();
        // act
        let actual = sut.get_all_tasks().await;

        // assert
        match actual {
            Ok(tasks) => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].id, expected_id);
            }
            other => panic!("Expected 1 Task, got {:?}", other),
        }
    }
}
