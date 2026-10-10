pub(crate) mod api {
    use garde::Validate;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Validate)]
    pub(crate) struct CreateTask {
        #[garde(length(min = 3, max = 100))]
        pub title: String,
    }

    #[derive(Deserialize, Validate)]
    pub(crate) struct UpdateTask {
        #[garde(length(min = 3, max = 100))]
        pub title: Option<String>,
        #[garde(skip)]
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
            (match self {
                Self::TaskCreated(_) => "New Task Added!",
                Self::TaskCompleted(_) => "Task Completed!",
            })
            .to_string()
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
