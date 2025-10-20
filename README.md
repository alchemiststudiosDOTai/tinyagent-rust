# tiny-agent-rs

A lightweight, type-safe Rust agent library for LLM tool calling with strong typing and deterministic error handling.

## Features

- **Type-Safe Tool Calling**: Define tools with Rust types and automatic JSON Schema generation
- **Deterministic Error Handling**: Comprehensive error types with structured payloads
- **Async-First**: Built on Tokio for efficient async execution
- **Modular Design**: Clean separation between tools, validation, and agent logic
- **OpenAI Integration**: Native support for OpenAI function calling
- **CLI Tool**: Ready-to-use command-line interface

## Quick Start

### Installation

```bash
cargo install tiny-agent-rs --features cli
```

### Basic Usage

```rust
use tiny_agent_rs::{Agent, FunctionFactory, tools::{CalculatorTool, WeatherTool}};
use async_openai::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create OpenAI client
    let client = Client::new();

    // Set up function factory with tools
    let mut function_factory = FunctionFactory::new();
    function_factory.register_tool(CalculatorTool::new());
    function_factory.register_tool(WeatherTool::new());

    // Create agent
    let agent = Agent::new(client, function_factory);

    // Run agent
    let response = agent.run("What is 15 + 27?").await?;
    println!("{}", response);

    Ok(())
}
```

### CLI Usage

```bash
# Set your OpenAI API key
export OPENAI_API_KEY="your-api-key-here"

# Run the agent
tiny-agent "What is the weather in Tokyo and 25 * 4?"

# Use different model
tiny-agent --model gpt-4 "Calculate 2^10"

# Custom timeout and iterations
tiny-agent --timeout 120 --max-iterations 20 "Complex query"
```

## Creating Custom Tools

```rust
use tiny_agent_rs::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyToolParams {
    pub input: String,
}

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &'static str {
        "my_tool"
    }

    fn description(&self) -> &'static str {
        "A custom tool example"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        schemars::schema_for!(MyToolParams).into()
    }

    async fn execute(&self, parameters: serde_json::Value) -> Result<serde_json::Value, AgentError> {
        let params: MyToolParams = serde_json::from_value(parameters)?;

        // Your tool logic here
        let result = format!("Processed: {}", params.input);

        Ok(serde_json::json!({ "result": result }))
    }
}
```

## Architecture

- **Agent**: Main orchestrator handling LLM interactions and tool execution
- **FunctionFactory**: Registry and execution manager for tools
- **Tool**: Trait for implementing callable functions with schema validation
- **Validator**: Parameter validation using serde or JSON Schema
- **Error**: Comprehensive error handling with structured payloads

## Examples

See the `examples/` directory for more complete examples:

- Basic calculator tool
- Weather information tool
- Custom tool implementation
- Error handling patterns

## Development

```bash
# Clone the repository
git clone https://github.com/tunahorse/tinyagent-rust.git
cd tinyagent-rust

# Run tests
cargo test

# Run example
cargo run --example simple_agent --features cli

# Build with CLI
cargo build --features cli
```

### Pre-commit Hooks and Secret Scanning

- Point Git hooks at `githooks/`: `git config core.hooksPath githooks`.
- Install [gitleaks](https://github.com/gitleaks/gitleaks) and ensure it is on your `PATH`; the pre-commit hook fails fast if it cannot run `gitleaks protect --staged --redact`.
- The hook sequence is `gitleaks` → `cargo fmt --all -- --check` → `cargo clippy --all-targets --all-features -D warnings` → `cargo test`.
- Run `gitleaks detect --redact --source .` before opening pull requests to scan the full tree.

## Requirements

- Rust 1.70+
- OpenAI API key for LLM integration
- Tokio runtime

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] Additional LLM provider support
- [ ] Streaming responses
- [ ] Tool result caching
- [ ] More built-in tools
- [ ] WASM support
