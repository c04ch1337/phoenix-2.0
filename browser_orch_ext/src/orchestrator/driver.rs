use super::cdp::CdpConnection;
use super::chromium_process::ChromiumProcess;
use crate::Action;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::sync::mpsc;

pub enum DriverResponse {
    State(Value),
    Complete,
    Error(anyhow::Error),
    Ready,
}

/// A driver for a web browser.
pub struct Driver {
    process: ChromiumProcess,
    cdp: CdpConnection,
    main_frame_id: String,
}

impl Driver {
    /// Creates a new driver.
    pub async fn new() -> Result<Self> {
        let process = ChromiumProcess::new()?;
        let chrome_port = std::env::var("CHROME_DEBUG_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9222);
        let (_cdp, _) = CdpConnection::new(format!("http://127.0.0.1:{}", chrome_port)).await?;

        Ok(Self {
            process,
            cdp: _cdp,
            main_frame_id: "".to_string(),
        })
    }

    /// Starts the driver.
    pub async fn start(&mut self) -> Result<()> {
        self.cdp
            .send_message("Page.enable", serde_json::json!({}))
            .await?;
        self.cdp
            .send_message("Runtime.enable", serde_json::json!({}))
            .await?;
        self.cdp
            .send_message("DOM.enable", serde_json::json!({}))
            .await?;

        let main_frame = self
            .cdp
            .send_message("Page.getFrameTree", serde_json::json!({}))
            .await?;
        self.main_frame_id = main_frame["frameTree"]["frame"]["id"]
            .as_str()
            .unwrap()
            .to_string();

        Ok(())
    }

    /// Stops the driver.
    pub fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn handle_action(&mut self, action: Action) -> Result<DriverResponse> {
        match action {
            Action::Navigate { url } => {
                self.cdp
                    .send_message(
                        "Page.navigate",
                        serde_json::json!({
                            "url": url,
                        }),
                    )
                    .await?;
                Ok(DriverResponse::Complete)
            }
            Action::State => {
                let state = self.cdp.get_page_state().await?;
                Ok(DriverResponse::State(state))
            }
            Action::Click { i } => {
                let js = format!("document.querySelector(\"[data-r='{}']\").click()", i);
                self.cdp
                    .send_message(
                        "Runtime.evaluate",
                        serde_json::json!({
                            "expression": js,
                        }),
                    )
                    .await?;
                Ok(DriverResponse::Complete)
            }
            _ => Err(anyhow!("Invalid action")),
        }
    }
}
