use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};

/// Represents a tool call request from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Name of the tool to execute
    pub name: String,
    /// Arguments to pass to the tool
    pub arguments: Value,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: String, name: String, arguments: Value) -> Self {
        Self {
            id,
            name,
            arguments,
        }
    }

    /// Parse a tool call from OpenAI response format
    pub fn from_openai_format(tool_call: &Value) -> Option<Self> {
        let id = tool_call.get("id")?.as_str()?.to_string();
        let function = tool_call.get("function")?;
        let name = function.get("name")?.as_str()?.to_string();

        let arguments_str = function.get("arguments")?.as_str()?;
        let arguments: Value = serde_json::from_str(arguments_str).ok()?;

        Some(Self {
            id,
            name,
            arguments,
        })
    }

    /// Convert to OpenAI tool call format
    pub fn to_openai_format(&self) -> Value {
        serde_json::json!({
            "id": self.id,
            "type": "function",
            "function": {
                "name": self.name,
                "arguments": serde_json::to_string(&self.arguments).unwrap_or_default()
            }
        })
    }

    /// Get a human-readable description
    pub fn describe(&self) -> String {
        format!("{}({})", self.name, self.arguments)
    }
}

/// Represents the output from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// The tool call ID this output corresponds to
    pub tool_call_id: String,
    /// The tool name that was executed
    pub tool_name: String,
    /// The output/result from the tool
    pub output: Value,
    /// Whether this is the final answer
    pub is_final: bool,
    /// Whether the execution resulted in an error
    pub is_error: bool,
    /// Execution duration in milliseconds
    pub duration_ms: Option<u128>,
}

impl ToolOutput {
    /// Create a successful tool output
    pub fn success(tool_call_id: String, tool_name: String, output: Value) -> Self {
        Self {
            tool_call_id,
            tool_name,
            output,
            is_final: false,
            is_error: false,
            duration_ms: None,
        }
    }

    /// Create an error tool output
    pub fn error(tool_call_id: String, tool_name: String, error_msg: String) -> Self {
        Self {
            tool_call_id,
            tool_name,
            output: serde_json::json!({
                "error": {
                    "message": error_msg
                }
            }),
            is_final: false,
            is_error: true,
            duration_ms: None,
        }
    }

    /// Set the execution duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = Some(duration.as_millis());
        self
    }

    /// Mark this as a final answer
    pub fn as_final(mut self) -> Self {
        self.is_final = true;
        self
    }

    /// Get the output as a string for message content
    pub fn as_string(&self) -> String {
        match &self.output {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    /// Convert to OpenAI tool message format
    pub fn to_openai_message(&self) -> Value {
        serde_json::json!({
            "role": "tool",
            "tool_call_id": self.tool_call_id,
            "content": self.as_string()
        })
    }
}

/// Tracks the execution of a tool call with timing information
#[derive(Debug)]
pub struct ToolExecution {
    pub tool_call: ToolCall,
    start_time: Instant,
}

impl ToolExecution {
    /// Start tracking a tool execution
    pub fn start(tool_call: ToolCall) -> Self {
        Self {
            tool_call,
            start_time: Instant::now(),
        }
    }

    /// Complete the execution and get the output with timing
    pub fn complete(self, output: Value, is_error: bool) -> ToolOutput {
        let duration = self.start_time.elapsed();
        ToolOutput {
            tool_call_id: self.tool_call.id,
            tool_name: self.tool_call.name,
            output,
            is_final: false,
            is_error,
            duration_ms: Some(duration.as_millis()),
        }
    }

    /// Complete with error
    pub fn complete_with_error(self, error_msg: String) -> ToolOutput {
        let duration = self.start_time.elapsed();
        ToolOutput::error(self.tool_call.id, self.tool_call.name, error_msg).with_duration(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall::new(
            "call_123".to_string(),
            "test_tool".to_string(),
            serde_json::json!({"arg": "value"}),
        );
        assert_eq!(call.id, "call_123");
        assert_eq!(call.name, "test_tool");
    }

    #[test]
    fn test_tool_call_from_openai() {
        let openai_format = serde_json::json!({
            "id": "call_456",
            "type": "function",
            "function": {
                "name": "calculator",
                "arguments": "{\"expression\": \"2+2\"}"
            }
        });

        let call = ToolCall::from_openai_format(&openai_format).unwrap();
        assert_eq!(call.id, "call_456");
        assert_eq!(call.name, "calculator");
        assert_eq!(call.arguments["expression"], "2+2");
    }

    #[test]
    fn test_tool_output_success() {
        let output = ToolOutput::success(
            "call_789".to_string(),
            "test".to_string(),
            serde_json::json!("result"),
        );
        assert!(!output.is_error);
        assert!(!output.is_final);
        assert_eq!(output.tool_call_id, "call_789");
    }

    #[test]
    fn test_tool_output_error() {
        let output = ToolOutput::error(
            "call_999".to_string(),
            "test".to_string(),
            "Something went wrong".to_string(),
        );
        assert!(output.is_error);
        assert_eq!(output.tool_name, "test");
    }

    #[test]
    fn test_tool_execution_timing() {
        let call = ToolCall::new("call_123".to_string(), "test".to_string(), Value::Null);
        let execution = ToolExecution::start(call);
        let output = execution.complete(serde_json::json!("result"), false);
        assert!(output.duration_ms.is_some());
    }
}
