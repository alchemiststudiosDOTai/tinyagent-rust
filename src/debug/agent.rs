//! Debug the API key loading

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("Debugging API Key Loading");
    println!("==========================");

    // Check what's in the environment
    match env::var("OPENAI_API_KEY") {
        Ok(key) => {
            println!("Found OPENAI_API_KEY in environment");
            println!("Length: {}", key.len());
            println!("Masked: {}", "*".repeat(key.len().min(8)));
        }
        Err(e) => {
            println!("Error loading OPENAI_API_KEY: {}", e);
        }
    }

    // Load from .env file manually
    println!("\nReading .env file directly:");
    let env_content = std::fs::read_to_string(".env")?;
    for line in env_content.lines() {
        if let Some(value) = line.strip_prefix("OPENAI_API_KEY=") {
            println!("Found OPENAI_API_KEY entry in .env");
            println!("Length: {}", value.len());
        }
    }

    Ok(())
}
