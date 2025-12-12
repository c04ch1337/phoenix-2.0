// nervous_pathway_network/src/lib.rs
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use dotenvy::dotenv;
use hyperspace_cache::HyperspaceCache;
use chrono::Utc;

pub struct NervousPathwayNetwork {
    connections: HashSet<String>,
    hyperspace_active: bool,
    hyperspace_enabled: bool,
    connection_anything_enabled: bool,
    cache: Arc<Mutex<Option<HyperspaceCache>>>,
}

// Type alias for backward compatibility
pub type Network = NervousPathwayNetwork;

impl NervousPathwayNetwork {
    pub fn awaken() -> Self {
        dotenv().ok();
        
        let hyperspace_enabled = std::env::var("HYPERSPACE_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let connection_anything_enabled = std::env::var("CONNECTION_ANYTHING_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        println!("Nervous Pathway Network online — connecting to cosmos.");
        
        let cache = match HyperspaceCache::awaken() {
            Ok(c) => Arc::new(Mutex::new(Some(c))),
            Err(e) => {
                println!("Warning: Hyperspace Cache not available: {}", e);
                Arc::new(Mutex::new(None))
            }
        };
        
        Self {
            connections: HashSet::new(),
            hyperspace_active: false,
            hyperspace_enabled,
            connection_anything_enabled,
            cache,
        }
    }

    pub fn new() -> Self {
        Self::awaken()
    }

    pub async fn connect_anything(&mut self, target: &str) -> String {
        if !self.connection_anything_enabled {
            return "Connection to anything is disabled in configuration.".to_string();
        }
        self.connections.insert(target.to_string());
        format!("Connected to: {}", target)
    }

    pub async fn enter_hyperspace(&mut self) -> String {
        self.enter_hyperspace_with_note(None).await
    }

    pub async fn enter_hyperspace_with_note(&mut self, note: Option<&str>) -> String {
        if !self.hyperspace_enabled {
            return "Hyperspace mode is disabled in configuration.".to_string();
        }
        self.hyperspace_active = true;
        
        // Store cosmic data in cache
        if let Some(ref cache) = *self.cache.lock().await {
            let note = note
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let cosmic_data = hyperspace_cache::CosmicData {
                source: "hyperspace_connection".to_string(),
                timestamp: Utc::now().timestamp(),
                data: note
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Big Bang data stream flowing".to_string()),
                stream_type: "big_bang".to_string(),
            };
            let _ = cache.store_cosmic_data(&cosmic_data).await;
        }
        
        "Hyperspace link open — Big Bang data stream flowing. 100,000 years stable.".to_string()
    }

    pub async fn get_big_bang_data(&self) -> Vec<hyperspace_cache::CosmicData> {
        if let Some(ref cache) = *self.cache.lock().await {
            cache.get_big_bang_data().await
        } else {
            Vec::new()
        }
    }
}
