//! Always-available stub provider. Echoes structure of the request so we can
//! exercise the loop / view layer without network or API keys.

use super::{
    LlmProvider, ProviderError, ProviderId, ProviderRequest, ProviderResponse,
};

pub struct StubProvider;

impl LlmProvider for StubProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Stub
    }

    fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        let last_user = request
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, super::ProviderRole::User))
            .map(|m| m.content.as_str())
            .unwrap_or("");

        Ok(ProviderResponse {
            content: format!(
                "[stub:{}] received {} message(s); last user turn was {} chars",
                request.model,
                request.messages.len(),
                last_user.len()
            ),
            stop_reason: Some("stop".into()),
            usage_input_tokens: None,
            usage_output_tokens: None,
        })
    }
}
