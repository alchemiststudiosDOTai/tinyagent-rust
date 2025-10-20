use crate::error::AgentError;
use serde_json::Value;

/// Extract tool_call_id from a tool call JSON object
pub(super) fn extract_tool_call_id(tool_call: &Value) -> &str {
    tool_call
        .get("id")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
}

/// Extract function name from a tool call JSON object
pub(super) fn extract_function_info(tool_call: &Value) -> Option<(Value, Option<String>)> {
    let function = tool_call.get("function").cloned()?;
    let function_name = function
        .get("name")
        .and_then(|value| value.as_str())
        .map(|s| s.to_string());
    Some((function, function_name))
}

/// Parse function arguments from JSON string
pub(super) fn parse_function_arguments(
    arguments_str: &str,
    function_name: &str,
) -> Result<Value, AgentError> {
    serde_json::from_str(arguments_str).map_err(|err| {
        AgentError::InvalidFunctionCall(format!(
            "Failed to parse arguments for tool '{}': {}",
            function_name, err
        ))
    })
}

/// Extract arguments string from function object
pub(super) fn extract_arguments_str(function: &Value) -> &str {
    function
        .get("arguments")
        .and_then(|value| value.as_str())
        .unwrap_or("")
}
