use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use tiny_agent_rs::{tools::Tool, Agent, FunctionFactory};

// Define a simple string manipulation tool using the macro
#[derive(Debug, Deserialize, JsonSchema)]
struct TextTransformParams {
    /// The text to transform
    text: String,
    /// Transformation to apply: "uppercase", "lowercase", or "reverse"
    operation: String,
}

tinyagent_macros::tool!(
    name = "text_transform",
    description = "Transform text by applying uppercase, lowercase, or reverse operations",
    params = TextTransformParams,
    |params: TextTransformParams| async move {
        let result = match params.operation.as_str() {
            "uppercase" => params.text.to_uppercase(),
            "lowercase" => params.text.to_lowercase(),
            "reverse" => params.text.chars().rev().collect(),
            _ => return Err(format!("Unknown operation: {}", params.operation)),
        };

        Ok(json!({
            "original": params.text,
            "operation": params.operation,
            "result": result
        }))
    }
);

// Define a math tool using the macro
#[derive(Debug, Deserialize, JsonSchema)]
struct MathParams {
    /// First number
    a: f64,
    /// Second number
    b: f64,
    /// Operation: "add", "subtract", "multiply", or "divide"
    operation: String,
}

tinyagent_macros::tool!(
    name = "math_calculator",
    description = "Perform basic math operations on two numbers",
    params = MathParams,
    |params: MathParams| async move {
        let result = match params.operation.as_str() {
            "add" => params.a + params.b,
            "subtract" => params.a - params.b,
            "multiply" => params.a * params.b,
            "divide" => {
                if params.b == 0.0 {
                    return Err("Cannot divide by zero".to_string());
                }
                params.a / params.b
            }
            _ => return Err(format!("Unknown operation: {}", params.operation)),
        };

        Ok(json!({
            "a": params.a,
            "b": params.b,
            "operation": params.operation,
            "result": result
        }))
    }
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".to_string());

    // Register the macro-generated tools
    let mut factory = FunctionFactory::new();
    factory.register_tool(TextTransform);
    factory.register_tool(MathCalculator);

    let _agent = Agent::new(api_key, factory).with_max_iterations(3);

    println!("=== Macro Tool Example ===\n");
    println!("Available tools:");
    println!("1. text_transform - Transform text (uppercase/lowercase/reverse)");
    println!("2. math_calculator - Basic math operations\n");

    // Demonstrate the tools work correctly
    println!("Testing text_transform tool:");
    let text_tool = TextTransform;
    let result = text_tool
        .execute(json!({
            "text": "Hello World",
            "operation": "uppercase"
        }))
        .await?;
    println!("Result: {}\n", serde_json::to_string_pretty(&result)?);

    println!("Testing math_calculator tool:");
    let math_tool = MathCalculator;
    let result = math_tool
        .execute(json!({
            "a": 10.0,
            "b": 3.0,
            "operation": "multiply"
        }))
        .await?;
    println!("Result: {}\n", serde_json::to_string_pretty(&result)?);

    println!("Schema for text_transform:");
    let schema = text_tool.parameters_schema();
    println!("{}\n", serde_json::to_string_pretty(&schema)?);

    Ok(())
}
