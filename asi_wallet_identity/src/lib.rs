// asi_wallet_identity/src/lib.rs
// Wallet-based identity (stub) for ASI-style, AI-native deployment.
//
// We intentionally keep this lightweight and non-invasive.
// It is a handle for future X402/crypto integrations (signing, payments, identity).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletIdentity {
    pub enabled: bool,
    pub chain: String,
    pub wallet_address: String,
    pub x402_premium_key: Option<String>,
}

impl WalletIdentity {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let enabled = std::env::var("ASI_WALLET_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let chain = std::env::var("ASI_WALLET_CHAIN").unwrap_or_else(|_| "eip155:1".to_string());

        let wallet_address = std::env::var("PHOENIX_WALLET_ADDRESS").unwrap_or_else(|_| {
            // Stub: generate a stable-ish pseudo-identity.
            // For real use, provide an actual wallet address.
            format!("phoenix://wallet/{}", uuid::Uuid::new_v4())
        });

        let x402_premium_key = std::env::var("X402_PREMIUM_KEY").ok().filter(|s| !s.is_empty());

        Self {
            enabled,
            chain,
            wallet_address,
            x402_premium_key,
        }
    }

    pub fn as_prompt_tag(&self) -> String {
        if !self.enabled {
            return "ASI_WALLET_ENABLED=false (identity offline)".to_string();
        }
        format!(
            "ASI identity: wallet_address={addr} chain={chain} x402={tier}",
            addr = self.wallet_address,
            chain = self.chain,
            tier = if self.x402_premium_key.is_some() { "present" } else { "none" }
        )
    }

    pub fn x402_header_value(&self) -> Option<String> {
        self.x402_premium_key.clone()
    }
}

