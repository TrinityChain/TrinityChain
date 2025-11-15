//! AI Validation module for TrinityChain

use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::ChainError;

const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

#[derive(Serialize)]
struct ApiRequestBody {
    model: String,
    prompt: String,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ApiResponseBody {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

pub struct AIValidator {
    client: Client,
    api_key: String,
}

impl AIValidator {
    pub fn new(api_key: String) -> Self {
        AIValidator {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn validate_transaction(&self, transaction_data: &str) -> Result<bool, ChainError> {
        let prompt = format!("Is the following transaction valid for the TrinityChain network? Respond with only 'true' or 'false'.\n\n{}", transaction_data);

        let request_body = ApiRequestBody {
            model: "deepseek-coder".to_string(),
            prompt,
            max_tokens: 1,
        };

        let response = self.client.post(DEEPSEEK_API_URL)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChainError::ApiError(e.to_string()))?;

        if response.status().is_success() {
            let response_body: ApiResponseBody = response.json().await.map_err(|e| ChainError::ApiError(e.to_string()))?;
            if let Some(choice) = response_body.choices.get(0) {
                return Ok(choice.text.trim().eq_ignore_ascii_case("true"));
            }
        }
        
        Err(ChainError::ApiError("Failed to get a valid response from the API".to_string()))
    }
}
