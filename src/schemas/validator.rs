use crate::{AgentError, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// Validation strategies for tool parameters
#[derive(Debug, Clone)]
pub enum Validator {
    /// Fast validation using serde
    SerdeFirst,
    /// Strict validation using JSON Schema
    Strict(StrictValidator),
}

impl Validator {
    /// Validate and deserialize parameters into type T
    pub fn validate<T: DeserializeOwned>(&self, params: Value) -> Result<T> {
        match self {
            Validator::SerdeFirst => serde_first_validate(params),
            Validator::Strict(validator) => validator.validate(params),
        }
    }
}

/// Fast serde-first validator
fn serde_first_validate<T: DeserializeOwned>(params: Value) -> Result<T> {
    serde_path_to_error::deserialize(params).map_err(|e| {
        AgentError::Validation(format!(
            "Parameter validation failed at {}: {}",
            e.path(),
            e
        ))
    })
}

/// Strict JSON Schema validator
#[derive(Debug, Clone)]
pub struct StrictValidator {
    schemas: HashMap<String, Value>,
}

impl StrictValidator {
    /// Create a new strict validator
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register a schema for a tool
    pub fn register_schema(&mut self, tool_name: &str, schema: Value) {
        self.schemas.insert(tool_name.to_string(), schema);
    }

    /// Validate parameters against registered schema
    pub fn validate<T: DeserializeOwned>(&self, params: Value) -> Result<T> {
        // For now, fall back to serde validation
        // In a production implementation, you would use jsonschema crate
        serde_first_validate(params)
    }
}

impl Default for StrictValidator {
    fn default() -> Self {
        Self::new()
    }
}
