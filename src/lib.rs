//! tiny-agent-rs: A lightweight, type-safe Rust agent library for LLM tool calling
//!
//! This library provides a simple, clean interface for building agents that can
//! execute tools based on LLM responses, with strong typing and deterministic error handling.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use tiny_agent_rs::{Agent, FunctionFactory, tools::CalculatorTool};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let api_key = std::env::var("OPENAI_API_KEY")?;
//!     let mut function_factory = FunctionFactory::new();
//!     function_factory.register_tool(CalculatorTool::new());
//!     
//!     let agent = Agent::new(api_key, function_factory);
//!
//!     let response = agent.run("What is 2 + 3?").await?;
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```

extern crate self as tiny_agent_rs;

pub mod core;
pub mod error;
pub mod schemas;
pub(crate) mod services;
pub mod tools;
pub mod types;

pub use core::{
    generate_planning_prompt, generate_tool_planning_prompt, get_tool_names, is_planning_response,
    Agent, AgentMemory, AgentStep, RunResult, TokenUsage, ToolCall, ToolExecution, ToolOutput,
};
pub use error::{AgentError, Result};
pub use schemas::validator::Validator;
pub use schemas::{schema_type_name, CompletionSchema, SchemaHandle};
pub use tinyagent_macros::completion_schema;
pub use tools::{FunctionFactory, Tool};
pub use types::response::{deserialize_structured_response, StructuredPayload};

pub use core as agent;
pub use schemas as schema;
pub use schemas::validator;
pub use types::response;
pub use types::vacation_types;

#[cfg(feature = "cli")]
pub mod cli;
