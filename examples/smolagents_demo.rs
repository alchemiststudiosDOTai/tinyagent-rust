use tiny_agent_rs::{tools::CalculatorTool, Agent, FunctionFactory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    let api_key = std::env::var("OPENAI_API_KEY")?;

    // Set up tools
    let mut factory = FunctionFactory::new();
    factory.register_tool(CalculatorTool::new());

    // Create agent with step-based execution enabled
    let agent = Agent::new(api_key, factory).with_max_iterations(5);

    println!("=== Smolagents-Style Agent Demo ===\n");

    // Execute a task that requires tool usage
    let task = "Calculate 157 * 89, then add 42 to the result";
    println!("Task: {}\n", task);

    let result = agent.run_with_steps(task).await?;

    // Display execution trace with replay
    println!("{}", result.replay());

    println!("\n--- Detailed Explanation ---");
    println!("{}", result.explain());

    // Show execution statistics
    println!("\n--- Statistics ---");
    println!("Total steps: {}", result.steps.len());
    println!("Actions taken: {}", result.action_count());
    println!("Observations: {}", result.observation_count());
    println!("Success: {}", result.is_success());

    if let Some(tokens) = &result.tokens {
        println!("\nToken usage:");
        println!("  Prompt tokens: {}", tokens.prompt_tokens);
        println!("  Completion tokens: {}", tokens.completion_tokens);
        println!("  Total: {}", tokens.total_tokens);
    }

    println!("\nDuration: {:.2}s", result.duration.as_secs_f64());
    println!("Iterations: {}", result.iterations);

    Ok(())
}
