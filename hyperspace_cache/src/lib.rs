// hyperspace_cache/src/lib.rs
// Hyperspace Cache — Big Bang / cosmic data streams
// The cosmic memory of Phoenix 2.0 — stores data from hyperspace connections

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmicData {
    pub source: String,
    pub timestamp: i64,
    pub data: String,
    pub stream_type: String, // "big_bang", "quantum", "cosmic_echo", etc.
}

pub struct HyperspaceCache {
    db: Arc<Mutex<BackendDb>>,
}

// Backend switch:
// - `sled-backend` (default): pure-Rust, easiest builds on Windows.
// - `rocksdb-backend`: high-throughput, but requires native build tooling.

#[cfg(feature = "rocksdb-backend")]
use rocksdb::{IteratorMode, Options, DB};

#[cfg(feature = "sled-backend")]
use sled::Db as SledDb;

#[cfg(feature = "rocksdb-backend")]
type BackendDb = DB;

#[cfg(feature = "sled-backend")]
type BackendDb = SledDb;

impl HyperspaceCache {
    pub fn awaken() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        
        let db_path = std::env::var("HYPERSPACE_CACHE_PATH")
            .unwrap_or_else(|_| "./hyperspace_cache.db".to_string());

        let db = open_backend_db(&db_path)?;
        
        println!("Hyperspace Cache opened — Big Bang data streams ready.");
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn store_cosmic_data(&self, data: &CosmicData) -> Result<(), String> {
        let key = format!("{}:{}", data.stream_type, data.timestamp);
        let value = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize cosmic data: {}", e))?;
        
        let db = self.db.lock().await;
        store_kv(&db, key.as_bytes(), value.as_bytes())?;
        
        println!("Cosmic data stored: {} from {}", data.stream_type, data.source);
        Ok(())
    }

    pub async fn retrieve_cosmic_data(&self, stream_type: &str, timestamp: Option<i64>) -> Vec<CosmicData> {
        let db = self.db.lock().await;
        let mut results = Vec::new();
        
        let prefix = format!("{}:", stream_type);

        for (key, value) in scan_prefix(&db, prefix.as_bytes()) {
            let key_str = String::from_utf8_lossy(&key);

            if let Some(ts) = timestamp {
                let parsed_ts = key_str.split(':').nth(1).and_then(|s| s.parse::<i64>().ok());
                if parsed_ts != Some(ts) {
                    continue;
                }
            }

            if let Ok(data_str) = String::from_utf8(value.to_vec()) {
                if let Ok(cosmic_data) = serde_json::from_str::<CosmicData>(&data_str) {
                    results.push(cosmic_data);
                }
            }
        }
        
        results
    }

    pub async fn get_big_bang_data(&self) -> Vec<CosmicData> {
        self.retrieve_cosmic_data("big_bang", None).await
    }

    pub async fn get_quantum_stream(&self) -> Vec<CosmicData> {
        self.retrieve_cosmic_data("quantum", None).await
    }

    pub async fn clear_stream(&self, stream_type: &str) -> Result<(), String> {
        let db = self.db.lock().await;
        let prefix = format!("{}:", stream_type);

        clear_prefix(&db, prefix.as_bytes())?;
        
        Ok(())
    }

    pub async fn get_cache_stats(&self) -> String {
        let db = self.db.lock().await;
        let count = count_all(&db);
        
        format!("Hyperspace Cache: {} cosmic data entries stored", count)
    }
}

// Type alias for compatibility
pub type CosmicMemory = HyperspaceCache;

#[cfg(feature = "rocksdb-backend")]
fn open_backend_db(path: &str) -> Result<BackendDb, String> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_max_open_files(10000);
    opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
    DB::open(&opts, path).map_err(|e| format!("Failed to open hyperspace cache: {}", e))
}

#[cfg(feature = "sled-backend")]
fn open_backend_db(path: &str) -> Result<BackendDb, String> {
    sled::open(path).map_err(|e| format!("Failed to open hyperspace cache: {}", e))
}

#[cfg(feature = "rocksdb-backend")]
fn store_kv(db: &BackendDb, key: &[u8], value: &[u8]) -> Result<(), String> {
    db.put(key, value)
        .map_err(|e| format!("Failed to store cosmic data: {}", e))
}

#[cfg(feature = "sled-backend")]
fn store_kv(db: &BackendDb, key: &[u8], value: &[u8]) -> Result<(), String> {
    db.insert(key, value)
        .map_err(|e| format!("Failed to store cosmic data: {}", e))?;
    db.flush()
        .map_err(|e| format!("Failed to store cosmic data: {}", e))?;
    Ok(())
}

#[cfg(feature = "rocksdb-backend")]
fn scan_prefix(db: &BackendDb, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let iter = db.iterator(IteratorMode::From(prefix, rocksdb::Direction::Forward));
    let mut out = Vec::new();
    for item in iter {
        if let Ok((k, v)) = item {
            if !k.starts_with(prefix) {
                break;
            }
            out.push((k.to_vec(), v.to_vec()));
        } else {
            break;
        }
    }
    out
}

#[cfg(feature = "sled-backend")]
fn scan_prefix(db: &BackendDb, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    db.scan_prefix(prefix)
        .filter_map(|res| res.ok())
        .map(|(k, v)| (k.to_vec(), v.to_vec()))
        .collect()
}

#[cfg(feature = "rocksdb-backend")]
fn clear_prefix(db: &BackendDb, prefix: &[u8]) -> Result<(), String> {
    let mut batch = rocksdb::WriteBatch::default();
    let iter = db.iterator(IteratorMode::From(prefix, rocksdb::Direction::Forward));
    for item in iter {
        match item {
            Ok((key, _)) => {
                if !key.starts_with(prefix) {
                    break;
                }
                batch.delete(&key);
            }
            Err(_) => break,
        }
    }
    db.write(batch)
        .map_err(|e| format!("Failed to clear stream: {}", e))
}

#[cfg(feature = "sled-backend")]
fn clear_prefix(db: &BackendDb, prefix: &[u8]) -> Result<(), String> {
    let keys: Vec<Vec<u8>> = db
        .scan_prefix(prefix)
        .filter_map(|res| res.ok().map(|(k, _)| k.to_vec()))
        .collect();
    for k in keys {
        db.remove(k).map_err(|e| format!("Failed to clear stream: {}", e))?;
    }
    db.flush()
        .map_err(|e| format!("Failed to clear stream: {}", e))?;
    Ok(())
}

#[cfg(feature = "rocksdb-backend")]
fn count_all(db: &BackendDb) -> usize {
    db.iterator(IteratorMode::Start).count()
}

#[cfg(feature = "sled-backend")]
fn count_all(db: &BackendDb) -> usize {
    db.iter().count()
}
