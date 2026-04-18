//! Lacoder agent runtime.
//!
//! Currently exports two interface layers:
//! - `provider` — the `LlmProvider` trait + provider implementations
//!   (Anthropic, OpenAI, plus an always-available `StubProvider`).
//! - `tools`    — the `Tool` trait + a `ToolRegistry`, with built-in
//!   `read_file`, `write_file`, `shell`, and `search` tool stubs.
//!
//! The agentic loop, the out-of-process runner, and the lsp-passthrough tool
//! land in follow-up commits.

pub mod provider;
pub mod tools;
pub mod wire;

pub use provider::{
    LlmProvider, ProviderError, ProviderId, ProviderMessage, ProviderRequest,
    ProviderResponse, ProviderRole, StubProvider,
};
pub use tools::{Tool, ToolError, ToolInvocation, ToolRegistry, ToolResult};
