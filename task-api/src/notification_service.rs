use crate::models::domain::Notification;
use async_trait::async_trait;
use spin_sdk::http::Request;

#[async_trait]
pub(crate) trait NotificationService {
    async fn send(&self, notification: Notification) -> anyhow::Result<()>;
}

pub(crate) fn new(
    notification_endpoint: impl Into<String>,
) -> impl NotificationService {

    #[cfg(test)]
    {
    mocks::MockService::new(notification_endpoint.into())
    }
    #[cfg(not(test))]
    {
    NtfyService { api_endpoint: notification_endpoint.into()}
    }
}
#[cfg_attr(test, allow(dead_code))]
pub struct NtfyService {
    api_endpoint: String,
}

#[async_trait]
impl NotificationService for NtfyService {
    async fn send(&self, notification: Notification) -> anyhow::Result<()> {
        let title = notification.get_title();
        let contents = notification.get_contents();
        let Ok(req) = Request::builder()
            .method("POST")
            .uri(&self.api_endpoint)
            .header("title", title)
            .body(contents)
        else {
            eprintln!("could not create notification request");
            return Ok(());
        };

        // Send it via the Spin Host API
        let Ok(res) = spin_sdk::http::send(req).await else {
            eprintln!("outbound HTTP req. failed");
            return Ok(());
        };
        if res.status() != 200 {
            eprintln!("ntfy publish returned {}", res.status());
        }
        Ok(())
    }
}
#[cfg(test)]
#[cfg_attr(not(test), allow(dead_code))]
mod mocks {
    use crate::{models::domain::Notification, notification_service::NotificationService};
    use async_trait::async_trait;

    pub(super) struct MockService {}

    
    impl MockService {
        pub(super) fn new(_api_endpoint: impl Into<String>) -> Self {
            Self {}
        }
    }
    #[async_trait]
    impl NotificationService for MockService {
        #[allow(unused_variables)]
        async fn send(&self, notification: Notification) -> anyhow::Result<()> {
            let title = notification.get_title();
            let contents = notification.get_contents();
            println!("Mock notification {title}\n{contents}");
            Ok(())
        }
    }
}
