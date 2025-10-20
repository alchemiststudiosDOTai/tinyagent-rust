//! Test OpenRouter Connection
//!
//! This example tests the connection to OpenRouter and verifies the API key works.

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Load environment variables
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable not set");

    println!("🔑 API key detected (length: {})", api_key.len());

    // Create a simple request to test the connection
    let client = reqwest::Client::new();
    let test_request = serde_json::json!({
        "model": "openai/gpt-4.1-mini",
        "messages": [
            {
                "role": "user",
                "content": "What is 2+2? Just give me the number."
            }
        ],
        "max_tokens": 10
    });

    println!("📤 Making test request to OpenRouter...");

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header(
            "HTTP-Referer",
            "https://github.com/tunahorse/tinyagent-rust",
        )
        .header("X-Title", "tiny-agent-rs")
        .json(&test_request)
        .send()
        .await?;

    println!("📥 Response status: {}", response.status());

    let response_text = response.text().await?;
    println!("📄 Response body:\n{}", response_text);

    // Try to parse the response
    if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
        if let Some(content) = response_json["choices"][0]["message"]["content"].as_str() {
            println!("✅ Success! Response: {}", content);
        }
    }

    Ok(())
}
