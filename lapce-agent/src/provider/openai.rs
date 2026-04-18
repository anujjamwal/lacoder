//! OpenAI Chat-Completions API provider (feature-gated).

use serde::{Deserialize, Serialize};

use super::{
    LlmProvider, ProviderError, ProviderId, ProviderMessage, ProviderRequest,
    ProviderResponse, ProviderRole,
};

const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAiProvider {
    pub api_key: String,
    pub endpoint: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            endpoint: DEFAULT_ENDPOINT.to_string(),
        }
    }
}

impl LlmProvider for OpenAiProvider {
    fn id(&self) -> ProviderId {
        ProviderId::OpenAi
    }

    fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        let body = WireRequest::from(&request);
        let client = reqwest::blocking::Client::new();

        let resp = client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(ProviderError::Auth(
                ProviderId::OpenAi,
                format!("HTTP {status}"),
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited(ProviderId::OpenAi));
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
    messages: Vec<WireMessage<'a>>,
}

#[derive(Serialize)]
struct WireMessage<'a> {
    role: &'static str,
    content: &'a str,
}

impl<'a> From<&'a ProviderRequest> for WireRequest<'a> {
    fn from(r: &'a ProviderRequest) -> Self {
        let mut messages: Vec<WireMessage<'a>> = Vec::new();
        if let Some(sys) = r.system.as_deref() {
            messages.push(WireMessage {
                role: "system",
                content: sys,
            });
        }
        for m in &r.messages {
            messages.push(map_message(m));
        }
        Self {
            model: &r.model,
            max_tokens: r.max_tokens,
            temperature: r.temperature,
            messages,
        }
    }
}

fn map_message(m: &ProviderMessage) -> WireMessage<'_> {
    let role = match m.role {
        ProviderRole::System => "system",
        ProviderRole::User => "user",
        ProviderRole::Assistant => "assistant",
        ProviderRole::Tool => "tool",
    };
    WireMessage {
        role,
        content: &m.content,
    }
}

#[derive(Deserialize)]
struct WireResponse {
    choices: Vec<WireChoice>,
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct WireChoice {
    message: WireResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct WireResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct WireUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl From<WireResponse> for ProviderResponse {
    fn from(r: WireResponse) -> Self {
        let first = r.choices.into_iter().next();
        let (content, stop_reason) = match first {
            Some(c) => (c.message.content.unwrap_or_default(), c.finish_reason),
            None => (String::new(), None),
        };
        ProviderResponse {
            content,
            stop_reason,
            usage_input_tokens: r.usage.as_ref().map(|u| u.prompt_tokens),
            usage_output_tokens: r.usage.map(|u| u.completion_tokens),
        }
    }
}
