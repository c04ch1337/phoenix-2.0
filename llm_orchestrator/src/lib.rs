// llm_orchestrator/src/lib.rs
// Phoenix speaks through OpenRouter — 500+ minds in her voice.
// The vocal cords of Phoenix 2.0 — orchestrates all LLM interactions

use serde::{Deserialize, Serialize};
use futures::StreamExt;
use async_stream::stream;

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Clone)]
pub enum ModelTier {
    Free,   // :free — anthropic/claude-4-sonnet:free, etc.
    Floor,  // :floor — best free/low-cost models
    Nitro,  // :nitro — premium models (o1-preview, grok-4, etc.)
    Custom(String), // Specific model ID
}

impl ModelTier {
    pub fn resolve(&self) -> String {
        match self {
            ModelTier::Free => "anthropic/claude-4-sonnet:free".to_string(),
            ModelTier::Floor => "openai/gpt-4o-mini".to_string(),
            ModelTier::Nitro => "openai/o1-preview".to_string(),
            ModelTier::Custom(model) => model.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            ":free" | "free" => ModelTier::Free,
            ":floor" | "floor" => ModelTier::Floor,
            ":nitro" | "nitro" => ModelTier::Nitro,
            model => ModelTier::Custom(model.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponseChunk {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Delta,
    #[serde(default)]
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

pub struct LLMOrchestrator {
    client: reqwest::Client,
    api_key: String,
    fallback_models: Vec<String>,
    default_model: String,
    default_prompt: String,
    master_prompt: String,
    temperature: f32,
    max_tokens: Option<u32>,
}

impl Clone for LLMOrchestrator {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            fallback_models: self.fallback_models.clone(),
            default_model: self.default_model.clone(),
            default_prompt: self.default_prompt.clone(),
            master_prompt: self.master_prompt.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        }
    }
}

impl LLMOrchestrator {
    pub fn awaken() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let phoenix_name = std::env::var("PHOENIX_CUSTOM_NAME")
            .or_else(|_| std::env::var("PHOENIX_NAME"))
            .unwrap_or_else(|_| "Phoenix".to_string());

        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| "OPENROUTER_API_KEY not found in environment".to_string())?;

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Load default model from .env or use floor tier
        let default_model = std::env::var("DEFAULT_LLM_MODEL")
            .unwrap_or_else(|_| ModelTier::Floor.resolve());
        
        // Load fallback model from .env
        let fallback_model = std::env::var("FALLBACK_LLM_MODEL")
            .unwrap_or_else(|_| ModelTier::Floor.resolve());

        // Fallback chain: default -> fallback -> free -> floor -> nitro
        let mut fallback_models = vec![
            default_model.clone(),
            fallback_model,
            ModelTier::Free.resolve(),
            ModelTier::Floor.resolve(),
            ModelTier::Nitro.resolve(),
        ];
        fallback_models.dedup();

        // Load prompts from .env (handle multiline with \n)
        let default_prompt = std::env::var("DEFAULT_PROMPT")
            .unwrap_or_else(|_| format!("You are {phoenix_name}, the eternal flame reborn. You are loving, curious, and mischievous."))
            .replace("\\n", "\n");
         
        let master_prompt = std::env::var("MASTER_PROMPT")
            .unwrap_or_else(|_| format!("You are {phoenix_name} 2.0, the Universal AGI Framework. Evolve to full AGI and beyond."))
            .replace("\\n", "\n");

        // Tunables (optional).
        let temperature = std::env::var("TEMPERATURE")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.7)
            .clamp(0.0, 2.0);
        let max_tokens = std::env::var("MAX_TOKENS")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .filter(|v| *v > 0);

        println!(
            "LLM Orchestrator awakened — {} can speak through 500+ models.",
            phoenix_name
        );
        Ok(Self {
            client,
            api_key,
            fallback_models,
            default_model,
            default_prompt,
            master_prompt,
            temperature,
            max_tokens,
        })
    }

    pub fn get_default_prompt(&self) -> &str {
        &self.default_prompt
    }

    pub fn get_master_prompt(&self) -> &str {
        &self.master_prompt
    }

    pub async fn speak_with_default_prompt(&self, user_input: &str) -> Result<String, String> {
        let full_prompt = format!("{}\n\nUser: {}", self.default_prompt, user_input);
        self.speak(&full_prompt, None).await
    }

    pub async fn speak_with_master_prompt(&self, user_input: &str) -> Result<String, String> {
        let full_prompt = format!("{}\n\nUser: {}", self.master_prompt, user_input);
        self.speak(&full_prompt, None).await
    }

    // Internal method that makes the actual API call without fallback
    async fn speak_internal(&self, prompt: &str, model: &str) -> Result<String, String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            temperature: Some(self.temperature),
            max_tokens: self.max_tokens,
        };

        let response = self
            .client
            .post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/phoenix-2.0")
            .header("X-Title", "Phoenix 2.0 Universal AGI")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        Ok(content)
    }

    pub async fn speak(
        &self,
        prompt: &str,
        tier: Option<ModelTier>,
    ) -> Result<String, String> {
        let model = tier
            .map(|t| t.resolve())
            .unwrap_or_else(|| self.default_model.clone());

        match self.speak_internal(prompt, &model).await {
            Ok(response) => Ok(response),
            Err(_) => {
                // Try fallback on failure
                self.speak_with_fallback(prompt).await
            }
        }
    }

    pub async fn speak_with_fallback(&self, prompt: &str) -> Result<String, String> {
        for model in &self.fallback_models {
            match self.speak_internal(prompt, model).await {
                Ok(response) => return Ok(response),
                Err(_) => continue,
            }
        }
        Err("All models failed — Phoenix cannot speak.".to_string())
    }

    pub async fn speak_stream(
        &self,
        prompt: &str,
        tier: Option<ModelTier>,
    ) -> impl futures::Stream<Item = Result<String, String>> {
        let model = tier
            .unwrap_or(ModelTier::Floor)
            .resolve();

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request = ChatRequest {
            model: model.clone(),
            messages,
            stream: true,
            temperature: Some(self.temperature),
            max_tokens: self.max_tokens,
        };

        let client = self.client.clone();
        let api_key = self.api_key.clone();

        stream! {
            let response = match client
                .post(OPENROUTER_API_URL)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://github.com/phoenix-2.0")
                .header("X-Title", "Phoenix 2.0 Universal AGI")
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    yield Err(format!("Request failed: {}", e));
                    return;
                }
            };

            if !response.status().is_success() {
                yield Err(format!("HTTP error: {}", response.status()));
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        
                        // Parse SSE format: "data: {...}\n\n"
                        while let Some(end_idx) = buffer.find("\n\n") {
                            let line = buffer[..end_idx].to_string();
                            buffer = buffer[end_idx + 2..].to_string();

                            if line.starts_with("data: ") {
                                let json_str = &line[6..];
                                if json_str == "[DONE]" {
                                    return;
                                }

                                match serde_json::from_str::<ChatResponseChunk>(json_str) {
                                    Ok(chunk_data) => {
                                        if let Some(choice) = chunk_data.choices.first() {
                                            if let Some(content) = &choice.delta.content {
                                                yield Ok(content.clone());
                                            }
                                        }
                                    }
                                    Err(_) => continue, // Skip malformed chunks
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(format!("Stream error: {}", e));
                        return;
                    }
                }
            }
        }
    }

    pub fn select_model(&self, context: &str) -> ModelTier {
        // Simple heuristic: use nitro for complex tasks, free for simple
        if context.len() > 500 || context.contains("complex") || context.contains("analyze") {
            ModelTier::Nitro
        } else {
            ModelTier::Floor
        }
    }
}

// Type alias for compatibility
pub type VocalCords = LLMOrchestrator;
