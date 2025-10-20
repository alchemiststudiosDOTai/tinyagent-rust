use serde_json::json;
use tiny_agent_rs::{
    tools::{CalculatorTool, WeatherTool},
    Agent, AgentStep, FunctionFactory, Tool,
};

#[tokio::test]
async fn test_calculator_tool() {
    let calculator = CalculatorTool::new();

    // Test addition
    let params = json!({
        "operation": "add",
        "a": 5.0,
        "b": 3.0
    });

    let result = calculator.execute(params).await.unwrap();
    assert_eq!(result["result"], 8.0);

    // Test division by zero
    let params = json!({
        "operation": "divide",
        "a": 5.0,
        "b": 0.0
    });

    let result = calculator.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_weather_tool() {
    let weather = WeatherTool::new();

    let params = json!({
        "location": "New York",
        "units": "celsius"
    });

    let result = weather.execute(params).await.unwrap();
    assert_eq!(result["location"], "New York");
    assert_eq!(result["units"], "°C");
}

#[tokio::test]
async fn test_function_factory() {
    let mut factory = FunctionFactory::new();
    factory.register_tool(CalculatorTool::new());
    factory.register_tool(WeatherTool::new());

    // Test tool registration
    assert!(factory.has_function("calculator"));
    assert!(factory.has_function("weather"));
    assert!(!factory.has_function("nonexistent"));

    // Test function execution
    let params = json!({
        "operation": "multiply",
        "a": 4.0,
        "b": 5.0
    });

    let result = factory
        .execute_function("calculator", params)
        .await
        .unwrap();
    assert_eq!(result["result"], 20.0);
}

#[test]
fn test_tool_schemas() {
    let calculator = CalculatorTool::new();
    let weather = WeatherTool::new();

    // Test that schemas are valid JSON
    let calc_schema = calculator.parameters_schema();
    assert!(calc_schema.is_object());
    assert!(calc_schema.get("properties").is_some());

    let weather_schema = weather.parameters_schema();
    assert!(weather_schema.is_object());
    assert!(weather_schema.get("properties").is_some());
}

#[test]
fn test_error_handling() {
    use tiny_agent_rs::AgentError;

    // Test error creation and formatting
    let error = AgentError::ToolExecution("Test error".to_string());
    assert_eq!(error.error_code(), "TOOL_EXECUTION_ERROR");
    assert!(error.to_string().contains("Test error"));

    // Test error payload
    let payload = error.to_error_payload();
    assert_eq!(payload["error"]["code"], "TOOL_EXECUTION_ERROR");
    assert_eq!(payload["error"]["retryable"], false);
}

#[tokio::test]
async fn test_smolagents_style_execution() {
    // Skip if no API key
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => return,
    };

    let mut factory = FunctionFactory::new();
    factory.register_tool(CalculatorTool::new());

    let agent = Agent::new(api_key, factory).with_max_iterations(5);

    let result = agent
        .run_with_steps("What is 15 multiplied by 7?")
        .await
        .unwrap();

    // Verify basic result structure
    assert!(!result.output.is_empty());
    assert!(result.iterations > 0);
    assert!(result.duration.as_secs() < 30);

    // Verify steps are recorded
    assert!(!result.steps.is_empty());

    // Should have at least: Task, Action, Observation, FinalAnswer
    let has_task = result
        .steps
        .iter()
        .any(|s| matches!(s, AgentStep::Task { .. }));
    let has_action = result
        .steps
        .iter()
        .any(|s| matches!(s, AgentStep::Action { .. }));
    let has_observation = result
        .steps
        .iter()
        .any(|s| matches!(s, AgentStep::Observation { .. }));
    let has_final_answer = result
        .steps
        .iter()
        .any(|s| matches!(s, AgentStep::FinalAnswer { .. }));

    assert!(has_task, "Should have Task step");
    assert!(has_action, "Should have Action step");
    assert!(has_observation, "Should have Observation step");
    assert!(has_final_answer, "Should have FinalAnswer step");

    // Verify replay functionality
    let replay = result.replay();
    assert!(replay.contains("Agent Execution Trace"));
    assert!(replay.contains("Duration"));
    assert!(replay.contains("Iterations"));
    assert!(replay.contains("Final Output"));

    // Verify explain functionality
    let explain = result.explain();
    assert!(explain.contains("Detailed Steps"));

    // Verify the result is successful
    assert!(result.is_success());

    // Should have at least one action and one observation
    assert!(result.action_count() > 0);
    assert!(result.observation_count() > 0);
}
