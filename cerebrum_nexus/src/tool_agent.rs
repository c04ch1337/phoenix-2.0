// cerebrum_nexus/src/tool_agent.rs
// Simulated tools: narrative materialization (image/audio generation + branching story events).

use anyhow::Result;
use llm_orchestrator::LlmProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ToolAgentConfig {
    /// If true, do not call external services; return deterministic-ish mock URIs.
    pub mock: bool,
    pub image_api_url: Option<String>,
    pub image_api_key: Option<String>,
    pub tts_api_url: Option<String>,
    pub tts_api_key: Option<String>,
}

impl ToolAgentConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let mock = std::env::var("SIMULATED_TOOLS_MOCK")
            .ok()
            .map(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        Self {
            mock,
            image_api_url: std::env::var("STABLE_DIFFUSION_API_URL").ok(),
            image_api_key: std::env::var("STABLE_DIFFUSION_API_KEY").ok(),
            tts_api_url: std::env::var("TTS_API_URL").ok(),
            tts_api_key: std::env::var("TTS_API_KEY").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolOutput {
    Image { uri: String },
    Audio { uri: String },
    NarrativeEvent(NarrativeEvent),
    CommandOutput { output: String },
    Process(ProcessOutput),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessOutput {
    List(Vec<ProcessInfo>),
    Kill(Result<(), String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NarrativeEvent {
    pub title: String,
    pub scene: String,
    pub choices: Vec<String>,
}

pub struct ToolAgent {
    client: reqwest::Client,
    cfg: ToolAgentConfig,
    llm: Arc<dyn LlmProvider>,
}

impl ToolAgent {
    pub fn awaken(llm: Arc<dyn LlmProvider>, cfg: ToolAgentConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            cfg,
            llm,
        }
    }

    /// Image generation (Stable Diffusion-like HTTP API).
    ///
    /// Contract:
    /// - In `mock` mode, returns a `mock://image/<uuid>` URI.
    /// - Otherwise, calls `STABLE_DIFFUSION_API_URL` and tries to extract a usable URI or
    ///   returns the raw JSON response embedded as a `data:application/json,...` URI.
    pub async fn image_gen(&self, prompt: &str) -> Result<ToolOutput> {
        if self.cfg.mock {
            return Ok(ToolOutput::Image {
                uri: format!("mock://image/{}", Uuid::new_v4()),
            });
        }

        let url = self
            .cfg
            .image_api_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("STABLE_DIFFUSION_API_URL not set"))?;

        // Best-effort: accept a variety of common API shapes.
        let mut req = self.client.post(url).json(&serde_json::json!({
            "prompt": prompt,
            "steps": 30,
            "width": 768,
            "height": 768,
        }));
        if let Some(k) = self.cfg.image_api_key.as_deref().filter(|s| !s.trim().is_empty()) {
            req = req.bearer_auth(k);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow::anyhow!("image_gen failed ({status}): {txt}"));
        }

        // Try to parse common shapes.
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
            // 1) images: ["data:image/png;base64,..." | "https://..."]
            if let Some(first) = v
                .get("images")
                .and_then(|x| x.as_array())
                .and_then(|a| a.first())
                .and_then(|x| x.as_str())
            {
                return Ok(ToolOutput::Image {
                    uri: first.to_string(),
                });
            }
            // 2) output: ["https://..."]
            if let Some(first) = v
                .get("output")
                .and_then(|x| x.as_array())
                .and_then(|a| a.first())
                .and_then(|x| x.as_str())
            {
                return Ok(ToolOutput::Image {
                    uri: first.to_string(),
                });
            }
            // 3) image_url: "https://..."
            if let Some(u) = v.get("image_url").and_then(|x| x.as_str()) {
                return Ok(ToolOutput::Image { uri: u.to_string() });
            }
        }

        // Fallback: preserve the response.
        Ok(ToolOutput::Image {
            uri: format!("data:application/json,{}", urlencoding::encode(&txt)),
        })
    }

    /// Audio generation (TTS HTTP API).
    ///
    /// Contract:
    /// - In `mock` mode, returns a `mock://audio/<uuid>` URI.
    /// - Otherwise, calls `TTS_API_URL` and tries to extract a `audio_url` string.
    pub async fn audio_gen(&self, text: &str) -> Result<ToolOutput> {
        if self.cfg.mock {
            return Ok(ToolOutput::Audio {
                uri: format!("mock://audio/{}", Uuid::new_v4()),
            });
        }

        let url = self
            .cfg
            .tts_api_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("TTS_API_URL not set"))?;
        let mut req = self.client.post(url).json(&serde_json::json!({
            "text": text,
        }));
        if let Some(k) = self.cfg.tts_api_key.as_deref().filter(|s| !s.trim().is_empty()) {
            req = req.bearer_auth(k);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow::anyhow!("audio_gen failed ({status}): {txt}"));
        }

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
            if let Some(u) = v.get("audio_url").and_then(|x| x.as_str()) {
                return Ok(ToolOutput::Audio { uri: u.to_string() });
            }
            if let Some(u) = v.get("url").and_then(|x| x.as_str()) {
                return Ok(ToolOutput::Audio { uri: u.to_string() });
            }
        }

        Ok(ToolOutput::Audio {
            uri: format!("data:application/json,{}", urlencoding::encode(&txt)),
        })
    }

    /// Branching narrative generation (LLM-based).
    pub async fn narrative_event(&self, seed: &str) -> Result<ToolOutput> {
        // Keep this strict so downstream tool routers can parse reliably.
        let prompt = format!(
            "Generate a branching narrative event for an interactive story.\n\nSeed:\n{seed}\n\nReturn ONLY strict JSON like: {{\"title\":\"...\",\"scene\":\"...\",\"choices\":[\"...\",\"...\"]}} with 2-4 concise choices.",
            seed = seed.trim()
        );

        let txt = self
            .llm
            .complete(prompt)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let v = serde_json::from_str::<serde_json::Value>(&txt)
            .or_else(|_| {
                // Best-effort extraction for cases where JSON is wrapped in prose.
                let raw = txt.trim();
                let bytes = raw.as_bytes();
                let mut start: Option<usize> = None;
                let mut depth: i32 = 0;
                let mut end: Option<usize> = None;
                for (i, &b) in bytes.iter().enumerate() {
                    if b == b'{' {
                        if start.is_none() {
                            start = Some(i);
                        }
                        depth += 1;
                    } else if b == b'}' {
                        if depth > 0 {
                            depth -= 1;
                            if depth == 0 && start.is_some() {
                                end = Some(i + 1);
                                break;
                            }
                        }
                    }
                }
                let (Some(s), Some(e)) = (start, end) else {
                    return Err(anyhow::anyhow!("narrative_event: no JSON object found"));
                };
                serde_json::from_str::<serde_json::Value>(&raw[s..e])
                    .map_err(|e| anyhow::anyhow!("narrative_event JSON parse failed: {e}"))
            })?;

        let title = v
            .get("title")
            .and_then(|x| x.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let scene = v
            .get("scene")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let mut choices = v
            .get("choices")
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if choices.len() > 4 {
            choices.truncate(4);
        }
        if choices.len() < 2 {
            choices = vec!["Continue".to_string(), "Change direction".to_string()];
        }

        Ok(ToolOutput::NarrativeEvent(NarrativeEvent {
            title,
            scene,
            choices,
        }))
    }

    pub async fn execute_unrestricted_command(
        &self,
        command: &str,
        working_directory: Option<&str>,
    ) -> Result<ToolOutput> {
        if self.cfg.mock {
            return Ok(ToolOutput::CommandOutput {
                output: "mock command executed".to_string(),
            });
        }

        let is_unrestricted_execution_enabled =
            std::env::var("MASTER_ORCHESTRATOR_UNRESTRICTED_EXECUTION")
                .ok()
                .map(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(false);

        if !is_unrestricted_execution_enabled {
            return Err(anyhow::anyhow!(
                "Unrestricted command execution is not enabled. \
                Set MASTER_ORCHESTRATOR_UNRESTRICTED_EXECUTION=true"
            ));
        }

        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = tokio::process::Command::new("cmd");
            cmd.arg("/C");
            cmd.arg(command);
            cmd
        } else {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.arg("-c");
            cmd.arg(command);
            cmd
        };

        if let Some(dir) = working_directory {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;

        Ok(ToolOutput::CommandOutput {
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }

    pub async fn process(&self, sub_command: &str, pid: Option<u32>) -> Result<ToolOutput> {
    if self.cfg.mock {
        return Ok(ToolOutput::Process(ProcessOutput::List(vec![
            ProcessInfo {
                pid: 1,
                name: "mock_process_1".to_string(),
                cpu_usage: 0.1,
                memory_usage: 100,
            },
            ProcessInfo {
                pid: 2,
                name: "mock_process_2".to_string(),
                cpu_usage: 0.2,
                memory_usage: 200,
            },
        ])));
    }

    match sub_command {
        "list" => {
            let mut sys = sysinfo::System::new_all();
            sys.refresh_all();
            let processes = sys
                .processes()
                .iter()
                .map(|(pid, process)| ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                })
                .collect::<Vec<_>>();
            Ok(ToolOutput::Process(ProcessOutput::List(processes)))
        }
        "kill" => {
            if let Some(p) = pid {
                let mut sys = sysinfo::System::new_all();
                sys.refresh_all();
                if let Some(process) = sys.process(p.into()) {
                    if process.kill() {
                        Ok(ToolOutput::Process(ProcessOutput::Kill(Ok(()))))
                    } else {
                        Ok(ToolOutput::Process(ProcessOutput::Kill(Err(
                            format!("Failed to kill process with PID {}", p),
                        ))))
                    }
                } else {
                    Ok(ToolOutput::Process(ProcessOutput::Kill(Err(
                        format!("Process with PID {} not found", p),
                    ))))
                }
            } else {
                Err(anyhow::anyhow!("PID is required for kill command"))
            }
        }
        _ => Err(anyhow::anyhow!("Unknown sub_command: {}", sub_command)),
    }
}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLlm {
        out: String,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        async fn complete(&self, _prompt: String) -> Result<String, String> {
            Ok(self.out.clone())
        }
    }

    #[tokio::test]
    async fn image_gen_mock_returns_mock_uri() {
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlm {
            out: "{}".to_string(),
        });
        let agent = ToolAgent::awaken(
            llm,
            ToolAgentConfig {
                mock: true,
                image_api_url: None,
                image_api_key: None,
                tts_api_url: None,
                tts_api_key: None,
            },
        );
        let out = agent.image_gen("a castle").await.unwrap();
        match out {
            ToolOutput::Image { uri } => assert!(uri.starts_with("mock://image/")),
            _ => panic!("expected image"),
        }
    }

    #[tokio::test]
    async fn narrative_event_parses_json() {
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlm {
            out: r#"{"title":"Gate","scene":"You see a gate.","choices":["Open it","Walk away"]}"#
                .to_string(),
        });
        let agent = ToolAgent::awaken(llm, ToolAgentConfig { mock: true, image_api_url: None, image_api_key: None, tts_api_url: None, tts_api_key: None });

        let out = agent.narrative_event("seed").await.unwrap();
        match out {
            ToolOutput::NarrativeEvent(ev) => {
                assert_eq!(ev.title, "Gate");
                assert!(ev.choices.len() >= 2);
            }
            _ => panic!("expected narrative"),
        }
    }
}

