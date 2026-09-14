mod common;
use common::SpinTestBed;

#[derive(serde::Serialize)]
struct NewTask<'a> {
    title: &'a str,
}

#[derive(serde::Deserialize)]
struct Task {
    id: String,
    title: String,
    classification: Option<String>,
    status: Option<String>,
}

#[tokio::test]
async fn create_then_list_tasks() {
    let app = SpinTestBed::start(true)
        .await
        .expect("Could not start test bed for Spin application");
    app.wait_until_ready().await;

    let tasks_endpoint = app.get_endpoint_url("/api/tasks");
    let client = reqwest::Client::new();

    // POST /api/tasks -> 201 {task}
    let create_task_res = client
        .post(tasks_endpoint.clone())
        .json(&NewTask {
            title: "Plan vacation someday",
        })
        .send()
        .await
        .unwrap();
    assert_eq!(create_task_res.status(), 201);
    let task: Task = create_task_res.json().await.unwrap();
    assert_eq!(task.classification.as_deref(), Some("relaxed"));
    assert_eq!(task.status.as_deref(), Some("open"));

    // GET /api/tasks -> 200 [{task}]
    let list = client.get(tasks_endpoint).send().await.unwrap();
    assert_eq!(list.status(), 200);

    let tasks: Vec<Task> = list.json().await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(
        tasks
            .iter()
            .any(|t| t.id == task.id && t.title == "Plan vacation someday")
    );
}
