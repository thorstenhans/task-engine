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
