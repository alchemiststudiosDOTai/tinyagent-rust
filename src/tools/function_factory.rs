use super::{tool::ToolRegistry, Tool};
use crate::{AgentError, Result};
use serde_json::Value;

/// Factory for creating and managing function/tool execution
#[derive(Debug)]
pub struct FunctionFactory {
    registry: ToolRegistry,
}

impl FunctionFactory {
    /// Create a new function factory
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Register a tool with the factory
    pub fn register_tool<T: Tool + 'static>(&mut self, tool: T) {
        self.registry.register(tool);
    }

    /// Execute a function call by name
    pub async fn execute_function(&self, function_name: &str, parameters: Value) -> Result<Value> {
        let tool = self
            .registry
            .get(function_name)
            .ok_or_else(|| AgentError::ToolNotFound(function_name.to_string()))?;

        tool.execute(parameters).await
    }

    /// Get all available tools for OpenAI function calling
    pub fn get_openai_tools(&self) -> Vec<Value> {
        self.registry.to_openai_tools()
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.registry.get(name).is_some()
    }
}

impl Default for FunctionFactory {
    fn default() -> Self {
        Self::new()
    }
}
