//! AI Validation module for TrinityChain
//!
//! Provides optional AI-assisted validation for transactions with proper safeguards,
//! rate limiting, caching, and fallback mechanisms.
//!
//! **CRITICAL SECURITY NOTE**: AI validation should NEVER be the sole validation mechanism.
//! This module provides supplementary heuristic analysis only. Always perform
//! deterministic cryptographic and rule-based validation first.

use crate::error::ChainError;
use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Constants
const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_TOKENS: u32 = 10;
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour
const MAX_CACHE_SIZE: usize = 1000;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(500);

// Rate limiting
const REQUESTS_PER_MINUTE: u32 = 60;
const REQUESTS_PER_HOUR: u32 = 1000;
const COST_BUDGET_PER_DAY: f64 = 10.0; // USD

/// DeepSeek API request format (correct structure)
#[derive(Debug, Clone, Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// DeepSeek API response format
#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    id: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Validation result with confidence score
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence: f32,
    pub reason: Option<String>,
    pub cached: bool,
}

/// Cached validation entry
#[derive(Debug, Clone)]
struct CacheEntry {
    result: ValidationResult,
    timestamp: Instant,
}

/// Rate limiter state
#[derive(Debug)]
struct RateLimitState {
    minute_count: u32,
    minute_reset: Instant,
    hour_count: u32,
    hour_reset: Instant,
    daily_cost: f64,
    day_reset: SystemTime,
}

impl RateLimitState {
    fn new() -> Self {
        let now = Instant::now();
        RateLimitState {
            minute_count: 0,
            minute_reset: now + Duration::from_secs(60),
            hour_count: 0,
            hour_reset: now + Duration::from_secs(3600),
            daily_cost: 0.0,
            day_reset: SystemTime::now() + Duration::from_secs(86400),
        }
    }

    fn check_and_increment(&mut self) -> Result<(), ChainError> {
        let now = Instant::now();
        let sys_now = SystemTime::now();

        // Reset counters if time windows have passed
        if now >= self.minute_reset {
            self.minute_count = 0;
            self.minute_reset = now + Duration::from_secs(60);
        }

        if now >= self.hour_reset {
            self.hour_count = 0;
            self.hour_reset = now + Duration::from_secs(3600);
        }

        if sys_now >= self.day_reset {
            self.daily_cost = 0.0;
            self.day_reset = sys_now + Duration::from_secs(86400);
        }

        // Check limits
        if self.minute_count >= REQUESTS_PER_MINUTE {
            return Err(ChainError::ApiError(
                "Rate limit exceeded: too many requests per minute".to_string()
            ));
        }

        if self.hour_count >= REQUESTS_PER_HOUR {
            return Err(ChainError::ApiError(
                "Rate limit exceeded: too many requests per hour".to_string()
            ));
        }

        if self.daily_cost >= COST_BUDGET_PER_DAY {
            return Err(ChainError::ApiError(
                format!("Daily cost budget exceeded: ${:.2}", COST_BUDGET_PER_DAY)
            ));
        }

        // Increment counters
        self.minute_count += 1;
        self.hour_count += 1;

        Ok(())
    }

    fn add_cost(&mut self, cost: f64) {
        self.daily_cost += cost;
    }
}

/// AI Validator with production-grade features
pub struct AIValidator {
    client: Client,
    api_key: String,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    rate_limiter: Arc<RwLock<RateLimitState>>,
    enabled: bool,
    model: String,
}

impl AIValidator {
    /// Create a new AI validator
    ///
    /// **SECURITY WARNING**: Store API keys securely, never in source code!
    /// Use environment variables or secure key management systems.
    pub fn new(api_key: String) -> Self {
        Self::with_config(api_key, true, "deepseek-chat".to_string())
    }

    /// Create a new AI validator with custom configuration
    pub fn with_config(api_key: String, enabled: bool, model: String) -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .unwrap_or_else(|_| Client::new());

        AIValidator {
            client,
            api_key,
            cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimitState::new())),
            enabled,
            model,
        }
    }

    /// Create a disabled validator (for testing or when AI is not needed)
    pub fn disabled() -> Self {
        Self::with_config(String::new(), false, String::new())
    }

    /// Check if AI validation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable AI validation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Validate a transaction with AI assistance
    ///
    /// **CRITICAL**: This should only be used as a supplementary check.
    /// Always perform deterministic validation first!
    pub async fn validate_transaction(
        &self,
        transaction_data: &str
    ) -> Result<ValidationResult, ChainError> {
        // If disabled, return neutral result
        if !self.enabled {
            return Ok(ValidationResult {
                is_valid: true,
                confidence: 0.0,
                reason: Some("AI validation disabled".to_string()),
                cached: false,
            });
        }

        // Check cache first
        let cache_key = self.compute_cache_key(transaction_data);
        if let Some(cached) = self.get_cached(&cache_key) {
            return Ok(cached);
        }

        // Check rate limits
        {
            let mut limiter = self.rate_limiter.write();
            limiter.check_and_increment()?;
        }

        // Perform validation with retry logic
        let result = self.validate_with_retry(transaction_data).await?;

        // Cache the result
        self.cache_result(&cache_key, result.clone());

        Ok(result)
    }

    /// Validate with automatic retry on transient failures
    async fn validate_with_retry(
        &self,
        transaction_data: &str
    ) -> Result<ValidationResult, ChainError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < MAX_RETRIES {
            match self.validate_with_api(transaction_data).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < MAX_RETRIES {
                        tokio::time::sleep(RETRY_DELAY * attempts).await;
                    }
                }
            }
        }

        // All retries failed, return fallback
        Err(last_error.unwrap_or_else(|| {
            ChainError::ApiError("Validation failed after retries".to_string())
        }))
    }

    /// Perform actual API validation
    async fn validate_with_api(
        &self,
        transaction_data: &str
    ) -> Result<ValidationResult, ChainError> {
        let prompt = self.build_validation_prompt(transaction_data);

        let request_body = DeepSeekRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a blockchain transaction validator. Analyze transactions for potential issues like invalid formats, suspicious patterns, or anomalies. Respond with 'VALID' or 'INVALID' followed by a brief reason.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: 0.1, // Low temperature for deterministic responses
            stream: Some(false),
        };

        let response = self.client
            .post(DEEPSEEK_API_URL)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChainError::ApiError(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ChainError::ApiError(
                format!("API returned {}: {}", status, error_text)
            ));
        }

        let response_body: DeepSeekResponse = response
            .json()
            .await
            .map_err(|e| ChainError::ApiError(format!("Failed to parse response: {}", e)))?;

        // Track costs
        self.track_usage(&response_body.usage);

        // Parse the AI response
        self.parse_validation_response(&response_body)
    }

    /// Build a structured validation prompt
    fn build_validation_prompt(&self, transaction_data: &str) -> String {
        format!(
            "Analyze this TrinityChain transaction:\n\n{}\n\nCheck for:\n\
             1. Valid format and structure\n\
             2. Reasonable values and amounts\n\
             3. Suspicious patterns or anomalies\n\n\
             Respond with 'VALID' or 'INVALID' followed by a brief reason.",
            transaction_data
        )
    }

    /// Parse the AI's validation response
    fn parse_validation_response(
        &self,
        response: &DeepSeekResponse
    ) -> Result<ValidationResult, ChainError> {
        let choice = response.choices.first()
            .ok_or_else(|| ChainError::ApiError("No response choices".to_string()))?;

        let content = choice.message.content.trim();
        let is_valid = content.to_uppercase().starts_with("VALID");
        
        // Extract reason (everything after first line or after "VALID"/"INVALID")
        let reason = if content.contains('\n') {
            Some(content.lines().skip(1).collect::<Vec<_>>().join(" ").trim().to_string())
        } else if content.len() > 7 {
            Some(content[7..].trim().to_string())
        } else {
            None
        };

        // Confidence based on clarity of response
        let confidence = if content.to_uppercase().starts_with("VALID") 
            || content.to_uppercase().starts_with("INVALID") {
            0.8
        } else {
            0.5
        };

        Ok(ValidationResult {
            is_valid,
            confidence,
            reason,
            cached: false,
        })
    }

    /// Track API usage costs
    fn track_usage(&self, usage: &Usage) {
        // DeepSeek pricing (approximate, check current rates)
        const COST_PER_1K_TOKENS: f64 = 0.0001;
        
        let cost = (usage.total_tokens as f64 / 1000.0) * COST_PER_1K_TOKENS;
        
        let mut limiter = self.rate_limiter.write();
        limiter.add_cost(cost);
    }

    /// Compute cache key for transaction data
    fn compute_cache_key(&self, transaction_data: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(transaction_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cached validation result
    fn get_cached(&self, key: &str) -> Option<ValidationResult> {
        let mut cache = self.cache.write();
        
        if let Some(entry) = cache.get(key) {
            // Check if cache entry is still valid
            if entry.timestamp.elapsed() < CACHE_TTL {
                let mut result = entry.result.clone();
                result.cached = true;
                return Some(result);
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        
        None
    }

    /// Cache a validation result
    fn cache_result(&self, key: &str, result: ValidationResult) {
        let mut cache = self.cache.write();
        
        // Limit cache size
        if cache.len() >= MAX_CACHE_SIZE {
            // Remove oldest entries (simple FIFO, could be improved with LRU)
            let keys_to_remove: Vec<_> = cache.keys()
                .take(MAX_CACHE_SIZE / 10)
                .cloned()
                .collect();
            
            for k in keys_to_remove {
                cache.remove(&k);
            }
        }
        
        cache.insert(key.to_string(), CacheEntry {
            result,
            timestamp: Instant::now(),
        });
    }

    /// Clear the validation cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read();
        let total = cache.len();
        let valid = cache.values()
            .filter(|e| e.timestamp.elapsed() < CACHE_TTL)
            .count();
        (valid, total)
    }

    /// Get rate limit statistics
    pub fn rate_limit_stats(&self) -> RateLimitStats {
        let limiter = self.rate_limiter.read();
        RateLimitStats {
            requests_this_minute: limiter.minute_count,
            requests_this_hour: limiter.hour_count,
            cost_today: limiter.daily_cost,
        }
    }
}

/// Rate limit statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub requests_this_minute: u32,
    pub requests_this_hour: u32,
    pub cost_today: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_validator() {
        let validator = AIValidator::disabled();
        assert!(!validator.is_enabled());
    }

    #[test]
    fn test_cache_key_generation() {
        let validator = AIValidator::disabled();
        let key1 = validator.compute_cache_key("test data");
        let key2 = validator.compute_cache_key("test data");
        let key3 = validator.compute_cache_key("different data");
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_rate_limiter_initialization() {
        let limiter = RateLimitState::new();
        assert_eq!(limiter.minute_count, 0);
        assert_eq!(limiter.hour_count, 0);
        assert_eq!(limiter.daily_cost, 0.0);
    }

    #[test]
    fn test_rate_limiter_increment() {
        let mut limiter = RateLimitState::new();
        
        // Should succeed
        assert!(limiter.check_and_increment().is_ok());
        assert_eq!(limiter.minute_count, 1);
        assert_eq!(limiter.hour_count, 1);
    }

    #[test]
    fn test_rate_limiter_minute_limit() {
        let mut limiter = RateLimitState::new();
        
        // Fill up to limit
        for _ in 0..REQUESTS_PER_MINUTE {
            assert!(limiter.check_and_increment().is_ok());
        }
        
        // Next should fail
        assert!(limiter.check_and_increment().is_err());
    }

    #[test]
    fn test_cache_operations() {
        let validator = AIValidator::disabled();
        let key = "test_key";
        
        // Initially empty
        assert!(validator.get_cached(key).is_none());
        
        // Cache a result
        let result = ValidationResult {
            is_valid: true,
            confidence: 0.9,
            reason: Some("Test".to_string()),
            cached: false,
        };
        
        validator.cache_result(key, result.clone());
        
        // Should retrieve cached result
        let cached = validator.get_cached(key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().is_valid, true);
    }

    #[test]
    fn test_parse_validation_response_valid() {
        let validator = AIValidator::disabled();
        
        let response = DeepSeekResponse {
            id: "test".to_string(),
            choices: vec![Choice {
                message: MessageResponse {
                    role: "assistant".to_string(),
                    content: "VALID - Transaction structure looks good".to_string(),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };
        
        let result = validator.parse_validation_response(&response).unwrap();
        assert!(result.is_valid);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn test_parse_validation_response_invalid() {
        let validator = AIValidator::disabled();
        
        let response = DeepSeekResponse {
            id: "test".to_string(),
            choices: vec![Choice {
                message: MessageResponse {
                    role: "assistant".to_string(),
                    content: "INVALID - Suspicious amount detected".to_string(),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };
        
        let result = validator.parse_validation_response(&response).unwrap();
        assert!(!result.is_valid);
        assert!(result.reason.is_some());
    }

    #[tokio::test]
    async fn test_disabled_validation() {
        let validator = AIValidator::disabled();
        let result = validator.validate_transaction("test tx").await.unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.confidence, 0.0);
        assert!(result.reason.is_some());
    }
}
