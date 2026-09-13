pub(crate) mod api {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub(crate) struct WorkerParams {
        pub token: String,
    }

    #[derive(Serialize)]
    pub(crate) struct Report {
        pub done: usize,
        pub open: usize,
        pub total: usize,
    }

    impl From<super::domain::Report> for Report {
        fn from(value: super::domain::Report) -> Self {
            Self {
                done: value.done,
                open: value.open,
                total: value.total,
            }
        }
    }
}

pub(crate) mod domain {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Default)]
    pub(crate) struct Report {
        pub done: usize,
        pub open: usize,
        pub total: usize,
        pub tasks_archived_during_this_run: Vec<String>,
    }

    impl Report {
        pub fn increment_open(&mut self) {
            self.open += 1;
            self.total += 1;
        }
        pub fn increment_done(&mut self, task_id: String) {
            self.done += 1;
            self.tasks_archived_during_this_run.push(task_id);
            self.total += 1;
        }
    }
}
