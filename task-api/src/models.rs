pub(crate) mod api {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub(crate) struct CreateTask {
        pub title: String,
    }

    #[derive(Deserialize)]
    pub(crate) struct UpdateTask {
        pub title: Option<String>,
        pub status: Option<Status>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub(crate) struct Task {
        pub id: String,
        pub title: String,
        pub classification: String,
        pub status: Status,
    }

    impl From<super::domain::Task> for Task {
        fn from(value: super::domain::Task) -> Self {
            Self {
                id: value.id.to_string(),
                title: value.title,
                classification: value.classification,
                status: match value.status {
                    super::domain::Status::Open => Status::Open,
                    super::domain::Status::Done => Status::Done,
                },
            }
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(rename_all = "lowercase")]
    pub(crate) enum Status {
        Open,
        Done,
    }
}

pub(crate) mod domain {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    pub(crate) enum Notification {
        TaskCreated(String),
        TaskCompleted(String),
    }
    impl Notification {
        pub fn get_title(&self) -> String {
            match self {
                Notification::TaskCreated(_) => String::from("New Task Added!"),
                Notification::TaskCompleted(_) => String::from("Task Completed!"),
            }
        }
        pub fn get_contents(&self) -> String {
            match self {
                Notification::TaskCreated(task_title) => {
                    format!("Task '{task_title}' has been created.")
                }
                Notification::TaskCompleted(task_title) => {
                    format!("Task '{task_title}' has been Completed.")
                }
            }
        }
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub(crate) struct Task {
        pub id: Uuid,
        pub title: String,
        pub classification: String,
        pub status: Status,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
    pub(crate) enum Status {
        Open,
        Done,
    }

    impl From<super::api::Status> for Status {
        fn from(value: super::api::Status) -> Self {
            match value {
                super::api::Status::Open => Self::Open,
                super::api::Status::Done => Self::Done,
            }
        }
    }
}

pub(crate) mod config {
    use spin_sdk::variables;

    #[derive(Clone, Default)]
    pub(crate) struct AppConfig {
        pub log_level: String,
        pub notification_url: String,
    }

    impl AppConfig {
        pub async fn try_from_spin_variables() -> anyhow::Result<Self> {
            let notification_url = variables::get("notification_url").await?;
            let log_level = variables::get("log_level").await?;

            Ok(Self {
                log_level,
                notification_url,
            })
        }
    }
}
