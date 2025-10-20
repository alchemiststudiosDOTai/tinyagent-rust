use super::response::deserialize_structured_response;
use crate::{
    core::steps::AgentStep,
    error::{AgentError, Result as AgentResult},
    schemas::{CompletionSchema, SchemaHandle},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Result of an agent execution run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// Final output from the agent
    pub output: String,
    /// Optional structured payload returned alongside the final answer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured: Option<Value>,
    /// Schema metadata associated with the structured payload
    #[serde(skip)]
    pub schema: Option<SchemaHandle>,
    /// All reasoning steps taken during execution
    pub steps: Vec<AgentStep>,
    /// Total tokens used (if available from API)
    pub tokens: Option<TokenUsage>,
    /// Total execution duration
    pub duration: Duration,
    /// Number of iterations used
    pub iterations: usize,
}

/// Token usage information from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl RunResult {
    /// Create a new RunResult
    pub fn new(
        output: String,
        structured: Option<Value>,
        schema: Option<SchemaHandle>,
        steps: Vec<AgentStep>,
        tokens: Option<TokenUsage>,
        duration: Duration,
        iterations: usize,
    ) -> Self {
        Self {
            output,
            structured,
            schema,
            steps,
            tokens,
            duration,
            iterations,
        }
    }

    /// Generate a human-readable replay of the execution
    pub fn replay(&self) -> String {
        let mut lines = Vec::new();

        lines.push("=== Agent Execution Trace ===".to_string());
        lines.push(format!("Duration: {:.2}s", self.duration.as_secs_f64()));
        lines.push(format!("Iterations: {}", self.iterations));

        if let Some(tokens) = &self.tokens {
            lines.push(format!(
                "Tokens: {} prompt + {} completion = {} total",
                tokens.prompt_tokens, tokens.completion_tokens, tokens.total_tokens
            ));
        }

        lines.push(String::new());
        lines.push("--- Steps ---".to_string());

        for (idx, step) in self.steps.iter().enumerate() {
            lines.push(format!("{}. {}", idx + 1, step.describe()));
        }

        lines.push(String::new());
        lines.push("--- Final Output ---".to_string());
        lines.push(self.output.clone());

        if let Some(structured) = &self.structured {
            lines.push(String::new());
            lines.push("--- Structured Output ---".to_string());
            lines.push(structured.to_string());
        }

        lines.join("\n")
    }

    /// Generate a detailed explanation with full step data
    pub fn explain(&self) -> String {
        let mut lines = Vec::new();

        lines.push("=== Agent Execution Explanation ===".to_string());
        lines.push(format!("Duration: {:.2}s", self.duration.as_secs_f64()));
        lines.push(format!("Iterations: {}", self.iterations));

        if let Some(tokens) = &self.tokens {
            lines.push(format!(
                "Tokens: {} prompt + {} completion = {} total",
                tokens.prompt_tokens, tokens.completion_tokens, tokens.total_tokens
            ));
        }

        lines.push(String::new());
        lines.push("--- Detailed Steps ---".to_string());

        for (idx, step) in self.steps.iter().enumerate() {
            lines.push(format!("\n{}. {}", idx + 1, step.describe()));

            match step {
                AgentStep::Task { content } => {
                    lines.push(format!("   Content: {}", content));
                }
                AgentStep::Planning { plan } => {
                    lines.push(format!("   Plan: {}", plan));
                }
                AgentStep::Action {
                    tool_name,
                    tool_call_id,
                    arguments,
                } => {
                    lines.push(format!("   Tool: {}", tool_name));
                    lines.push(format!("   Call ID: {}", tool_call_id));
                    lines.push(format!("   Arguments: {}", arguments));
                }
                AgentStep::Observation {
                    tool_call_id,
                    result,
                    is_error,
                } => {
                    lines.push(format!("   Call ID: {}", tool_call_id));
                    lines.push(format!("   Error: {}", is_error));
                    lines.push(format!("   Result: {}", result));
                }
                AgentStep::FinalAnswer { answer, .. } => {
                    lines.push(format!("   Answer: {}", answer));
                }
            }
        }

        lines.push(String::new());
        lines.push("--- Final Output ---".to_string());
        lines.push(self.output.clone());

        if let Some(structured) = &self.structured {
            lines.push(String::new());
            lines.push("--- Structured Output ---".to_string());
            lines.push(structured.to_string());
        }

        lines.join("\n")
    }

    /// Access the structured payload, if present.
    pub fn structured(&self) -> Option<&Value> {
        self.structured.as_ref()
    }

    /// Access the schema handle associated with the structured payload.
    pub fn schema(&self) -> Option<&SchemaHandle> {
        self.schema.as_ref()
    }

    /// Check whether a structured payload is available.
    pub fn has_structured(&self) -> bool {
        self.structured.is_some()
    }

    /// Deserialize the structured payload into the requested type using the stored schema metadata.
    pub fn deserialize_structured<T>(&self) -> AgentResult<T>
    where
        T: CompletionSchema,
    {
        let payload = self.structured.as_ref().ok_or_else(|| {
            AgentError::Validation("No structured response available on this run".to_string())
        })?;

        let schema = self.schema.as_ref().ok_or_else(|| {
            AgentError::Validation(
                "Missing completion schema metadata for structured response".to_string(),
            )
        })?;

        deserialize_structured_response::<T>(payload, schema)
    }

    /// Get count of actions (tool calls) executed
    pub fn action_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s, AgentStep::Action { .. }))
            .count()
    }

    /// Get count of observations (tool results)
    pub fn observation_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s, AgentStep::Observation { .. }))
            .count()
    }

    /// Check if execution completed successfully (has final answer)
    pub fn is_success(&self) -> bool {
        self.steps
            .iter()
            .any(|s| matches!(s, AgentStep::FinalAnswer { .. }))
    }

    /// Get all error observations
    pub fn errors(&self) -> Vec<&str> {
        self.steps
            .iter()
            .filter_map(|s| match s {
                AgentStep::Observation {
                    result, is_error, ..
                } if *is_error => Some(result.as_str()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion_schema;
    use crate::schema::CompletionSchema;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[test]
    fn test_run_result_creation() {
        let steps = vec![
            AgentStep::Task {
                content: "Test task".to_string(),
            },
            AgentStep::FinalAnswer {
                answer: "Test answer".to_string(),
                structured: None,
            },
        ];

        let result = RunResult::new(
            "Test answer".to_string(),
            None,
            None,
            steps,
            None,
            Duration::from_secs(1),
            1,
        );

        assert_eq!(result.output, "Test answer");
        assert_eq!(result.iterations, 1);
        assert!(result.is_success());
    }

    #[test]
    fn test_action_count() {
        let steps = vec![
            AgentStep::Action {
                tool_name: "tool1".to_string(),
                tool_call_id: "1".to_string(),
                arguments: json!({}),
            },
            AgentStep::Action {
                tool_name: "tool2".to_string(),
                tool_call_id: "2".to_string(),
                arguments: json!({}),
            },
        ];

        let result = RunResult::new(
            "output".to_string(),
            None,
            None,
            steps,
            None,
            Duration::from_secs(1),
            1,
        );

        assert_eq!(result.action_count(), 2);
    }

    #[test]
    fn test_replay_format() {
        let steps = vec![
            AgentStep::Task {
                content: "Test".to_string(),
            },
            AgentStep::FinalAnswer {
                answer: "Done".to_string(),
                structured: None,
            },
        ];

        let result = RunResult::new(
            "Done".to_string(),
            None,
            None,
            steps,
            Some(TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            Duration::from_secs(2),
            1,
        );

        let replay = result.replay();
        assert!(replay.contains("Duration"));
        assert!(replay.contains("Tokens"));
        assert!(replay.contains("Task"));
        assert!(replay.contains("Final Answer"));
    }

    #[test]
    fn test_error_tracking() {
        let steps = vec![
            AgentStep::Observation {
                tool_call_id: "1".to_string(),
                result: "Error occurred".to_string(),
                is_error: true,
            },
            AgentStep::Observation {
                tool_call_id: "2".to_string(),
                result: "Success".to_string(),
                is_error: false,
            },
        ];

        let result = RunResult::new(
            "output".to_string(),
            None,
            None,
            steps,
            None,
            Duration::from_secs(1),
            1,
        );

        let errors = result.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], "Error occurred");
    }

    #[test]
    fn test_deserialize_structured_payload() {
        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
        #[completion_schema]
        struct SamplePlan {
            title: String,
        }

        let structured = json!({ "title": "Sample" });
        let schema = SamplePlan::schema().clone();

        let steps = vec![AgentStep::FinalAnswer {
            answer: "Sample".to_string(),
            structured: Some(structured.clone()),
        }];

        let result = RunResult::new(
            "Sample".to_string(),
            Some(structured),
            Some(schema),
            steps,
            None,
            Duration::from_secs(1),
            1,
        );

        let typed = result.deserialize_structured::<SamplePlan>().unwrap();
        assert_eq!(typed.title, "Sample");
    }
}
