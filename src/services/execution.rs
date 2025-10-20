use super::response_handler::{
    handle_final_answer_messages, handle_final_answer_steps, handle_structured_response_messages,
    handle_structured_response_steps, ErrorSink, FinalAnswerContext, FinalAnswerStepsContext,
    HandlerOutcome, StructuredResponseContext, StructuredResponseStepsContext,
};
use crate::{
    core::{agent::Agent, memory::AgentMemory, steps::AgentStep},
    error::{AgentError, Result},
    schemas::validation::{
        final_answer_tool_definition, inject_schema_instructions,
        structured_response_tool_definition, structured_response_tool_name,
    },
    services::{
        openai_client::ChatCompletionRequest,
        tool_call_utils::{
            extract_arguments_str, extract_function_info, extract_tool_call_id,
            parse_function_arguments,
        },
    },
    types::result::{RunResult, TokenUsage},
};
use serde_json::{json, Value};
use std::time::Instant;
use tokio::time::timeout;

/// ErrorSink implementation for AgentMemory (run_with_steps)
struct MemorySink<'a> {
    memory: &'a mut AgentMemory,
}

impl<'a> ErrorSink for MemorySink<'a> {
    fn report_error(&mut self, tool_call_id: &str, error_message: String) {
        self.memory.add_step(AgentStep::Observation {
            tool_call_id: tool_call_id.to_string(),
            result: error_message,
            is_error: true,
        });
    }

    fn report_observation(&mut self, tool_call_id: &str, result: String, is_error: bool) {
        self.memory.add_step(AgentStep::Observation {
            tool_call_id: tool_call_id.to_string(),
            result,
            is_error,
        });
    }
}

/// ErrorSink implementation for Vec<Value> messages (run_with_messages)
struct MessagesSink<'a> {
    messages: &'a mut Vec<Value>,
}

impl<'a> ErrorSink for MessagesSink<'a> {
    fn report_error(&mut self, tool_call_id: &str, error_message: String) {
        self.messages.push(json!({
            "role": "tool",
            "tool_call_id": tool_call_id,
            "content": error_message
        }));
    }

    fn report_observation(&mut self, tool_call_id: &str, result: String, _is_error: bool) {
        self.messages.push(json!({
            "role": "tool",
            "tool_call_id": tool_call_id,
            "content": result
        }));
    }
}

impl Agent {
    pub async fn run_with_steps(&self, prompt: &str) -> Result<RunResult> {
        let start_time = Instant::now();
        let mut memory = AgentMemory::with_default_system();

        memory.add_step(AgentStep::Task {
            content: prompt.to_string(),
        });

        let mut iteration = 0;
        let mut has_final_answer = false;
        let mut final_answer_value: Option<String> = None;

        while iteration < self.max_iterations() {
            iteration += 1;

            let mut messages = memory.as_messages();
            if let Some(schema) = self.completion_schema() {
                inject_schema_instructions(&mut messages, schema);
            }

            let mut tools = self.function_factory().get_openai_tools();
            if let Some(schema) = self.completion_schema() {
                tools.push(structured_response_tool_definition(schema));
            } else {
                tools.push(final_answer_tool_definition());
            }

            let mut chat_request =
                ChatCompletionRequest::new(self.model().to_owned(), messages.clone())
                    .with_max_tokens(self.max_tokens());

            if !tools.is_empty() {
                chat_request = chat_request
                    .with_tools(tools)
                    .with_tool_choice(json!("auto"));
            }

            let request_body = chat_request.into_value();

            let response = timeout(self.timeout(), self.make_raw_request(&request_body))
                .await
                .map_err(|_| AgentError::Timeout("OpenAI API call timed out".to_string()))??;

            let choices = response
                .get("choices")
                .and_then(|value| value.as_array())
                .ok_or_else(|| {
                    AgentError::Unknown(
                        "Missing 'choices' array in completion response".to_string(),
                    )
                })?;

            let first_choice = choices.first().ok_or_else(|| {
                AgentError::Unknown("Completion response contained no choices".to_string())
            })?;

            let assistant_message = first_choice.get("message").cloned().ok_or_else(|| {
                AgentError::Unknown("Completion response missing assistant message".to_string())
            })?;

            let token_usage = response.get("usage").and_then(|usage| {
                Some(TokenUsage {
                    prompt_tokens: usage.get("prompt_tokens")?.as_u64()? as u32,
                    completion_tokens: usage.get("completion_tokens")?.as_u64()? as u32,
                    total_tokens: usage.get("total_tokens")?.as_u64()? as u32,
                })
            });

            if let Some(tool_calls) = assistant_message.get("tool_calls") {
                if let Some(tool_calls_array) = tool_calls.as_array() {
                    let turn_has_final_answer = tool_calls_array.iter().any(|tool_call| {
                        tool_call
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|name| name.as_str())
                            .map(|name| name == "final_answer")
                            .unwrap_or(false)
                    });

                    if turn_has_final_answer && tool_calls_array.len() > 1 {
                        memory.add_step(AgentStep::Observation {
                            tool_call_id: "final_answer".to_string(),
                            result: AgentError::InvalidFunctionCall(
                                "`final_answer` must be the only tool call in a single turn"
                                    .to_string(),
                            )
                            .to_error_payload()
                            .to_string(),
                            is_error: true,
                        });
                        continue;
                    }

                    for tool_call in tool_calls_array {
                        let tool_call_id = extract_tool_call_id(tool_call);

                        let (function, function_name_opt) = match extract_function_info(tool_call) {
                            Some(info) => info,
                            None => {
                                memory.add_step(AgentStep::Observation {
                                    tool_call_id: tool_call_id.to_string(),
                                    result: "Tool call missing function".to_string(),
                                    is_error: true,
                                });
                                continue;
                            }
                        };

                        let function_name = match function_name_opt {
                            Some(name) if !name.is_empty() => name,
                            _ => {
                                memory.add_step(AgentStep::Observation {
                                    tool_call_id: tool_call_id.to_string(),
                                    result: "Tool call missing function name".to_string(),
                                    is_error: true,
                                });
                                continue;
                            }
                        };

                        let arguments_str = extract_arguments_str(&function);
                        let parsed_arguments =
                            parse_function_arguments(arguments_str, &function_name);

                        match parsed_arguments {
                            Ok(arguments_json) => {
                                if function_name == "final_answer" {
                                    let steps = memory.steps().to_vec();
                                    let mut sink = MemorySink {
                                        memory: &mut memory,
                                    };
                                    let ctx = FinalAnswerStepsContext {
                                        base: FinalAnswerContext {
                                            tool_call_id,
                                            arguments_json,
                                            completion_schema: self.completion_schema(),
                                            has_final_answer: &mut has_final_answer,
                                            final_answer_value: &mut final_answer_value,
                                        },
                                        steps: &steps,
                                        token_usage: token_usage.clone(),
                                        start_duration: start_time.elapsed(),
                                        iteration,
                                    };

                                    match handle_final_answer_steps(ctx, &mut sink)? {
                                        HandlerOutcome::Continue => continue,
                                        HandlerOutcome::ReturnResult(result) => return Ok(result),
                                        HandlerOutcome::ReturnAnswer(_) => unreachable!(),
                                    }
                                }

                                if function_name == structured_response_tool_name() {
                                    let schema = match self.completion_schema() {
                                        Some(schema) => schema.clone(),
                                        None => {
                                            let payload = AgentError::InvalidFunctionCall(
                                                "No completion schema is active for structured response".to_string(),
                                            )
                                            .to_error_payload();
                                            memory.add_step(AgentStep::Observation {
                                                tool_call_id: tool_call_id.to_string(),
                                                result: payload.to_string(),
                                                is_error: true,
                                            });
                                            continue;
                                        }
                                    };

                                    let steps = memory.steps().to_vec();
                                    let mut sink = MemorySink {
                                        memory: &mut memory,
                                    };
                                    let ctx = StructuredResponseStepsContext {
                                        base: StructuredResponseContext {
                                            tool_call_id,
                                            arguments_json,
                                            schema: &schema,
                                            final_answer_value: final_answer_value.clone(),
                                        },
                                        steps: &steps,
                                        token_usage: token_usage.clone(),
                                        start_duration: start_time.elapsed(),
                                        iteration,
                                    };

                                    match handle_structured_response_steps(ctx, &mut sink)? {
                                        HandlerOutcome::Continue => continue,
                                        HandlerOutcome::ReturnResult(result) => return Ok(result),
                                        HandlerOutcome::ReturnAnswer(_) => unreachable!(),
                                    }
                                }

                                // Regular tool execution
                                memory.add_step(AgentStep::Action {
                                    tool_name: function_name.to_string(),
                                    tool_call_id: tool_call_id.to_string(),
                                    arguments: arguments_json.clone(),
                                });

                                match self
                                    .function_factory()
                                    .execute_function(&function_name, arguments_json)
                                    .await
                                {
                                    Ok(result) => {
                                        memory.add_step(AgentStep::Observation {
                                            tool_call_id: tool_call_id.to_string(),
                                            result: result.to_string(),
                                            is_error: false,
                                        });
                                    }
                                    Err(e) => {
                                        let error_payload = e.to_error_payload();
                                        memory.add_step(AgentStep::Observation {
                                            tool_call_id: tool_call_id.to_string(),
                                            result: error_payload.to_string(),
                                            is_error: true,
                                        });
                                    }
                                };
                            }
                            Err(error) => {
                                memory.add_step(AgentStep::Observation {
                                    tool_call_id: tool_call_id.to_string(),
                                    result: error.to_error_payload().to_string(),
                                    is_error: true,
                                });
                            }
                        }
                    }
                }
            } else {
                let answer = assistant_message
                    .get("content")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let message = if !has_final_answer {
                    if answer.is_empty() {
                        "Assistant must call the `final_answer` tool to conclude the task, but returned no content.".to_string()
                    } else {
                        format!(
                            "Assistant must call the `final_answer` tool to conclude the task. Received plain response: {}",
                            answer
                        )
                    }
                } else if answer.is_empty() {
                    format!(
                        "Assistant must call the `{}` tool with the structured schema payload, but returned no content.",
                        structured_response_tool_name()
                    )
                } else {
                    format!(
                        "Assistant must call the `{}` tool with the structured schema payload instead of responding directly: {}",
                        structured_response_tool_name(),
                        answer
                    )
                };

                let tool_name = if self.completion_schema().is_some() {
                    structured_response_tool_name()
                } else {
                    "final_answer"
                };

                memory.add_step(AgentStep::Observation {
                    tool_call_id: tool_name.to_string(),
                    result: message,
                    is_error: true,
                });

                continue;
            }
        }

        Err(AgentError::MaxIterations(self.max_iterations()))
    }

    pub async fn run_with_messages(&self, mut messages: Vec<Value>) -> Result<String> {
        let mut iteration = 0;
        let mut has_final_answer = false;
        let mut final_answer_value: Option<String> = None;

        while iteration < self.max_iterations() {
            iteration += 1;

            if let Some(schema) = self.completion_schema() {
                inject_schema_instructions(&mut messages, schema);
            }

            let mut tools = self.function_factory().get_openai_tools();
            if let Some(schema) = self.completion_schema() {
                tools.push(structured_response_tool_definition(schema));
            } else {
                tools.push(final_answer_tool_definition());
            }

            let mut chat_request =
                ChatCompletionRequest::new(self.model().to_owned(), messages.clone())
                    .with_max_tokens(self.max_tokens());

            if !tools.is_empty() {
                chat_request = chat_request
                    .with_tools(tools)
                    .with_tool_choice(json!("auto"));
            }

            let request_body = chat_request.into_value();

            let response = timeout(self.timeout(), self.make_raw_request(&request_body))
                .await
                .map_err(|_| AgentError::Timeout("OpenAI API call timed out".to_string()))??;

            let choices = response
                .get("choices")
                .and_then(|value| value.as_array())
                .ok_or_else(|| {
                    AgentError::Unknown(
                        "Missing 'choices' array in completion response".to_string(),
                    )
                })?;

            let first_choice = choices.first().ok_or_else(|| {
                AgentError::Unknown("Completion response contained no choices".to_string())
            })?;

            let assistant_message = first_choice.get("message").cloned().ok_or_else(|| {
                AgentError::Unknown("Completion response missing assistant message".to_string())
            })?;

            if let Some(tool_calls) = assistant_message.get("tool_calls") {
                if let Some(tool_calls_array) = tool_calls.as_array() {
                    messages.push(json!({
                        "role": "assistant",
                        "content": assistant_message.get("content").unwrap_or(&json!("")),
                        "tool_calls": tool_calls
                    }));

                    let turn_has_final_answer = tool_calls_array.iter().any(|tool_call| {
                        tool_call
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|name| name.as_str())
                            .map(|name| name == "final_answer")
                            .unwrap_or(false)
                    });

                    if turn_has_final_answer && tool_calls_array.len() > 1 {
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": "final_answer",
                            "content": AgentError::InvalidFunctionCall(
                                "`final_answer` must be the only tool call in a single turn".to_string()
                            )
                            .to_error_payload()
                            .to_string()
                        }));
                        continue;
                    }

                    for tool_call in tool_calls_array {
                        let tool_call_id = extract_tool_call_id(tool_call);

                        let (function, function_name_opt) = match extract_function_info(tool_call) {
                            Some(info) => info,
                            None => {
                                messages.push(json!({
                                    "role": "tool",
                                    "tool_call_id": tool_call_id,
                                    "content": AgentError::InvalidFunctionCall(
                                        "Tool call missing function".to_string()
                                    )
                                    .to_error_payload()
                                    .to_string()
                                }));
                                continue;
                            }
                        };

                        let function_name = match function_name_opt {
                            Some(name) if !name.is_empty() => name,
                            _ => {
                                messages.push(json!({
                                    "role": "tool",
                                    "tool_call_id": tool_call_id,
                                    "content": AgentError::InvalidFunctionCall(
                                        "Tool call missing function name".to_string()
                                    )
                                    .to_error_payload()
                                    .to_string()
                                }));
                                continue;
                            }
                        };

                        let arguments_str = extract_arguments_str(&function);
                        let parsed_arguments =
                            parse_function_arguments(arguments_str, &function_name);

                        if function_name == "final_answer" {
                            let arguments_json = match parsed_arguments {
                                Ok(val) => val,
                                Err(err) => {
                                    messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": err.to_error_payload().to_string()
                                    }));
                                    continue;
                                }
                            };

                            let mut sink = MessagesSink {
                                messages: &mut messages,
                            };
                            let ctx = FinalAnswerContext {
                                tool_call_id,
                                arguments_json,
                                completion_schema: self.completion_schema(),
                                has_final_answer: &mut has_final_answer,
                                final_answer_value: &mut final_answer_value,
                            };

                            match handle_final_answer_messages(ctx, &mut sink)? {
                                HandlerOutcome::Continue => continue,
                                HandlerOutcome::ReturnAnswer(answer) => return Ok(answer),
                                HandlerOutcome::ReturnResult(_) => unreachable!(),
                            }
                        }

                        if function_name == structured_response_tool_name() {
                            let schema = match self.completion_schema() {
                                Some(schema) => schema.clone(),
                                None => {
                                    messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": AgentError::InvalidFunctionCall(
                                            "No completion schema is active for structured response".to_string()
                                        )
                                        .to_error_payload()
                                        .to_string()
                                    }));
                                    continue;
                                }
                            };

                            let arguments_json = match parsed_arguments {
                                Ok(val) => val,
                                Err(err) => {
                                    messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_call_id,
                                        "content": err.to_error_payload().to_string()
                                    }));
                                    continue;
                                }
                            };

                            let mut sink = MessagesSink {
                                messages: &mut messages,
                            };
                            let ctx = StructuredResponseContext {
                                tool_call_id,
                                arguments_json,
                                schema: &schema,
                                final_answer_value: final_answer_value.clone(),
                            };

                            match handle_structured_response_messages(ctx, &mut sink)? {
                                HandlerOutcome::Continue => continue,
                                HandlerOutcome::ReturnAnswer(answer) => return Ok(answer),
                                HandlerOutcome::ReturnResult(_) => unreachable!(),
                            }
                        }

                        // Regular tool execution
                        let result = match parsed_arguments {
                            Ok(arguments_json) => match self
                                .function_factory()
                                .execute_function(&function_name, arguments_json)
                                .await
                            {
                                Ok(result) => result,
                                Err(e) => e.to_error_payload(),
                            },
                            Err(error) => error.to_error_payload(),
                        };

                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_call_id,
                            "content": result.to_string()
                        }));
                    }
                }
            } else {
                let answer = assistant_message
                    .get("content")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let content = if self.completion_schema().is_some() {
                    if answer.is_empty() {
                        format!(
                            "Reminder: Call the `{}` tool with the structured schema payload to complete the task.",
                            structured_response_tool_name()
                        )
                    } else {
                        format!(
                            "Reminder: Provide the structured schema by calling the `{}` tool instead of responding directly: {}",
                            structured_response_tool_name(),
                            answer
                        )
                    }
                } else if answer.is_empty() {
                    "Reminder: You must call the `final_answer` tool with the completed answer to finish.".to_string()
                } else {
                    format!(
                        "Reminder: Do not respond directly. Call the `final_answer` tool with the final answer instead of: {}",
                        answer
                    )
                };

                messages.push(json!({
                    "role": "system",
                    "content": content
                }));

                continue;
            }
        }

        Err(AgentError::MaxIterations(self.max_iterations()))
    }
}
