mod common;

use common::{SpinTestBed, wait_until_ready};
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
async fn create_then_fetch_task() {
    let app = SpinTestBed::start(true).await
        .expect("Could not start test bed for Spin application");
    let base = app.get_base_url();
    wait_until_ready(&base).await;

    let client = reqwest::Client::new();
    // POST /api/tasks -> 201 {task}
    let resp = client
        .post(format!("{base}/api/tasks"))
        .json(&NewTask { title: "Buy Coffee ASAP" })
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let created: Task = resp.json().await.unwrap();
    // The classifier is composed into task-api at build time; a populated
    // classification proves `spin build` wired the dependency correctly.
    assert_eq!(created.classification.as_deref(), Some("urgent"));

    // GET /api/tasks/{id} -> 200 {task}
    let got = client
        .get(format!("{base}/api/tasks/{}", created.id))
        .send()
        .await
        .unwrap();
    assert_eq!(got.status(), 200);
    assert_eq!(got.json::<Task>().await.unwrap().title, "Buy Coffee ASAP");

    // DELETE /api/tasks/{id} -> 204
    let deleted = client
        .delete(format!("{base}/api/tasks/{}", created.id))
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), 204);
}
