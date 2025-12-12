// vascular_integrity_system/src/lib.rs
use sled::Db;
use std::sync::Arc;
use chrono::Utc;
use sha2::{Sha256, Digest};

pub struct VascularIntegritySystem {
    db: Arc<Db>,
    last_hash: Arc<std::sync::Mutex<Option<Vec<u8>>>>,
}

impl VascularIntegritySystem {
    pub fn awaken() -> Self {
        let db = sled::open("./compliance_audit.db").unwrap(); // Renamed to compliance_audit.db
        println!("Vascular Integrity System flowing — immutable truth.");
        
        // Load last hash from chain
        let last_hash = Self::load_last_hash(&db);
        
        Self {
            db: Arc::new(db),
            last_hash: Arc::new(std::sync::Mutex::new(last_hash)),
        }
    }

    fn load_last_hash(db: &Db) -> Option<Vec<u8>> {
        // Get the last entry's hash
        db.iter()
            .last()
            .and_then(|entry| entry.ok())
            .and_then(|(_, value)| {
                // Extract hash from stored entry (format: hash|event)
                let data = value.to_vec();
                if data.len() > 64 {
                    Some(data[..64].to_vec())
                } else {
                    None
                }
            })
    }

    fn compute_hash(&self, event: &str, previous_hash: Option<&[u8]>) -> Vec<u8> {
        let mut hasher = Sha256::new();
        
        if let Some(prev) = previous_hash {
            hasher.update(prev);
        }
        
        hasher.update(event.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        hasher.finalize().to_vec()
    }

    pub fn log_event(&self, event: &str) -> Result<(), sled::Error> {
        let timestamp = Utc::now().timestamp();
        
        // Get previous hash for chain
        let previous_hash = {
            let last = self.last_hash.lock().unwrap();
            last.clone()
        };
        
        // Compute new hash
        let new_hash = self.compute_hash(event, previous_hash.as_deref());
        
        // Store: hash|timestamp|event
        let mut stored_value = new_hash.clone();
        stored_value.extend_from_slice(b"|");
        stored_value.extend_from_slice(timestamp.to_string().as_bytes());
        stored_value.extend_from_slice(b"|");
        stored_value.extend_from_slice(event.as_bytes());
        
        self.db.insert(timestamp.to_string().as_bytes(), stored_value)?;
        self.db.flush()?;
        
        // Update last hash
        {
            let mut last = self.last_hash.lock().unwrap();
            *last = Some(new_hash);
        }
        
        println!("Event logged (tamper-proof): {}", event);
        Ok(())
    }

    pub fn verify_integrity(&self) -> Result<bool, sled::Error> {
        // Verify hash chain integrity
        let mut previous_hash: Option<Vec<u8>> = None;
        let mut is_valid = true;
        
        for entry in self.db.iter() {
            let (_, value) = entry?;
            let data = value.to_vec();
            
            if data.len() < 64 {
                is_valid = false;
                break;
            }
            
            let stored_hash = &data[..64];
            let rest = &data[65..]; // Skip '|'
            
            // Find event part
            let parts: Vec<&[u8]> = rest.split(|&b| b == b'|').collect();
            if parts.len() < 2 {
                is_valid = false;
                break;
            }
            
            let event = String::from_utf8_lossy(parts[1]);
            let computed_hash = self.compute_hash(&event, previous_hash.as_deref());
            
            if computed_hash != stored_hash {
                is_valid = false;
                break;
            }
            
            previous_hash = Some(computed_hash);
        }
        
        Ok(is_valid)
    }
}
