//! `LlmProvider` trait + impls.
//!
//! Provider impls (Anthropic, OpenAI) are gated behind feature flags so
//! the trait + stub compile without dragging in network deps.

#[cfg(feature = "anthropic")]
pub mod anthropic;
#[cfg(feature = "openai")]
pub mod openai;
pub mod stub;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use stub::StubProvider;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderId {
    Stub,
    Anthropic,
    OpenAi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderMessage {
    pub role: ProviderRole,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub system: Option<String>,
    pub messages: Vec<ProviderMessage>,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub content: String,
    pub stop_reason: Option<String>,
    pub usage_input_tokens: Option<u32>,
    pub usage_output_tokens: Option<u32>,
}

/// Synchronous one-shot request → response. Streaming + tool-use mid-stream
/// land with the full agentic loop in a follow-up.
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError>;
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider {0:?} feature not compiled in")]
    FeatureDisabled(ProviderId),

    #[error("auth failed for provider {0:?}: {1}")]
    Auth(ProviderId, String),

    #[error("rate limited by provider {0:?}")]
    RateLimited(ProviderId),

    #[error("network error: {0}")]
    Network(String),

    #[error("provider returned unparseable response: {0}")]
    Parse(String),

    #[error("provider error: {0}")]
    Other(String),
}
