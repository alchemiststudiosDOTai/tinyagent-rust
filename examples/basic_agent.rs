//! Basic agent example without tools

use tiny_agent_rs::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Load the OpenRouter API key from the environment
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Set OPENROUTER_API_KEY before running the example");

    println!("🤖 Basic Agent Example (No Tools)");
    println!("=================================");

    // Create agent without tools
    let agent = Agent::new(api_key.to_string(), tiny_agent_rs::FunctionFactory::new())
        .with_model("microsoft/wizardlm-2-8x22b")
        .with_timeout(std::time::Duration::from_secs(60));

    // Test basic calculation (without tool calling)
    let prompt = "What is 25 * 4? Just give me the number.";
    println!("\n📝 Prompt: {}", prompt);

    match agent.run(prompt).await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }

    // Test weather question (without tool calling)
    let weather_prompt = "What's the weather like in Tokyo today? Since you can't access real data, give a realistic example.";
    println!("\n📝 Prompt: {}", weather_prompt);

    match agent.run(weather_prompt).await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }

    println!("\n✅ Basic agent example completed!");
    Ok(())
}
