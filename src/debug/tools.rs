//! Debug tool schema generation
//!
//! This example shows what tool schemas are being generated and sent to the API.

use serde_json::json;
use tiny_agent_rs::{
    tools::{CalculatorTool, WeatherTool},
    Agent, FunctionFactory,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("Debug Tool Schema Generation");
    println!("================================");

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| std::io::Error::other("OPENAI_API_KEY environment variable not set"))?;

    // Set up function factory with tools
    let mut function_factory = FunctionFactory::new();
    function_factory.register_tool(CalculatorTool::new());
    function_factory.register_tool(WeatherTool::new());

    let tools = function_factory.get_openai_tools();

    println!("Tools being sent to API:");
    for (i, tool) in tools.iter().enumerate() {
        println!("\n--- Tool {} ---", i + 1);
        println!("{}", serde_json::to_string_pretty(tool)?);
    }

    // Create the request
    let request_body = json!({
        "model": "openai/gpt-4.1-mini",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant with access to tools. Use tools when necessary to provide accurate information. Be concise and helpful."
            },
            {
                "role": "user",
                "content": "Calculate 15 * 8"
            }
        ],
        "max_tokens": 1000,
        "tools": tools,
        "tool_choice": "auto"
    });

    println!("\nFull request body:");
    println!("{}", serde_json::to_string_pretty(&request_body)?);

    // Make the request
    println!("\nMaking API request...");
    let client = reqwest::Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header(
            "HTTP-Referer",
            "https://github.com/tunahorse/tinyagent-rust",
        )
        .header("X-Title", "tiny-agent-rs")
        .json(&request_body)
        .send()
        .await?;

    println!("Response status: {}", response.status());
    let response_text = response.text().await?;
    println!("Response body:\n{}", response_text);

    Ok(())
}
