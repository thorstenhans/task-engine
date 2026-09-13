use crate::models::api::{self, WorkerParams};
use crate::models::domain;
use serde_json::Value;
use serde_querystring::{ParseMode, from_str};
use spin_sdk::http::{IntoResponse, Json, Method, Request, StatusCode};
use spin_sdk::key_value::Store;
use spin_sdk::{http_service, variables};

mod models;

const KEY_TASK_PREFIX: &str = "task-";
const KEY_REPORT: &str = "report";
const KEY_ARCHIVED_TASK_PREFIX: &str = "archived-task-";
const TASK_STATUS_OPEN: &str = "open";
const TASK_STATUS_DONE: &str = "done";
const TASK_STATUS_PROPERTY: &str = "status";
const TASK_ID_PROPERTY: &str = "id";

#[http_service]
async fn handle_worker(req: Request) -> anyhow::Result<impl IntoResponse> {
    if req.method() != Method::POST || req.method() != Method::GET {
        return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
    }
    let Ok(expected_token) = variables::get("token").await else {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };
    let Some(qs) = req.uri().query() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    let Ok(params) = from_str::<WorkerParams>(qs, ParseMode::UrlEncoded) else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    if params.token != expected_token {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let result: api::Report = cleanup_and_generate_report()
        .await?
        .into();
    
    Ok(Json(result).into_response())
}

async fn cleanup_and_generate_report() -> anyhow::Result<domain::Report> {
    let store = Store::open_default().await?;

    let mut report = store.get_json::<domain::Report>(KEY_REPORT)
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    let all_keys: Vec<String> = store.get_keys().await.collect().await?;
    let task_keys = all_keys
        .into_iter()
        .filter(|k| k.starts_with(KEY_TASK_PREFIX));

    for key in task_keys {
        // With the `json` feature we skip manual (de)serialization.
        let Some(task): Option<Value> = store.get_json(&key).await? else {
            continue;
        };
        let status = task
            .get(TASK_STATUS_PROPERTY)
            .map(|v| v.to_string())
            .unwrap_or_default()
            .to_lowercase();
        let task_id = task.get(TASK_ID_PROPERTY)
            .map(|v| v.to_string())
            .unwrap_or_default();

        match status.as_str() {
            TASK_STATUS_OPEN => {
                report.increment_open();
            }
            TASK_STATUS_DONE => {
                let archived_task_key = format!("{}{}", KEY_ARCHIVED_TASK_PREFIX, task_id);
                store.set_json(archived_task_key, &task).await?;
                store.delete(&key).await?;
                report.increment_done(task_id.clone());
            }
            _ => {}
        }
    }
    store.set_json(KEY_REPORT, &report).await?;
    Ok(report)
}
