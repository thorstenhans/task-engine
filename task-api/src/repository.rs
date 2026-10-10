use crate::models::domain::Task;
use anyhow::Context;
use async_trait::async_trait;
use spin_sdk::key_value::Store;
use uuid::Uuid;

const TASK_KEY_PREFIX: &str = "task-";

#[async_trait]
pub trait Repository {
    async fn create(&mut self, value: Task) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Task>>;
    async fn get_by_id(&self, id: Uuid) -> anyhow::Result<Option<Task>>;
    async fn update_task(&mut self, value: Task) -> anyhow::Result<()>;
    async fn delete_task_by_id(&mut self, id: Uuid) -> anyhow::Result<()>;
}

pub(crate) fn new() -> impl Repository {
    #[cfg(test)]
    {
        mocks::MockTaskRepository::new()
    }
    #[cfg(not(test))]
    {
        KeyValueRepository {}
    }
}

#[cfg_attr(test, allow(dead_code))]
pub(crate) struct KeyValueRepository {}

fn get_task_key(task_id: &Uuid) -> String {
    format!("{TASK_KEY_PREFIX}{task_id}")
}

fn is_task(task_key: &str) -> bool {
    task_key.starts_with(TASK_KEY_PREFIX)
}
#[async_trait]
impl Repository for KeyValueRepository {
    async fn create(&mut self, value: Task) -> anyhow::Result<()> {
        let store = Store::open_default().await?;
        let key = get_task_key(&value.id);
        store.set_json(key, &value).await
    }

    async fn get_all(&self) -> anyhow::Result<Vec<Task>> {
        let store = Store::open_default().await?;
        let all_keys = store.get_keys().await.collect().await?;
        let mut all_tasks = vec![];
        for maybe_task_key in all_keys.iter() {
            if is_task(maybe_task_key) {
                let Some(task) = store.get_json::<Task>(maybe_task_key).await? else {
                    continue;
                };
                all_tasks.push(task);
            }
        }
        Ok(all_tasks)
    }

    async fn get_by_id(&self, id: Uuid) -> anyhow::Result<Option<Task>> {
        let store = Store::open_default().await?;
        let key = get_task_key(&id);
        let maybe_task = store.get_json::<Task>(key).await?;
        Ok(maybe_task)
    }

    async fn update_task(&mut self, value: Task) -> anyhow::Result<()> {
        let store = Store::open_default().await?;
        let key = get_task_key(&value.id);
        let _ = store.set_json(key, &value).await?;
        Ok(())
    }

    async fn delete_task_by_id(&mut self, id: Uuid) -> anyhow::Result<()> {
        let store = Store::open_default().await?;
        let key = get_task_key(&id);
        store
            .delete(key)
            .await
            .with_context(|| "Could not delete task")
    }
}

#[cfg(test)]
pub(crate) const MOCK_TASK_ID: &str = "4451ee82-1c52-4acd-bce3-6d6becb71e60";

#[cfg(test)]
mod mocks {
    use crate::{
        models::domain::Task,
        repository::{MOCK_TASK_ID, Repository, get_task_key, is_task},
    };
    use async_trait::async_trait;
    use std::collections::HashMap;
    use uuid::Uuid;

    pub(crate) struct MockTaskRepository {
        store: HashMap<String, Task>,
    }

    impl MockTaskRepository {
        pub(super) fn new() -> Self {
            use uuid::Uuid;

            use crate::models::domain::Status;

            let id = Uuid::parse_str(MOCK_TASK_ID).unwrap();
            let key = get_task_key(&id);
            let mut service = Self {
                store: HashMap::new(),
            };
            service.store.insert(
                key,
                Task {
                    id,
                    title: String::from("Buy milk"),
                    status: Status::Open,
                    classification: "Neutral".to_string(),
                },
            );
            service
        }
    }

    #[async_trait]
    impl Repository for MockTaskRepository {
        async fn create(&mut self, value: Task) -> anyhow::Result<()> {
            let key = get_task_key(&value.id);
            self.store.insert(key, value);
            Ok(())
        }

        async fn get_all(&self) -> anyhow::Result<Vec<Task>> {
            let mut all_tasks = vec![];
            for (key, maybe_task) in self.store.clone() {
                if is_task(&key) {
                    all_tasks.push(maybe_task);
                }
            }
            Ok(all_tasks)
        }

        async fn get_by_id(&self, id: Uuid) -> anyhow::Result<Option<Task>> {
            let key = get_task_key(&id);
            let maybe_task = self.store.get(&key).cloned();
            Ok(maybe_task)
        }

        async fn update_task(&mut self, value: Task) -> anyhow::Result<()> {
            let key = get_task_key(&value.id);
            self.store.insert(key, value);
            Ok(())
        }

        async fn delete_task_by_id(&mut self, id: Uuid) -> anyhow::Result<()> {
            let key = get_task_key(&id);
            self.store.remove(&key);
            Ok(())
        }
    }
}
