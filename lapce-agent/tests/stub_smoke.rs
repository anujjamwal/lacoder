//! Smoke tests for the default-feature surface.

use lapce_agent::{
    LlmProvider, ProviderId, ProviderMessage, ProviderRequest, ProviderRole,
    StubProvider, ToolError, ToolInvocation, ToolRegistry,
};

#[test]
fn stub_provider_round_trip() {
    let p = StubProvider;
    assert_eq!(p.id(), ProviderId::Stub);

    let resp = p
        .complete(ProviderRequest {
            system: Some("you are a stub".into()),
            messages: vec![ProviderMessage {
                role: ProviderRole::User,
                content: "ping".into(),
            }],
            model: "stub-1".into(),
            max_tokens: 64,
            temperature: None,
        })
        .expect("stub never errors");

    assert!(resp.content.contains("[stub:stub-1]"));
    assert!(resp.content.contains("4 chars"), "content: {}", resp.content);
}

#[test]
fn tool_registry_routes_and_stubs_error() {
    let registry = ToolRegistry::with_builtins();
    let mut names = registry.names();
    names.sort();
    assert_eq!(names, vec!["read_file", "search", "shell", "write_file"]);

    let err = registry
        .invoke(ToolInvocation {
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "src/lib.rs"}),
        })
        .err()
        .expect("stub returns NotImplemented");
    assert!(matches!(err, ToolError::NotImplemented("read_file")));

    let err = registry
        .invoke(ToolInvocation {
            name: "does_not_exist".into(),
            arguments: serde_json::json!({}),
        })
        .err()
        .expect("missing tool");
    assert!(matches!(err, ToolError::NotFound(_)));
}
