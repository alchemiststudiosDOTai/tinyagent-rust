# Tool Macro Guide

## Overview

The `tool!` macro simplifies tool creation by eliminating boilerplate and automatically generating JSON schemas from parameter structs. This reduces tool implementation effort by approximately 80%.

## Quick Start

### 1. Define Your Parameters

Use `schemars::JsonSchema` and `serde::Deserialize` derives on your parameter struct:

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
struct MyToolParams {
    /// Description for this field (shown in schema)
    required_field: String,
    
    /// An optional parameter
    #[serde(default)]
    optional_field: Option<i32>,
}
```

### 2. Create Your Tool

Use the `tool!` macro with a closure that contains your tool logic:

```rust
tinyagent_macros::tool!(
    name = "my_tool",
    description = "Brief description of what the tool does",
    params = MyToolParams,
    |params: MyToolParams| async move {
        // Your tool logic here
        let result = do_something(params.required_field);
        
        // Return Ok with JSON value
        Ok(serde_json::json!({
            "result": result
        }))
    }
);
```

### 3. Register and Use

The macro generates a struct named using PascalCase (e.g., `MyTool` for `my_tool`):

```rust
let mut factory = FunctionFactory::new();
factory.register_tool(MyTool);

let agent = Agent::new(api_key, factory);
```

## Macro Syntax

```rust
tool!(
    name = "tool_name",           // Snake case name for the tool
    description = "Tool purpose",  // Description for LLM
    params = ParamStructType,      // Your parameter struct type
    |params: ParamStructType| async move {
        // Async closure with tool logic
        // Must return Result<serde_json::Value, String>
    }
);
```

## What the Macro Generates

The macro automatically generates:

1. **Tool struct** - Named in PascalCase (e.g., `tool_name` → `ToolName`)
2. **Tool trait implementation** - All required methods
3. **JSON schema** - Derived from your `JsonSchema` implementation
4. **Parameter parsing** - With proper error messages
5. **Error handling** - Converts your `String` errors to `AgentError::ToolExecution`

## Parameter Structs

### Required Fields

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct RequiredParams {
    field1: String,
    field2: i32,
}
```

### Optional Fields

Use `#[serde(default)]` for optional fields:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct OptionalParams {
    required: String,
    
    #[serde(default)]
    optional: Option<String>,
}
```

### Nested Structures

Complex nested types are supported:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct Address {
    street: String,
    city: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PersonParams {
    name: String,
    
    #[serde(default)]
    address: Option<Address>,
}
```

### Field Descriptions

Use doc comments to add descriptions (appear in generated schema):

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct DescribedParams {
    /// The user's full name
    name: String,
    
    /// Age in years (must be positive)
    age: u32,
}
```

## Complete Examples

### Text Transformation Tool

```rust
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize, JsonSchema)]
struct TextTransformParams {
    /// The text to transform
    text: String,
    
    /// Operation: "uppercase", "lowercase", or "reverse"
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

// Use it:
// factory.register_tool(TextTransform);
```

### Calculator Tool

```rust
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

// Use it:
// factory.register_tool(MathCalculator);
```

### API Integration Tool

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct ApiParams {
    /// The API endpoint to call
    endpoint: String,
    
    /// Optional query parameters
    #[serde(default)]
    params: Option<std::collections::HashMap<String, String>>,
}

tinyagent_macros::tool!(
    name = "api_call",
    description = "Make HTTP GET requests to external APIs",
    params = ApiParams,
    |params: ApiParams| async move {
        let client = reqwest::Client::new();
        let mut request = client.get(&params.endpoint);
        
        if let Some(query_params) = params.params {
            request = request.query(&query_params);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
            
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        
        Ok(json!({
            "endpoint": params.endpoint,
            "response": body
        }))
    }
);
```

## Error Handling

Return errors as `String`:

```rust
tinyagent_macros::tool!(
    name = "divide",
    description = "Divide two numbers",
    params = DivideParams,
    |params: DivideParams| async move {
        if params.divisor == 0.0 {
            return Err("Division by zero".to_string());
        }
        
        Ok(json!({ "result": params.dividend / params.divisor }))
    }
);
```

The macro automatically converts your `String` errors to `AgentError::ToolExecution`.

## Schema Generation

The generated schema is OpenAI-compatible and includes:

- **Field types** - Automatically inferred from Rust types
- **Required fields** - Non-optional fields marked as required
- **Descriptions** - From doc comments
- **Nested objects** - Complex types properly structured

Example generated schema:

```json
{
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "description": "The text to transform"
    },
    "operation": {
      "type": "string",
      "description": "Operation: \"uppercase\", \"lowercase\", or \"reverse\""
    }
  },
  "required": ["text", "operation"],
  "title": "TextTransformParams"
}
```

## Best Practices

### 1. Clear Descriptions

Always document your parameters and tool purpose:

```rust
/// The user's email address (must be valid format)
email: String,
```

### 2. Validate Input

Validate parameters in your tool logic:

```rust
|params: EmailParams| async move {
    if !params.email.contains('@') {
        return Err("Invalid email format".to_string());
    }
    // ...
}
```

### 3. Structured Output

Return structured JSON for better parsing:

```rust
Ok(json!({
    "status": "success",
    "data": result,
    "timestamp": chrono::Utc::now().to_rfc3339()
}))
```

### 4. Use Enums for Choices

For limited options, consider using enums:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MathParams {
    a: f64,
    b: f64,
    operation: Operation,
}
```

## Comparison: Before vs After

### Before (Manual Implementation)

```rust
#[derive(Debug)]
struct MyTool;

#[derive(Debug, Deserialize)]
struct MyToolParams {
    value: i32,
}

impl Tool for MyTool {
    fn name(&self) -> &'static str {
        "my_tool"
    }

    fn description(&self) -> &'static str {
        "Does something useful"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": "integer",
                    "description": "A value"
                }
            },
            "required": ["value"],
            "additionalProperties": false
        })
    }

    fn execute(
        &self,
        parameters: serde_json::Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, AgentError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let params: MyToolParams = serde_json::from_value(parameters)
                .map_err(|e| AgentError::ToolExecution(format!("Invalid params: {}", e)))?;
            
            Ok(json!({ "result": params.value * 2 }))
        })
    }
}

// ~70 lines of code
```

### After (With Macro)

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct MyToolParams {
    /// A value
    value: i32,
}

tinyagent_macros::tool!(
    name = "my_tool",
    description = "Does something useful",
    params = MyToolParams,
    |params: MyToolParams| async move {
        Ok(json!({ "result": params.value * 2 }))
    }
);

// ~15 lines of code - 80% reduction!
```

## Testing Your Tools

### Unit Tests

Test tools directly by calling execute:

```rust
#[tokio::test]
async fn test_my_tool() {
    let tool = MyTool;
    
    let result = tool.execute(json!({
        "value": 21
    })).await.unwrap();
    
    assert_eq!(result["result"], 42);
}
```

### Schema Validation

Verify schema generation:

```rust
#[test]
fn test_schema() {
    let tool = MyTool;
    let schema = tool.parameters_schema();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].get("value").is_some());
}
```

## Troubleshooting

### Struct Name Not Found

The macro generates a struct in PascalCase. For `my_tool`, use `MyTool`:

```rust
factory.register_tool(MyTool);  // ✅ Correct
factory.register_tool(my_tool);  // ❌ Wrong
```

### Type Annotations Needed

Always annotate the closure parameter type:

```rust
|params: MyToolParams| async move { ... }  // ✅ Correct
|params| async move { ... }                 // ❌ Wrong - type needed
```

### Schema Not Generated

Ensure `JsonSchema` derive is present:

```rust
#[derive(Debug, Deserialize, JsonSchema)]  // ✅ Correct
#[derive(Debug, Deserialize)]              // ❌ Missing JsonSchema
struct MyParams { ... }
```

## See Also

- [Example: macro_tool_example.rs](../examples/macro_tool_example.rs)
- [Tests: tests/macro_test.rs](../tests/macro_test.rs)
- [Schema validation: tests/schema_validation_test.rs](../tests/schema_validation_test.rs)
- [Macro implementation: tinyagent-macros/src/lib.rs](../tinyagent-macros/src/lib.rs)
