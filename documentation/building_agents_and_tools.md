# Building Agents and Tools in tiny-agent-rs

This guide shows how to assemble an agent and author a reusable tool inside the `tiny-agent-rs` workspace. Every step below is backed by an in-repo source citation so you can verify the behavior.

## Agent Workflow

1. **Load runtime credentials from your environment**  
   Proof: `examples/basic_agent.rs:9-11` reads `OPENROUTER_API_KEY` before any agent setup.

2. **Create a `FunctionFactory` to manage tools (optional if you run tool-free)**  
   Proof: `src/lib.rs:14-18` initializes a `FunctionFactory`, registers a tool, and passes it into `Agent::new`.

3. **Configure the agent fluently before execution**  
   Proof: `examples/basic_agent.rs:17-19` chains `.with_model` and `.with_timeout` on the freshly constructed agent.

4. **Call `run` or `run_with_steps` to execute prompts**  
   Proof: `examples/basic_agent.rs:25-44` drives two prompts through `agent.run`, while `tests/integration_tests.rs:114-167` exercises `run_with_steps` and asserts the structured output.

## Example: Minimal Agent Without Tools

```rust
use tiny_agent_rs::Agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENROUTER_API_KEY")?;
    let agent = Agent::new(api_key, tiny_agent_rs::FunctionFactory::new())
        .with_model("microsoft/wizardlm-2-8x22b")
        .with_timeout(std::time::Duration::from_secs(60));

    let reply = agent.run("What is 25 * 4? Just give the number.").await?;
    println!("{reply}");
    Ok(())
}
```

Proof: Directly adapted from `examples/basic_agent.rs:3-47`, which compiles and runs as part of the examples suite.

## Attaching Tools to an Agent

1. **Register tools on the `FunctionFactory` before constructing the agent**  
   Proof: `tests/integration_tests.rs:48-68` registers the calculator and weather tools, then invokes `execute_function` successfully.

2. **Expose tool schemas to the model automatically**  
   Proof: `src/tools/tool.rs:54-68` shows `ToolRegistry::to_openai_tools` formatting each tool into OpenAI-compatible function metadata that the agent forwards upstream.

## Creating a Tool

1. **Model the request payload with `serde` (and `schemars` for schemas)**  
   Proof: `src/tools/calculator.rs:5-22` defines `CalculatorParams` and `Operation` with `Serialize`, `Deserialize`, and `JsonSchema` derives.

2. **Implement the `Tool` trait to declare metadata and behavior**  
   Proof: `src/tools/calculator.rs:40-99` implements `Tool::name`, `description`, `parameters_schema`, and an async `execute` body.

3. **Return structured JSON results or `AgentError`**  
   Proof: `src/tools/calculator.rs:74-98` maps invalid parameters to `AgentError::ToolExecution` and returns a JSON object containing the computed result.

4. **Register the tool so agents can invoke it**  
   Proof: `src/tools/function_factory.rs:19-32` stores tools in the registry and dispatches to their async `execute` implementations.

## Example: Calculator Tool Skeleton

```rust
use tiny_agent_rs::tools::Tool;

#[derive(Debug)]
pub struct ExampleTool;

impl Tool for ExampleTool {
    fn name(&self) -> &'static str { "example" }
    fn description(&self) -> &'static str { "Say hello." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {}, "required": [] })
    }
    fn execute(
        &self,
        _parameters: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = tiny_agent_rs::Result<serde_json::Value>> + Send>> {
        Box::pin(async { Ok(serde_json::json!({ "message": "hello" })) })
    }
}
```

Proof: Mirrors the structure in `src/tools/calculator.rs:40-99`; swapping the business logic still satisfies the same trait contract.

## Validating Your Implementation

- **Unit and integration coverage** confirm both the tool behavior and agent orchestration.  
  Proof: `tests/integration_tests.rs:7-84` validates calculator and weather tools, schema generation, and error handling.

- **End-to-end agent runs** verify step recording and replay/explain helpers.  
  Proof: `tests/integration_tests.rs:101-167` drives a full agent run through `run_with_steps` and asserts the resulting telemetry.

- **Formatting, linting, and tests** run automatically whenever you commit.  
  Proof: `githooks/pre-commit:1-38` invokes `cargo fmt --check`, `cargo clippy --all-targets --all-features -D warnings`, and `cargo test` after a gitleaks scan.
