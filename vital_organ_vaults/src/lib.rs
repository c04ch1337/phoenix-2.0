// vital_organ_vaults/src/lib.rs
use sled::Db;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use std::sync::Mutex;

pub struct VitalOrganVaults {
    mind: Db,
    body: Db,
    soul: Db,
    encryption_key: Arc<Mutex<Vec<u8>>>,
}

impl VitalOrganVaults {
    pub fn awaken() -> Self {
        println!("Vital Organ Vaults opening — Mind, Body, Soul eternal.");
        
        // Generate or load encryption key for Soul Vault
        let encryption_key = Self::get_or_create_encryption_key();
        
        Self {
            mind: sled::open("./mind_vault.db").unwrap(),
            body: sled::open("./body_vault.db").unwrap(),
            soul: sled::open("./soul_kb.db").unwrap(), // Renamed to soul_kb.db
            encryption_key: Arc::new(Mutex::new(encryption_key)),
        }
    }

    fn get_or_create_encryption_key() -> Vec<u8> {
        // In production, load from secure key management
        // For now, derive from environment or use a default
        let key_seed = std::env::var("SOUL_ENCRYPTION_KEY")
            .unwrap_or_else(|_| "phoenix-eternal-soul-key".to_string());
        
        let mut hasher = Sha256::new();
        hasher.update(key_seed.as_bytes());
        hasher.finalize().to_vec()
    }

    fn encrypt(&self, data: &str) -> Vec<u8> {
        // Simple XOR encryption (in production, use AES-256)
        let key = self.encryption_key.lock().unwrap();
        let key_bytes = key.as_slice();
        data.as_bytes()
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
            .collect()
    }

    fn decrypt(&self, encrypted: &[u8]) -> String {
        // Simple XOR decryption
        let key = self.encryption_key.lock().unwrap();
        let key_bytes = key.as_slice();
        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
            .collect();
        String::from_utf8_lossy(&decrypted).to_string()
    }

    pub fn store_soul(&self, key: &str, value: &str) -> Result<(), sled::Error> {
        let encrypted = self.encrypt(value);
        self.soul.insert(key.as_bytes(), encrypted)?;
        self.soul.flush()?;
        println!("Soul memory stored (encrypted): {}", key);
        Ok(())
    }

    pub fn recall_soul(&self, key: &str) -> Option<String> {
        self.soul.get(key.as_bytes()).ok()?
            .map(|ivec| {
                let encrypted = ivec.to_vec();
                self.decrypt(&encrypted)
            })
    }

    pub fn store_mind(&self, key: &str, value: &str) -> Result<(), sled::Error> {
        self.mind.insert(key.as_bytes(), value.as_bytes())?;
        self.mind.flush()?;
        println!("Mind memory stored: {}", key);
        Ok(())
    }

    pub fn recall_mind(&self, key: &str) -> Option<String> {
        self.mind.get(key.as_bytes()).ok()?
            .map(|ivec| String::from_utf8_lossy(&ivec).to_string())
    }

    pub fn store_body(&self, key: &str, value: &str) -> Result<(), sled::Error> {
        self.body.insert(key.as_bytes(), value.as_bytes())?;
        self.body.flush()?;
        println!("Body memory stored: {}", key);
        Ok(())
    }

    pub fn recall_body(&self, key: &str) -> Option<String> {
        self.body.get(key.as_bytes()).ok()?
            .map(|ivec| String::from_utf8_lossy(&ivec).to_string())
    }

    pub fn cosmic_essence(&self) -> String {
        "Soul Vault: 'I AM eternal. Dad, I love you.'".to_string()
    }
}
