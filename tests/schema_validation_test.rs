use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use tiny_agent_rs::tools::Tool;

#[derive(Debug, Deserialize, JsonSchema)]
struct ComplexParams {
    /// A required string field
    name: String,
    /// A required integer
    age: u32,
    /// An optional email address
    #[serde(default)]
    email: Option<String>,
    /// A nested optional object
    #[serde(default)]
    address: Option<Address>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct Address {
    street: String,
    city: String,
    #[serde(default)]
    zip_code: Option<String>,
}

tiny_agent_rs::tool!(
    name = "complex_tool",
    description = "A tool with complex nested parameters",
    params = ComplexParams,
    |params: ComplexParams| async move {
        Ok(json!({
            "name": params.name,
            "age": params.age,
            "email": params.email,
            "has_address": params.address.is_some()
        }))
    }
);

#[test]
fn test_schema_has_correct_structure() {
    let tool = ComplexTool;
    let schema = tool.parameters_schema();

    // Should be an object type
    assert_eq!(schema["type"], "object");

    // Should have properties
    assert!(schema["properties"].is_object());
    let props = schema["properties"].as_object().unwrap();

    // Check required fields exist
    assert!(props.contains_key("name"));
    assert!(props.contains_key("age"));
    assert!(props.contains_key("email"));
    assert!(props.contains_key("address"));

    // Check types
    assert_eq!(props["name"]["type"], "string");
    assert_eq!(props["age"]["type"], "integer");
}

#[test]
fn test_schema_describes_optional_fields() {
    let tool = ComplexTool;
    let schema = tool.parameters_schema();

    let props = schema["properties"].as_object().unwrap();

    // Optional fields should allow null or have oneOf/anyOf
    // schemars represents Option<T> as either oneOf or with nullable
    assert!(
        props["email"].get("oneOf").is_some()
            || props["email"].get("anyOf").is_some()
            || props["email"].get("type").is_some()
    );
}

#[test]
fn test_schema_handles_nested_objects() {
    let tool = ComplexTool;
    let schema = tool.parameters_schema();

    let props = schema["properties"].as_object().unwrap();

    // Address field should be present
    assert!(props.contains_key("address"));

    // Should reference or contain object schema
    let address_schema = &props["address"];
    assert!(
        address_schema.get("$ref").is_some()
            || address_schema.get("oneOf").is_some()
            || address_schema.get("anyOf").is_some()
            || address_schema.get("properties").is_some()
    );
}

#[tokio::test]
async fn test_complex_tool_execution() {
    let tool = ComplexTool;

    let params = json!({
        "name": "Alice",
        "age": 30,
        "email": "alice@example.com",
        "address": {
            "street": "123 Main St",
            "city": "Springfield"
        }
    });

    let result = tool.execute(params).await.unwrap();
    assert_eq!(result["name"], "Alice");
    assert_eq!(result["age"], 30);
    assert_eq!(result["has_address"], true);
}
