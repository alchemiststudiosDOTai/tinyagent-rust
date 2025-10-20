use super::steps::AgentStep;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

/// Memory structure that replaces raw `Vec<Value>` messages
/// Maintains the agent's reasoning steps and converts them to OpenAI format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    steps: Vec<AgentStep>,
    system_prompt: Option<String>,
}

impl AgentMemory {
    /// Create a new memory with optional system prompt
    pub fn new(system_prompt: Option<String>) -> Self {
        Self {
            steps: Vec::new(),
            system_prompt,
        }
    }

    /// Create memory with default system prompt
    pub fn with_default_system() -> Self {
        Self::new(Some(
            "You are a helpful assistant with access to tools. Use tools when necessary to provide accurate information. Be concise and helpful. When you are ready to give the final response, you MUST call the `final_answer` tool with an `answer` string instead of replying directly.".to_string()
        ))
    }

    /// Add a step to memory
    pub fn add_step(&mut self, step: AgentStep) {
        let description = step.describe();
        info!(target: "tinyagent::steps", "{}", description);
        self.steps.push(step);
    }

    /// Get all steps
    pub fn steps(&self) -> &[AgentStep] {
        &self.steps
    }

    /// Get the last step
    pub fn last_step(&self) -> Option<&AgentStep> {
        self.steps.last()
    }

    /// Convert memory to OpenAI message format
    pub fn as_messages(&self) -> Vec<Value> {
        let mut messages = Vec::new();

        if let Some(system_prompt) = &self.system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        for step in &self.steps {
            messages.push(step.to_message());
        }

        messages
    }

    /// Clear all steps but keep system prompt
    pub fn clear_steps(&mut self) {
        self.steps.clear();
    }

    /// Get number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if memory is empty (excluding system prompt)
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get all steps of a specific type
    pub fn filter_steps<F>(&self, predicate: F) -> Vec<&AgentStep>
    where
        F: Fn(&AgentStep) -> bool,
    {
        self.steps.iter().filter(|step| predicate(step)).collect()
    }

    /// Count steps of a specific variant
    pub fn count_actions(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s, AgentStep::Action { .. }))
            .count()
    }

    pub fn count_observations(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s, AgentStep::Observation { .. }))
            .count()
    }

    pub fn set_final_answer_structured(&mut self, structured: Value) {
        if let Some(AgentStep::FinalAnswer {
            structured: slot, ..
        }) = self
            .steps
            .iter_mut()
            .rev()
            .find(|step| matches!(step, AgentStep::FinalAnswer { .. }))
        {
            *slot = Some(structured);
        }
    }
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self::with_default_system()
    }
}

impl From<Vec<Value>> for AgentMemory {
    fn from(messages: Vec<Value>) -> Self {
        let mut memory = AgentMemory::new(None);

        for msg in messages {
            if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                match role {
                    "system" => {
                        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                            memory.system_prompt = Some(content.to_string());
                        }
                    }
                    "user" => {
                        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                            memory.add_step(AgentStep::Task {
                                content: content.to_string(),
                            });
                        }
                    }
                    "assistant" => {
                        if let Some(tool_calls) = msg.get("tool_calls") {
                            if let Some(calls_array) = tool_calls.as_array() {
                                for call in calls_array {
                                    if let (Some(id), Some(function)) = (
                                        call.get("id").and_then(|i| i.as_str()),
                                        call.get("function"),
                                    ) {
                                        let name = function
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("unknown");
                                        let args_str = function
                                            .get("arguments")
                                            .and_then(|a| a.as_str())
                                            .unwrap_or("{}");
                                        let args: Value =
                                            serde_json::from_str(args_str).unwrap_or(Value::Null);

                                        memory.add_step(AgentStep::Action {
                                            tool_name: name.to_string(),
                                            tool_call_id: id.to_string(),
                                            arguments: args,
                                        });
                                    }
                                }
                            }
                        } else if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                memory.add_step(AgentStep::FinalAnswer {
                                    answer: content.to_string(),
                                    structured: None,
                                });
                            }
                        }
                    }
                    "tool" => {
                        if let (Some(id), Some(content)) = (
                            msg.get("tool_call_id").and_then(|i| i.as_str()),
                            msg.get("content").and_then(|c| c.as_str()),
                        ) {
                            let is_error = detect_tool_error(content);
                            memory.add_step(AgentStep::Observation {
                                tool_call_id: id.to_string(),
                                result: content.to_string(),
                                is_error,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        memory
    }
}

fn detect_tool_error(content: &str) -> bool {
    match serde_json::from_str::<Value>(content) {
        Ok(Value::Object(map)) => map.get("error").map(|err| !err.is_null()).unwrap_or(false),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = AgentMemory::new(Some("System".to_string()));
        assert_eq!(memory.step_count(), 0);
        assert!(memory.is_empty());
    }

    #[test]
    fn test_add_steps() {
        let mut memory = AgentMemory::default();
        memory.add_step(AgentStep::Task {
            content: "Test task".to_string(),
        });
        assert_eq!(memory.step_count(), 1);
        assert!(!memory.is_empty());
    }

    #[test]
    fn test_as_messages() {
        let mut memory = AgentMemory::with_default_system();
        memory.add_step(AgentStep::Task {
            content: "Hello".to_string(),
        });

        let messages = memory.as_messages();
        assert_eq!(messages.len(), 2); // system + task
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
    }

    #[test]
    fn test_count_actions() {
        let mut memory = AgentMemory::default();
        memory.add_step(AgentStep::Action {
            tool_name: "test".to_string(),
            tool_call_id: "1".to_string(),
            arguments: Value::Null,
        });
        memory.add_step(AgentStep::Action {
            tool_name: "test2".to_string(),
            tool_call_id: "2".to_string(),
            arguments: Value::Null,
        });
        assert_eq!(memory.count_actions(), 2);
    }
}
