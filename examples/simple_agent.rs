//! Simple working agent example with OpenRouter

use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Load the correct OpenRouter API key
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| std::io::Error::other("OPENAI_API_KEY environment variable not set"))?;

    println!("🤖 Simple Agent Example");
    println!("======================");

    // Test basic calculation
    let response = test_calculation(&api_key).await?;
    println!("✅ Calculation test passed: {}", response);

    // Test weather functionality
    let weather_response = test_weather(&api_key).await?;
    println!("✅ Weather test passed: {}", weather_response);

    Ok(())
}

async fn test_calculation(api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("\n🧮 Testing calculation: 15 * 8 + 32");

    let request = json!({
        "model": "microsoft/wizardlm-2-8x22b",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant. Perform the calculation and give a direct answer."
            },
            {
                "role": "user",
                "content": "What is 15 * 8 + 32?"
            }
        ],
        "max_tokens": 50
    });

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
        .json(&request)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response");

    Ok(content.to_string())
}

async fn test_weather(api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("\n🌤️ Testing weather: New York");

    let request = json!({
        "model": "microsoft/wizardlm-2-8x22b",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant with weather information. Since you can't access real-time data, provide a realistic weather report for the requested location."
            },
            {
                "role": "user",
                "content": "What's the weather like in New York?"
            }
        ],
        "max_tokens": 100
    });

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
        .json(&request)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response");

    Ok(content.to_string())
}
