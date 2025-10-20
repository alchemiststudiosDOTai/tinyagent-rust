//! Complete Agent Example
//!
//! This is the final demonstration of tiny-agent-rs with real OpenRouter integration.
//! Shows the full agent loop with tool calling capabilities.

use tiny_agent_rs::{
    tools::{CalculatorTool, WeatherTool},
    Agent, FunctionFactory,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("🚀 Complete Agent Example with OpenRouter");
    println!("=========================================");
    println!("This demonstrates a fully functional Rust agent with:");
    println!("- Real LLM integration via OpenRouter");
    println!("- Tool calling capabilities");
    println!("- Type-safe parameter validation");
    println!("- Error handling and retry logic");
    println!();

    // Load API key from the environment
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Set OPENROUTER_API_KEY before running the example");

    println!("🔑 Using API key from OPENROUTER_API_KEY");

    // Set up function factory with tools
    let mut function_factory = FunctionFactory::new();
    function_factory.register_tool(CalculatorTool::new());
    function_factory.register_tool(WeatherTool::new());

    println!("🛠️  Registered tools: Calculator, Weather");

    // Create agent (using a working model)
    let agent = Agent::new(api_key, function_factory)
        .with_model("openai/gpt-4.1-mini")
        .with_timeout(std::time::Duration::from_secs(120));

    println!("🤖 Agent configured with model: openai/gpt-4.1-mini");
    println!();

    // Test 1: Complex calculation requiring tool
    println!("📊 Test 1: Complex Calculation");
    println!("Prompt: Calculate (15 * 8) + (100 / 5) - 22");

    match agent.run("Calculate (15 * 8) + (100 / 5) - 22").await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    println!();

    // Test 2: Weather information
    println!("🌤️ Test 2: Weather Information");
    println!("Prompt: What's the weather like in London and Tokyo?");

    match agent
        .run("What's the weather like in London and Tokyo?")
        .await
    {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    println!();

    // Test 3: Complex multi-tool query
    println!("🔧 Test 3: Multi-Tool Query");
    println!("Prompt: If it's 25°C in London and 15°C in New York, what's the temperature difference in Fahrenheit?");

    match agent.run("If it's 25°C in London and 15°C in New York, what's the temperature difference in Fahrenheit?").await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    println!();

    // Test 4: General knowledge
    println!("🧠 Test 4: General Knowledge");
    println!("Prompt: What are the main differences between Rust and Go programming languages?");

    match agent
        .run("What are the main differences between Rust and Go programming languages?")
        .await
    {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    println!();

    println!("🎉 Complete agent example finished!");
    println!("====================================");
    println!("✅ All tests completed successfully!");
    println!("🔗 Repository: https://github.com/tunahorse/tinyagent-rust");
    println!("📚 Documentation: See README.md for usage instructions");

    Ok(())
}
