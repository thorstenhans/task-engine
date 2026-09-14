mod common;

use common::SpinTestBed;

#[derive(serde::Serialize)]
struct NewTask<'a> {
    title: &'a str,
}

#[derive(serde::Serialize)]
struct UpdateTask<'a> {
    status: Option<&'a str>,
}

#[derive(serde::Deserialize)]
struct Task {
    id: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
struct Report {
    done: usize,
    open: usize,
    total: usize,
}

#[tokio::test]
async fn run_worker_job() {
    let app = SpinTestBed::start(true)
        .await
        .expect("Could not start test bed for Spin application");
    app.wait_until_ready().await;

    let client = reqwest::Client::new();
    let worker_endpoint = app.get_endpoint_url("/jobs/cleanup?token=test-token");
    let tasks_endpoint = app.get_endpoint_url("/api/tasks");

    // POST /api/tasks -> 201 (task to be completed)
    let new_task_res = client
        .post(tasks_endpoint.clone())
        .json(&NewTask {
            title: "Sample task",
        })
        .send()
        .await
        .unwrap();
    assert_eq!(new_task_res.status(), 201);
    let new_task: Task = new_task_res.json().await.unwrap();
    let task_endpoint = app.get_endpoint_url(&format!("/api/tasks/{}", new_task.id));

    // PUT /api/tasks/{id} -> 200 (mark done task as done)
    let done_task_res = client
        .put(&task_endpoint)
        .json(&UpdateTask {
            status: Some("done"),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(done_task_res.status(), 200);

    // POST /jobs/cleanup?token=test-token -> 200 {done: 1, open: 0, total: 1}
    let worker_resp = client.post(worker_endpoint).send().await.unwrap();
    assert_eq!(worker_resp.status(), 200);

    let report: Report = worker_resp.json().await.unwrap();
    assert_eq!(
        report,
        Report {
            done: 1,
            open: 0,
            total: 1,
        }
    );

    // GET /api/tasks/{id} -> 404 (done task was archived and removed from active tasks)
    let got_done = client.get(task_endpoint).send().await.unwrap();
    assert_eq!(got_done.status(), 404);
}
