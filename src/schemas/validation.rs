use crate::{error::AgentError, schemas::SchemaHandle};
use jsonschema::{Draft, JSONSchema};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_SCHEMA_ERRORS: usize = 3;
const STRUCTURED_RESPONSE_TOOL_NAME: &str = "structured_response";

/// Arguments for the final_answer tool
#[derive(Deserialize)]
pub(crate) struct FinalAnswerArguments {
    pub answer: String,
    #[serde(default)]
    pub structured: Option<Value>,
    #[serde(default)]
    pub _meta: Option<Value>,
}

/// Arguments for the structured_response tool
#[derive(Deserialize)]
pub(crate) struct StructuredResponseArguments {
    pub structured: Value,
    #[serde(default)]
    pub _meta: Option<Value>,
}

/// Validate a structured payload against a schema
pub(crate) fn validate_structured_payload(
    schema: &SchemaHandle,
    payload: &Value,
) -> std::result::Result<(), AgentError> {
    let validator = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(schema.schema_json())
        .map_err(|err| {
            AgentError::Validation(format!(
                "Failed to prepare `{}` schema for validation: {}",
                schema.schema_name(),
                err
            ))
        })?;

    if let Err(errors) = validator.validate(payload) {
        let mut details = Vec::new();
        let mut truncated = false;

        for (idx, error) in errors.enumerate() {
            if idx < MAX_SCHEMA_ERRORS {
                let mut path = error.instance_path.to_string();
                if path.is_empty() {
                    path = "<root>".to_string();
                }
                details.push(format!("{}: {}", path, error));
            } else {
                truncated = true;
                break;
            }
        }

        let mut detail_str = if details.is_empty() {
            "structured payload failed schema validation".to_string()
        } else {
            details.join("; ")
        };

        if truncated {
            detail_str.push_str("; additional errors truncated");
        }

        return Err(AgentError::Validation(format!(
            "Structured payload does not match `{}` schema: {}",
            schema.schema_name(),
            detail_str
        )));
    }

    Ok(())
}

/// Generate the final_answer tool definition
pub(crate) fn final_answer_tool_definition() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "final_answer",
            "description": "Signal that the agent has completed the task by providing the final answer.",
            "parameters": {
                "type": "object",
                "properties": {
                    "answer": {
                        "type": "string",
                        "description": "Final response for the user"
                    },
                    "structured": {
                        "type": "object",
                        "description": "Structured payload matching the active completion schema",
                        "additionalProperties": true
                    },
                    "meta": {
                        "type": "object",
                        "description": "Optional metadata about the answer",
                        "additionalProperties": true
                    }
                },
                "required": ["answer"]
            }
        }
    })
}

/// Generate the structured_response tool definition with the actual schema
pub(crate) fn structured_response_tool_definition(schema: &SchemaHandle) -> Value {
    let mut properties = serde_json::Map::new();

    // Instead of a generic object, use the actual schema
    let mut structured_param = serde_json::Map::new();
    structured_param.insert("type".to_string(), json!("object"));
    structured_param.insert(
        "description".to_string(),
        json!(format!(
            "The {} data structure. This must match the schema exactly.",
            schema.schema_name()
        )),
    );

    // Inject the actual schema properties
    if let Some(schema_props) = schema.schema_json().get("properties") {
        structured_param.insert("properties".to_string(), schema_props.clone());
    }
    if let Some(schema_required) = schema.schema_json().get("required") {
        structured_param.insert("required".to_string(), schema_required.clone());
    }
    structured_param.insert("additionalProperties".to_string(), json!(false));

    properties.insert("structured".to_string(), json!(structured_param));

    json!({
        "type": "function",
        "function": {
            "name": STRUCTURED_RESPONSE_TOOL_NAME,
            "description": format!(
                "Complete the task by providing a {} object with all required fields.",
                schema.schema_name()
            ),
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": ["structured"],
                "additionalProperties": false
            }
        }
    })
}

/// Inject schema instructions into the first system message
pub(crate) fn inject_schema_instructions(messages: &mut [Value], schema: &SchemaHandle) {
    let Some(first_message) = messages.first_mut() else {
        return;
    };

    if first_message.get("role").and_then(|value| value.as_str()) != Some("system") {
        return;
    }

    if let Some(obj) = first_message.as_object_mut() {
        if let Some(content_value) = obj.get_mut("content") {
            if let Some(content_str) = content_value.as_str() {
                if content_str.contains("Structured response requirement:") {
                    return;
                }

                let mut updated = content_str.to_string();
                updated.push_str(&format!(
                    "\n\nStructured response requirement: when you finish the task, you MUST call the `{}` tool with a JSON payload that strictly conforms to the `{}` schema. This is the ONLY way to complete the task.",
                    STRUCTURED_RESPONSE_TOOL_NAME,
                    schema.schema_name()
                ));

                *content_value = Value::String(updated);
            }
        }
    }
}

pub(crate) fn structured_response_tool_name() -> &'static str {
    STRUCTURED_RESPONSE_TOOL_NAME
}
