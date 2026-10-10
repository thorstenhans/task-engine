use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use anyhow::Context;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub(crate) struct SpinTestBed {
    pub child: Child,
    pub port: u16,
}

const LOOPBACK: &str = "127.0.0.1";
const SPIN_HEALTH_ENDPOINT: &str = "/.well-known/spin/health";

impl SpinTestBed {
    pub(crate) async fn start(quiet: bool) -> anyhow::Result<Self> {
        let ntfy_mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/notify"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&ntfy_mock)
            .await;

        let Ok(port) = find_free_port() else {
            return Err(anyhow::anyhow!("Could not find free port"));
        };
        let listener = format!("{LOOPBACK}:{port}");
        let unique_state_id = Uuid::new_v4();
        let state_dir = format!(".spin/{unique_state_id}");

        let mut args = vec!["up", "--listen", &listener, "--state-dir", &state_dir];
        if quiet {
            args.push("--quiet");
        }
        let ntfy_mock_origin = ntfy_mock.uri();

        let mut cmd = Command::new("spin");
        cmd.stderr(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.args(args)
            .env("SPIN_VARIABLE_NOTIFICATION_ORIGIN", ntfy_mock_origin)
            .env("SPIN_VARIABLE_WORKER_TOKEN", "test-token");

        #[cfg(unix)]
        {
            cmd.process_group(0);
        }
        let child = cmd
            .spawn()
            .with_context(|| "Failed to launch Spin for Integration Test")?;
        Ok(SpinTestBed { child, port })
    }

    pub(crate) fn get_endpoint_url(&self, path: &str) -> String {
        format!("http://{LOOPBACK}:{}{path}", self.port)
    }

    pub(crate) async fn wait_until_ready(&self) {
        let client = reqwest::Client::new();
        let url = self.get_endpoint_url(SPIN_HEALTH_ENDPOINT);
        for _ in 0..50 {
            if client.get(url.clone()).send().await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        panic!("app not ready in time");
    }
}

impl Drop for SpinTestBed {
    fn drop(&mut self) {
        let pid = self.child.id();
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .args(["-9", &format!("-{}", pid)])
                .status();
        }
        #[cfg(not(unix))]
        {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn find_free_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind(format!("{LOOPBACK}:0"))
        .with_context(|| "Failed to bind to ephemeral port")?;
    let local_addr = listener
        .local_addr()
        .with_context(|| "Faild to determine free local port")?;
    Ok(local_addr.port())
}
