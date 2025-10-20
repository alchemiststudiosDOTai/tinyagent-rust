use crate::{
    core::steps::AgentStep,
    error::AgentError,
    schemas::{
        validation::{
            validate_structured_payload, FinalAnswerArguments, StructuredResponseArguments,
        },
        SchemaHandle,
    },
    types::result::{RunResult, TokenUsage},
};
use serde_json::Value;
use std::time::Duration;
use tracing::debug;

/// Trait to abstract over memory.add_step vs messages.push
pub(super) trait ErrorSink {
    fn report_error(&mut self, tool_call_id: &str, error_message: String);
    fn report_observation(&mut self, tool_call_id: &str, result: String, is_error: bool);
}

/// Handler outcome indicating what the execution loop should do next
#[derive(Debug)]
pub(super) enum HandlerOutcome {
    /// Continue with the next iteration
    Continue,
    /// Return a complete RunResult (for run_with_steps)
    ReturnResult(RunResult),
    /// Return a string answer (for run_with_messages)
    ReturnAnswer(String),
}

/// Context needed for final_answer handler
pub(super) struct FinalAnswerContext<'a> {
    pub tool_call_id: &'a str,
    pub arguments_json: Value,
    pub completion_schema: Option<&'a SchemaHandle>,
    pub has_final_answer: &'a mut bool,
    pub final_answer_value: &'a mut Option<String>,
}

/// Context needed for run_with_steps final_answer handler
pub(super) struct FinalAnswerStepsContext<'a> {
    pub base: FinalAnswerContext<'a>,
    pub steps: &'a [AgentStep],
    pub token_usage: Option<TokenUsage>,
    pub start_duration: Duration,
    pub iteration: usize,
}

/// Handle final_answer tool call for run_with_steps
pub(super) fn handle_final_answer_steps(
    ctx: FinalAnswerStepsContext<'_>,
    sink: &mut dyn ErrorSink,
) -> Result<HandlerOutcome, AgentError> {
    if *ctx.base.has_final_answer {
        sink.report_error(
            ctx.base.tool_call_id,
            AgentError::InvalidFunctionCall(
                "`final_answer` was already provided for this run".to_string(),
            )
            .to_error_payload()
            .to_string(),
        );
        return Ok(HandlerOutcome::Continue);
    }

    let final_args =
        match serde_json::from_value::<FinalAnswerArguments>(ctx.base.arguments_json.clone()) {
            Ok(args) => args,
            Err(err) => {
                sink.report_error(
                    ctx.base.tool_call_id,
                    AgentError::InvalidFunctionCall(format!(
                        "Invalid final_answer arguments: {}",
                        err
                    ))
                    .to_error_payload()
                    .to_string(),
                );
                return Ok(HandlerOutcome::Continue);
            }
        };

    let answer = final_args.answer.trim();
    if answer.is_empty() {
        sink.report_error(
            ctx.base.tool_call_id,
            AgentError::InvalidFunctionCall(
                "final_answer requires a non-empty `answer` field".to_string(),
            )
            .to_error_payload()
            .to_string(),
        );
        return Ok(HandlerOutcome::Continue);
    }

    let structured_opt = final_args.structured.clone();
    if let Some(schema) = ctx.base.completion_schema {
        if let Some(structured_val) = structured_opt.as_ref() {
            if !structured_val.is_object() {
                let err = AgentError::Validation(format!(
                    "`final_answer.structured` must be a JSON object that matches the `{}` schema",
                    schema.schema_name()
                ));
                sink.report_error(ctx.base.tool_call_id, err.to_error_payload().to_string());
                return Ok(HandlerOutcome::Continue);
            }

            if let Err(err) = validate_structured_payload(schema, structured_val) {
                debug!(
                    target: "tinyagent::schema",
                    schema = schema.schema_name(),
                    error = %err,
                    payload = %structured_val
                );
                sink.report_error(ctx.base.tool_call_id, err.to_error_payload().to_string());
                return Ok(HandlerOutcome::Continue);
            }
        }
    }

    let answer_string = answer.to_string();
    *ctx.base.has_final_answer = true;
    *ctx.base.final_answer_value = Some(answer_string.clone());

    // Report to sink (adds FinalAnswer step)
    sink.report_observation(
        ctx.base.tool_call_id,
        format!(
            "{{\"answer\":{}}}",
            serde_json::to_string(&answer_string).unwrap()
        ),
        false,
    );

    if let Some(structured_val) = structured_opt.clone() {
        return Ok(HandlerOutcome::ReturnResult(RunResult::new(
            answer_string,
            Some(structured_val),
            ctx.base.completion_schema.cloned(),
            ctx.steps.to_vec(),
            ctx.token_usage,
            ctx.start_duration,
            ctx.iteration,
        )));
    }

    if ctx.base.completion_schema.is_none() {
        return Ok(HandlerOutcome::ReturnResult(RunResult::new(
            answer_string,
            None,
            None,
            ctx.steps.to_vec(),
            ctx.token_usage,
            ctx.start_duration,
            ctx.iteration,
        )));
    }

    Ok(HandlerOutcome::Continue)
}

/// Handle final_answer tool call for run_with_messages
pub(super) fn handle_final_answer_messages(
    ctx: FinalAnswerContext<'_>,
    sink: &mut dyn ErrorSink,
) -> Result<HandlerOutcome, AgentError> {
    if *ctx.has_final_answer {
        sink.report_error(
            ctx.tool_call_id,
            AgentError::InvalidFunctionCall(
                "`final_answer` was already provided for this run".to_string(),
            )
            .to_error_payload()
            .to_string(),
        );
        return Ok(HandlerOutcome::Continue);
    }

    let final_args = match serde_json::from_value::<FinalAnswerArguments>(ctx.arguments_json) {
        Ok(args) => args,
        Err(err) => {
            sink.report_error(
                ctx.tool_call_id,
                AgentError::InvalidFunctionCall(format!("Invalid final_answer arguments: {}", err))
                    .to_error_payload()
                    .to_string(),
            );
            return Ok(HandlerOutcome::Continue);
        }
    };

    let answer = final_args.answer.trim();
    if answer.is_empty() {
        sink.report_error(
            ctx.tool_call_id,
            AgentError::InvalidFunctionCall(
                "final_answer requires a non-empty `answer` field".to_string(),
            )
            .to_error_payload()
            .to_string(),
        );
        return Ok(HandlerOutcome::Continue);
    }

    let structured_opt = final_args.structured.clone();
    if let Some(schema) = ctx.completion_schema {
        if let Some(structured_val) = structured_opt.as_ref() {
            if !structured_val.is_object() {
                let err = AgentError::Validation(format!(
                    "`final_answer.structured` must be a JSON object that matches the `{}` schema",
                    schema.schema_name()
                ));
                sink.report_error(ctx.tool_call_id, err.to_error_payload().to_string());
                return Ok(HandlerOutcome::Continue);
            }

            if let Err(err) = validate_structured_payload(schema, structured_val) {
                debug!(
                    target: "tinyagent::schema",
                    schema = schema.schema_name(),
                    error = %err,
                    payload = %structured_val
                );
                sink.report_error(ctx.tool_call_id, err.to_error_payload().to_string());
                return Ok(HandlerOutcome::Continue);
            }
        }
    }

    let answer_string = answer.to_string();
    *ctx.has_final_answer = true;
    *ctx.final_answer_value = Some(answer_string.clone());

    if structured_opt.is_some() {
        return Ok(HandlerOutcome::ReturnAnswer(answer_string));
    }

    if ctx.completion_schema.is_none() {
        return Ok(HandlerOutcome::ReturnAnswer(answer_string));
    }

    // Acknowledge and continue (waiting for structured_response)
    sink.report_observation(
        ctx.tool_call_id,
        serde_json::json!({ "status": "acknowledged" }).to_string(),
        false,
    );

    Ok(HandlerOutcome::Continue)
}

/// Context needed for structured_response handler
pub(super) struct StructuredResponseContext<'a> {
    pub tool_call_id: &'a str,
    pub arguments_json: Value,
    pub schema: &'a SchemaHandle,
    pub final_answer_value: Option<String>,
}

/// Context for run_with_steps structured_response handler
pub(super) struct StructuredResponseStepsContext<'a> {
    pub base: StructuredResponseContext<'a>,
    pub steps: &'a [AgentStep],
    pub token_usage: Option<TokenUsage>,
    pub start_duration: Duration,
    pub iteration: usize,
}

/// Handle structured_response tool call for run_with_steps
pub(super) fn handle_structured_response_steps(
    ctx: StructuredResponseStepsContext<'_>,
    sink: &mut dyn ErrorSink,
) -> Result<HandlerOutcome, AgentError> {
    let args = match serde_json::from_value::<StructuredResponseArguments>(ctx.base.arguments_json)
    {
        Ok(val) => val,
        Err(err) => {
            sink.report_error(
                ctx.base.tool_call_id,
                AgentError::InvalidFunctionCall(format!(
                    "Invalid structured_response arguments: {}",
                    err
                ))
                .to_error_payload()
                .to_string(),
            );
            return Ok(HandlerOutcome::Continue);
        }
    };

    if !args.structured.is_object() {
        sink.report_error(
            ctx.base.tool_call_id,
            AgentError::Validation(format!(
                "`structured_response.structured` must be a JSON object that matches the `{}` schema",
                ctx.base.schema.schema_name()
            ))
            .to_error_payload()
            .to_string(),
        );
        return Ok(HandlerOutcome::Continue);
    }

    if let Err(err) = validate_structured_payload(ctx.base.schema, &args.structured) {
        debug!(
            target: "tinyagent::schema",
            schema = ctx.base.schema.schema_name(),
            error = %err,
            payload = %args.structured
        );
        sink.report_error(ctx.base.tool_call_id, err.to_error_payload().to_string());
        return Ok(HandlerOutcome::Continue);
    }

    let answer_string = ctx
        .base
        .final_answer_value
        .unwrap_or_else(|| "Task completed with structured response".to_string());

    Ok(HandlerOutcome::ReturnResult(RunResult::new(
        answer_string,
        Some(args.structured),
        Some(ctx.base.schema.clone()),
        ctx.steps.to_vec(),
        ctx.token_usage,
        ctx.start_duration,
        ctx.iteration,
    )))
}

/// Handle structured_response tool call for run_with_messages
pub(super) fn handle_structured_response_messages(
    ctx: StructuredResponseContext<'_>,
    sink: &mut dyn ErrorSink,
) -> Result<HandlerOutcome, AgentError> {
    let args = match serde_json::from_value::<StructuredResponseArguments>(ctx.arguments_json) {
        Ok(val) => val,
        Err(err) => {
            sink.report_error(
                ctx.tool_call_id,
                AgentError::InvalidFunctionCall(format!(
                    "Invalid structured_response arguments: {}",
                    err
                ))
                .to_error_payload()
                .to_string(),
            );
            return Ok(HandlerOutcome::Continue);
        }
    };

    if !args.structured.is_object() {
        sink.report_error(
            ctx.tool_call_id,
            AgentError::Validation(format!(
                "`structured_response.structured` must be a JSON object that matches the `{}` schema",
                ctx.schema.schema_name()
            ))
            .to_error_payload()
            .to_string(),
        );
        return Ok(HandlerOutcome::Continue);
    }

    if let Err(err) = validate_structured_payload(ctx.schema, &args.structured) {
        debug!(
            target: "tinyagent::schema",
            schema = ctx.schema.schema_name(),
            error = %err,
            payload = %args.structured
        );
        sink.report_error(ctx.tool_call_id, err.to_error_payload().to_string());
        return Ok(HandlerOutcome::Continue);
    }

    let answer_string = ctx
        .final_answer_value
        .unwrap_or_else(|| "Task completed with structured response".to_string());

    sink.report_observation(
        ctx.tool_call_id,
        serde_json::json!({ "status": "accepted" }).to_string(),
        false,
    );

    Ok(HandlerOutcome::ReturnAnswer(answer_string))
}
