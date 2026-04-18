//! Anthropic Messages API provider (feature-gated).
//!
//! Phase 2.3 ships the synchronous one-shot path. Streaming + tool_use
//! responses arrive with the full agentic loop. Auth: ANTHROPIC_API_KEY
//! from the env or a key configured via `LapceConfig.plugins["agent"]`.

use serde::{Deserialize, Serialize};

use super::{
    LlmProvider, ProviderError, ProviderId, ProviderMessage, ProviderRequest,
    ProviderResponse, ProviderRole,
};

const DEFAULT_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    pub api_key: String,
    pub endpoint: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            endpoint: DEFAULT_ENDPOINT.to_string(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Anthropic
    }

    fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        let body = WireRequest::from(&request);
        let client = reqwest::blocking::Client::new();

        let resp = client
            .post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(ProviderError::Auth(
                ProviderId::Anthropic,
                format!("HTTP {status}"),
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited(ProviderId::Anthropic));
        }
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ProviderError::Other(format!("HTTP {status}: {body}")));
        }

        let parsed: WireResponse = resp
            .json()
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        Ok(parsed.into())
    }
}

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<WireMessage<'a>>,
}

#[derive(Serialize)]
struct WireMessage<'a> {
    role: &'static str,
    content: &'a str,
}

impl<'a> From<&'a ProviderRequest> for WireRequest<'a> {
    fn from(r: &'a ProviderRequest) -> Self {
        // Anthropic's Messages API only accepts user/assistant turns; system
        // is a top-level field. Drop ProviderRole::System / Tool entries from
        // the messages array (system is forwarded once via `r.system`; tool
        // turns get a future channel once tool_use lands).
        let messages = r
            .messages
            .iter()
            .filter_map(map_role)
            .collect();
        Self {
            model: &r.model,
            max_tokens: r.max_tokens,
            temperature: r.temperature,
            system: r.system.as_deref(),
            messages,
        }
    }
}

fn map_role(m: &ProviderMessage) -> Option<WireMessage<'_>> {
    let role = match m.role {
        ProviderRole::User => "user",
        ProviderRole::Assistant => "assistant",
        ProviderRole::System | ProviderRole::Tool => return None,
    };
    Some(WireMessage {
        role,
        content: &m.content,
    })
}

#[derive(Deserialize)]
struct WireResponse {
    content: Vec<WireContentBlock>,
    stop_reason: Option<String>,
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum WireContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct WireUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl From<WireResponse> for ProviderResponse {
    fn from(r: WireResponse) -> Self {
        let content = r
            .content
            .into_iter()
            .filter_map(|b| match b {
                WireContentBlock::Text { text } => Some(text),
                WireContentBlock::Other => None,
            })
            .collect::<Vec<_>>()
            .join("");
        ProviderResponse {
            content,
            stop_reason: r.stop_reason,
            usage_input_tokens: r.usage.as_ref().map(|u| u.input_tokens),
            usage_output_tokens: r.usage.map(|u| u.output_tokens),
        }
    }
}
