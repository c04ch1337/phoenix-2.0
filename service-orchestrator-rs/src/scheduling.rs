use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Result;

// A placeholder for the real ConnectorRegistry
pub struct ConnectorRegistry;

impl ConnectorRegistry {
    pub async fn execute(&self, _operation: &str, _params: HashMap<&str, String>) -> Result<()> {
        // In a real implementation, this would find the right connector
        // and call the appropriate method (e.g., `schedule_post`).
        println!("Executing operation via registry");
        Ok(())
    }
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    #[error("Job creation error: {0}")]
    Job(String),
}


pub struct Scheduler {
    sched: JobScheduler,
    registry: Arc<ConnectorRegistry>,
}

impl Scheduler {
    pub async fn new(registry: Arc<ConnectorRegistry>) -> Result<Self, Error> {
        let sched = JobScheduler::new().await.map_err(|e| Error::Scheduler(e.to_string()))?;
        Ok(Self { sched, registry })
    }

    pub async fn add_social_job(&self, platform: &str, cron: &str, content: Value) -> Result<String, Error> {
        let registry_clone = self.registry.clone();
        let platform_str = platform.to_string();
        let job = Job::new_async(cron, move |_uuid, _l| {
            let reg = registry_clone.clone();
            let content_clone = content.clone();
            let p_str = platform_str.clone();
            Box::pin(async move {
                let mut params = HashMap::new();
                params.insert("platform", p_str.to_string());
                if let Ok(content_str) = serde_json::to_string(&content_clone) {
                    params.insert("content", content_str);
                }

                if let Err(e) = reg.execute("social.post", params).await {
                    println!("Failed to execute social post job: {}", e);
                }
            })
        }).map_err(|e| Error::Job(e.to_string()))?;

        self.sched.add(job).await.map_err(|e| Error::Scheduler(e.to_string()))?;
        Ok("Job added".to_string())
    }

    pub async fn start(&self) -> Result<(), Error> {
        self.sched.start().await.map_err(|e| Error::Scheduler(e.to_string()))?;
        Ok(())
    }
}