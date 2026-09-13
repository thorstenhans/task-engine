use crate::models::domain::Notification;
use async_trait::async_trait;
use spin_sdk::http::Request;

#[async_trait]
pub(crate) trait Notifier {
    async fn send(&self, notification: Notification) -> anyhow::Result<()>;
}

pub(crate) fn new(
    notification_endpoint: impl Into<String>,
) -> impl Notifier {

    #[cfg(test)]
    {
    mocks::MockNotifier::new(notification_endpoint.into())
    }
    #[cfg(not(test))]
    {
    NtfyNotifier { api_endpoint: notification_endpoint.into()}
    }
}
#[cfg_attr(test, allow(dead_code))]
pub struct NtfyNotifier {
    api_endpoint: String,
}

#[async_trait]
impl Notifier for NtfyNotifier {
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
    use crate::{models::domain::Notification};
    use super::Notifier;
    use async_trait::async_trait;

    pub(super) struct MockNotifier {}

    
    impl MockNotifier {
        pub(super) fn new(_api_endpoint: impl Into<String>) -> Self {
            Self {}
        }
    }
    #[async_trait]
    impl Notifier for MockNotifier {
        #[allow(unused_variables)]
        async fn send(&self, notification: Notification) -> anyhow::Result<()> {
            let title = notification.get_title();
            let contents = notification.get_contents();
            println!("Mock notification {title}\n{contents}");
            Ok(())
        }
    }
}
