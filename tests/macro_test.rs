use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use tiny_agent_rs::tools::Tool;

#[derive(Debug, Deserialize, JsonSchema)]
struct TestParams {
    value: i32,
    #[serde(default)]
    optional_text: Option<String>,
}

tiny_agent_rs::tool!(
    name = "test_tool",
    description = "A test tool that doubles a value",
    params = TestParams,
    |params: TestParams| async move {
        let doubled = params.value * 2;
        Ok(json!({
            "doubled": doubled,
            "text": params.optional_text
        }))
    }
);

#[tokio::test]
async fn test_macro_generated_tool() {
    let tool = TestTool;

    assert_eq!(tool.name(), "test_tool");
    assert_eq!(tool.description(), "A test tool that doubles a value");

    let schema = tool.parameters_schema();
    assert!(schema.is_object());

    let params = json!({
        "value": 21,
        "optional_text": "hello"
    });

    let result = tool.execute(params).await.unwrap();
    assert_eq!(result["doubled"], 42);
    assert_eq!(result["text"], "hello");
}

#[tokio::test]
async fn test_macro_tool_optional_params() {
    let tool = TestTool;

    let params = json!({
        "value": 10
    });

    let result = tool.execute(params).await.unwrap();
    assert_eq!(result["doubled"], 20);
    assert!(result["text"].is_null());
}

#[tokio::test]
async fn test_macro_tool_invalid_params() {
    let tool = TestTool;

    let params = json!({
        "wrong_field": "oops"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}
