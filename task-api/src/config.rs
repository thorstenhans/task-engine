use spin_sdk::variables;

#[derive(Clone, Default)]
pub(crate) struct AppConfig {
    pub log_level: String,
    pub log_as_json: bool,
    pub notification_url: String,
}

impl AppConfig {
    pub async fn try_from_spin_variables() -> anyhow::Result<Self> {
        let notification_url = variables::get("notification_url").await?;
        let log_level = variables::get("log_level").await?;
        let log_as_json: bool = variables::get("log_json")
            .await
            .unwrap_or("true".to_string())
            .parse().unwrap_or(true);

        Ok(Self {
            log_level,
            log_as_json,
            notification_url,
        })
    }
}
