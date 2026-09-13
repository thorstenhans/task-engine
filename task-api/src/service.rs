use uuid::Uuid;

use crate::{models::domain, task_repository::{self, Repository}};

pub(crate) async fn get_task_by_id(id: Uuid) -> anyhow::Result<Option<domain::Task>> {
    let task_repository = task_repository::new();
    task_repository.get_by_id(id).await
}
