use crate::{
    tools::{CalculatorTool, WeatherTool},
    Agent, FunctionFactory,
};
use clap::{Arg, Command};
use dotenvy;
use std::env;
use tracing::{error, info};

/// CLI entry point for the tiny-agent tool
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let matches = Command::new("tiny-agent")
        .version("0.1.0")
        .about("A lightweight Rust agent for LLM tool calling with OpenRouter")
        .arg(
            Arg::new("prompt")
                .help("The prompt to send to the agent")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL")
                .help("The OpenRouter model to use")
                .default_value("openai/gpt-4.1-mini"),
        )
        .arg(
            Arg::new("api-key")
                .short('k')
                .long("api-key")
                .value_name("KEY")
                .help("OpenRouter API key (or set OPENAI_API_KEY env var)"),
        )
        .arg(
            Arg::new("base-url")
                .short('u')
                .long("base-url")
                .value_name("URL")
                .help(
                    "OpenRouter base URL (or set OPENAI_BASE_URL / OPENROUTER_BASE_URL env vars)",
                ),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("SECONDS")
                .help("Request timeout in seconds")
                .default_value("120"),
        )
        .arg(
            Arg::new("max-iterations")
                .short('i')
                .long("max-iterations")
                .value_name("COUNT")
                .help("Maximum agent iterations")
                .default_value("10"),
        )
        .get_matches();

    // Get API key from argument or environment
    let api_key = matches
        .get_one::<String>("api-key")
        .cloned()
        .or_else(|| env::var("OPENAI_API_KEY").ok())
        .ok_or("OpenRouter API key is required. Set OPENAI_API_KEY environment variable or use --api-key")?;

    // Resolve base URL from CLI or environment
    let base_url = matches
        .get_one::<String>("base-url")
        .cloned()
        .or_else(|| env::var("OPENAI_BASE_URL").ok())
        .or_else(|| env::var("OPENROUTER_BASE_URL").ok())
        .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());

    // Set up function factory with tools
    let mut function_factory = FunctionFactory::new();
    function_factory.register_tool(CalculatorTool::new());
    function_factory.register_tool(WeatherTool::new());

    // Create agent
    let timeout_seconds: u64 = matches.get_one::<String>("timeout").unwrap().parse()?;
    let max_iterations: usize = matches
        .get_one::<String>("max-iterations")
        .unwrap()
        .parse()?;

    let agent = Agent::new(api_key, function_factory)
        .with_model(matches.get_one::<String>("model").unwrap().as_str())
        .with_timeout(std::time::Duration::from_secs(timeout_seconds))
        .with_max_iterations(max_iterations)
        .with_base_url(base_url.clone());

    // Run the agent
    let prompt = matches.get_one::<String>("prompt").unwrap();
    info!("Running agent with prompt: {}", prompt);
    info!(
        "Using model: {}",
        matches.get_one::<String>("model").unwrap()
    );
    info!("Base URL: {}", base_url);

    match agent.run(prompt).await {
        Ok(response) => {
            println!("\nAgent Response:\n{}", response);
            info!("Agent execution completed successfully");
        }
        Err(e) => {
            error!("Agent execution failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
