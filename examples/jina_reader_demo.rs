use tiny_agent_rs::{
    tools::{CalculatorTool, JinaReaderTool},
    Agent, FunctionFactory,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load keys from environment
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let jina_key = std::env::var("JINA_API_KEY")?;

    let mut factory = FunctionFactory::new();
    factory.register_tool(CalculatorTool::new());
    factory.register_tool(JinaReaderTool::new(jina_key));

    let agent = Agent::new(api_key, factory).with_max_iterations(6);

    println!("=== Jina Reader Tool Demo ===\n");

    let task = "Use the jina_reader tool to fetch markdown for https://www.example.com, then summarize the content in one sentence.";
    println!("Task: {}\n", task);

    let result = agent.run_with_steps(task).await?;

    println!("{}", result.replay());

    println!("\n--- Detailed Explanation ---");
    println!("{}", result.explain());

    Ok(())
}
