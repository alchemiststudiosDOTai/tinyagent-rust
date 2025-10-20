use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a single step in the agent's reasoning process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentStep {
    /// Initial task provided by the user
    Task { content: String },
    /// Planning step where the agent thinks about how to approach the task
    Planning { plan: String },
    /// Action step where the agent calls a tool
    Action {
        tool_name: String,
        tool_call_id: String,
        arguments: Value,
    },
    /// Observation step capturing the result of a tool execution
    Observation {
        tool_call_id: String,
        result: String,
        is_error: bool,
    },
    /// Final answer from the agent
    FinalAnswer {
        answer: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        structured: Option<Value>,
    },
}

impl AgentStep {
    /// Convert step to OpenAI message format
    pub fn to_message(&self) -> Value {
        match self {
            AgentStep::Task { content } => {
                serde_json::json!({
                    "role": "user",
                    "content": content
                })
            }
            AgentStep::Planning { plan } => {
                serde_json::json!({
                    "role": "assistant",
                    "content": plan
                })
            }
            AgentStep::Action {
                tool_name,
                tool_call_id,
                arguments,
            } => {
                serde_json::json!({
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": tool_call_id,
                        "type": "function",
                        "function": {
                            "name": tool_name,
                            "arguments": serde_json::to_string(arguments).unwrap_or_default()
                        }
                    }]
                })
            }
            AgentStep::Observation {
                tool_call_id,
                result,
                ..
            } => {
                serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tool_call_id,
                    "content": result
                })
            }
            AgentStep::FinalAnswer { answer, .. } => {
                serde_json::json!({
                    "role": "assistant",
                    "content": answer
                })
            }
        }
    }

    /// Get a human-readable description of the step
    pub fn describe(&self) -> String {
        match self {
            AgentStep::Task { content } => format!("🧭 Task: {}", content),
            AgentStep::Planning { plan } => format!("🧩 Plan: {}", plan),
            AgentStep::Action {
                tool_name,
                arguments,
                ..
            } => {
                format!("🔧 Action: {}({})", tool_name, arguments)
            }
            AgentStep::Observation {
                result, is_error, ..
            } => {
                if *is_error {
                    format!("❌ Error: {}", result)
                } else {
                    format!("👁 Observation: {}", result)
                }
            }
            AgentStep::FinalAnswer { answer, .. } => format!("✅ Final Answer: {}", answer),
        }
    }
}
