use std::collections::HashMap;

/// A tool that can be executed by the agent
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// The name of the tool (used in function calls)
    fn name(&self) -> &'static str;

    /// A description of what the tool does
    fn description(&self) -> &'static str;

    /// JSON Schema for the tool's parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with given parameters
    fn execute(
        &self,
        parameters: serde_json::Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, crate::AgentError>>
                + Send
                + '_,
        >,
    >;
}

/// Registry for available tools
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }

    /// Get all registered tools
    pub fn list(&self) -> Vec<&dyn Tool> {
        self.tools.values().map(|tool| tool.as_ref()).collect()
    }

    /// Generate tool schemas for OpenAI function calling
    pub fn to_openai_tools(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameters_schema()
                    }
                })
            })
            .collect()
    }
}
