//! Real Agent Example
//!
//! This example demonstrates how to use tiny-agent-rs with OpenRouter
//! to create a fully functional agent with tool calling capabilities.

use tiny_agent_rs::{
    tools::{CalculatorTool, WeatherTool},
    Agent, FunctionFactory,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Get API key from environment
    let api_key =
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable not set");

    // Set up function factory with tools
    let mut function_factory = FunctionFactory::new();
    function_factory.register_tool(CalculatorTool::new());
    function_factory.register_tool(WeatherTool::new());

    // Create agent with real LLM
    let agent = Agent::new(api_key, function_factory)
        .with_model("microsoft/wizardlm-2-8x22b")
        .with_timeout(std::time::Duration::from_secs(120));

    println!("🤖 Real Agent Example with OpenRouter");
    println!("=====================================\n");

    // Test with a calculation query
    let prompt1 = "What is 15 * 8 + 32?";
    println!("Query: {}", prompt1);

    match agent.run(prompt1).await {
        Ok(response) => {
            println!("Response: {}\n", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Test with a weather query
    let prompt2 = "What's the weather like in Tokyo?";
    println!("Query: {}", prompt2);

    match agent.run(prompt2).await {
        Ok(response) => {
            println!("Response: {}\n", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Test with a complex query requiring both tools
    let prompt3 = "If it's 25°C in London and 15°C in New York, what's the temperature difference and also calculate 25% of 1000?";
    println!("Query: {}", prompt3);

    match agent.run(prompt3).await {
        Ok(response) => {
            println!("Response: {}\n", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("✅ Real agent example completed!");
    Ok(())
}
