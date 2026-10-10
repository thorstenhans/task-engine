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
}

#[tokio::test]
async fn create_then_get_and_delete() {
    let app = SpinTestBed::start(true)
        .await
        .expect("Could not start test bed for Spin application");
    app.wait_until_ready().await;

    let tasks_endpoint = app.get_endpoint_url("/api/tasks");
    let client = reqwest::Client::new();
    // POST /api/tasks -> 201 {task}
    let resp = client
        .post(tasks_endpoint)
        .json(&NewTask {
            title: "Buy Coffee ASAP",
        })
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let created: Task = resp.json().await.unwrap();
    // The classifier is composed into task-api at build time; a populated
    // classification proves `spin build` wired the dependency correctly.
    assert_eq!(created.classification.as_deref(), Some("urgent"));

    // GET /api/tasks/{id} -> 200 {task}
    let task_endpoint = app.get_endpoint_url(&format!("/api/tasks/{}", created.id));

    let got = client.get(task_endpoint.clone()).send().await.unwrap();
    assert_eq!(got.status(), 200);
    assert_eq!(got.json::<Task>().await.unwrap().title, "Buy Coffee ASAP");

    // DELETE /api/tasks/{id} -> 204
    let deleted = client.delete(task_endpoint).send().await.unwrap();
    assert_eq!(deleted.status(), 204);
}
