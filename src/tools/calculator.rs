use super::Tool;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Parameters for calculator operations
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CalculatorParams {
    pub operation: Operation,
    pub a: f64,
    pub b: f64,
}

/// Supported calculator operations
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

/// A calculator tool for basic arithmetic operations
#[derive(Debug)]
pub struct CalculatorTool;

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for CalculatorTool {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn description(&self) -> &'static str {
        "Perform basic arithmetic operations (add, subtract, multiply, divide, power)"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide", "power"]
                },
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        })
    }

    fn execute(
        &self,
        parameters: serde_json::Value,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, crate::AgentError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let params: CalculatorParams = serde_json::from_value(parameters).map_err(|e| {
                crate::AgentError::ToolExecution(format!("Invalid parameters: {}", e))
            })?;

            let result = match params.operation {
                Operation::Add => params.a + params.b,
                Operation::Subtract => params.a - params.b,
                Operation::Multiply => params.a * params.b,
                Operation::Divide => {
                    if params.b == 0.0 {
                        return Err(crate::AgentError::ToolExecution(
                            "Division by zero is not allowed".to_string(),
                        ));
                    }
                    params.a / params.b
                }
                Operation::Power => params.a.powf(params.b),
            };

            Ok(serde_json::json!({
                "result": result,
                "operation": format!("{:?} {} {}", params.operation, params.a, params.b)
            }))
        })
    }
}
