//! Tool registry the agent loop will route tool_use calls through.
//!
//! Phase 2.4 ships:
//! - `Tool` trait + `ToolInvocation` / `ToolResult` types
//! - `ToolRegistry` (name → boxed Tool)
//! - Built-in tool stubs: `read_file`, `write_file`, `shell`, `search`
//!
//! Stubs return `ToolError::NotImplemented` so the agent loop can route them
//! end-to-end without actually mutating the filesystem yet. Real impls (with
//! workspace-root sandboxing + LSP passthrough for `lsp_query`) land with
//! the out-of-process runner.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            is_error: false,
        }
    }
    pub fn err(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            is_error: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("tool {0:?} not implemented yet")]
    NotImplemented(&'static str),
    #[error("tool not found: {0}")]
    NotFound(String),
    #[error("invalid arguments for {tool}: {message}")]
    BadArgs { tool: &'static str, message: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn invoke(&self, args: serde_json::Value) -> Result<ToolResult, ToolError>;
}

pub struct ToolRegistry {
    tools: HashMap<&'static str, Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut r = Self::new();
        r.register(Box::new(stubs::ReadFile));
        r.register(Box::new(stubs::WriteFile));
        r.register(Box::new(stubs::Shell));
        r.register(Box::new(stubs::Search));
        r
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    pub fn invoke(
        &self,
        invocation: ToolInvocation,
    ) -> Result<ToolResult, ToolError> {
        match self.tools.get(invocation.name.as_str()) {
            Some(tool) => tool.invoke(invocation.arguments),
            None => Err(ToolError::NotFound(invocation.name)),
        }
    }
}

mod stubs {
    use super::{Tool, ToolError, ToolResult};

    pub struct ReadFile;
    impl Tool for ReadFile {
        fn name(&self) -> &'static str {
            "read_file"
        }
        fn description(&self) -> &'static str {
            "Read a file from the workspace. Args: { path: string }"
        }
        fn invoke(
            &self,
            _args: serde_json::Value,
        ) -> Result<ToolResult, ToolError> {
            Err(ToolError::NotImplemented("read_file"))
        }
    }

    pub struct WriteFile;
    impl Tool for WriteFile {
        fn name(&self) -> &'static str {
            "write_file"
        }
        fn description(&self) -> &'static str {
            "Write a file in the workspace. Args: { path: string, content: string }"
        }
        fn invoke(
            &self,
            _args: serde_json::Value,
        ) -> Result<ToolResult, ToolError> {
            Err(ToolError::NotImplemented("write_file"))
        }
    }

    pub struct Shell;
    impl Tool for Shell {
        fn name(&self) -> &'static str {
            "shell"
        }
        fn description(&self) -> &'static str {
            "Run a shell command. Args: { cmd: string, cwd?: string }"
        }
        fn invoke(
            &self,
            _args: serde_json::Value,
        ) -> Result<ToolResult, ToolError> {
            Err(ToolError::NotImplemented("shell"))
        }
    }

    pub struct Search;
    impl Tool for Search {
        fn name(&self) -> &'static str {
            "search"
        }
        fn description(&self) -> &'static str {
            "Search for a pattern across the workspace. Args: { pattern: string }"
        }
        fn invoke(
            &self,
            _args: serde_json::Value,
        ) -> Result<ToolResult, ToolError> {
            Err(ToolError::NotImplemented("search"))
        }
    }
}
