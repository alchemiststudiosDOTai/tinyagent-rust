# Tiny Agent Refactor Explained

This document walks through the refactored Smolagents-style agent and explains how the new pieces fit together.

## High-Level Workflow

1. **Configure the agent.** Construct `Agent` with an API key and registered tools.
2. **Start a run.** Call `run_with_steps(prompt)` which seeds an `AgentMemory` instance with the task.
3. **Talk to the model.** Build an OpenRouter chat request (model, messages, optional tools) and await the response.
4. **Follow tool calls.** When the model issues tool calls, execute them through `FunctionFactory` and append observations to memory.
5. **Stop on final answer.** As soon as the model responds with normal assistant content, capture it as the final answer and wrap everything into a `RunResult`.
6. **Inspect the trace.** Use `RunResult` helpers (`replay`, `explain`, `action_count`, `errors`, …) to review what happened.

## Core Components

### Agent (`src/agent/mod.rs`)
- Holds runtime configuration: API key, model (`openai/gpt-4.1-mini` by default), maximum iterations, token limits, timeout, and a `FunctionFactory`.
- Provides builder-style setters (`with_model`, `with_max_iterations`, …).
- Exposes the legacy `run` / `run_with_messages` methods for backward compatibility, plus the new structured `run_with_steps` API.

### AgentMemory (`src/agent/memory.rs`)
- Stores every reasoning step (`AgentStep`) while preserving an optional system prompt.
- Converts the internal representation back into OpenAI-compatible chat messages via `as_messages()`.
- Supports analytics helpers such as `count_actions` / `count_observations`.
- Includes an adapter `impl From<Vec<Value>>` so older code based on raw message vectors still works.

### AgentStep (`src/agent/steps.rs`)
- Enum capturing the reasoning lifecycle: `Task`, optional `Planning`, tool `Action`, resulting `Observation` (with error flag), and `FinalAnswer`.
- `describe()` produces the emoji-labelled trace strings used by `RunResult::replay()`.
- `to_message()` converts each step back into the roles OpenAI expects (`user`, `assistant`, `tool`).

### RunResult (`src/agent/result.rs`)
- Bundles the final output string with full step history, timing metrics, iteration count, and optional token usage stats from the API.
- Provides user-facing helpers: `replay()` (concise trace), `explain()` (verbose JSON-like dump), `is_success()`, `errors()`, etc.

### Tool Execution Helpers (`src/agent/tool_call.rs`)
- Defines `ToolCall`, `ToolExecution`, and `ToolOutput` structures used when executing tools and serializing responses.

### Built-in `final_answer` Tool (`src/agent/mod.rs`)
- Added automatically to every request so the model must explicitly signal completion.
- Requires an `answer` string (optional metadata is accepted) and short-circuits the loop once invoked.
- Direct assistant text without calling this tool is rejected; the agent asks the model to retry via an error observation.

### Planning Utilities (`src/agent/planning.rs`)
- Supplies prompt builders (`generate_planning_prompt`, `generate_tool_planning_prompt`) and the `is_planning_response` detector.
  - Planning helpers (`planning.rs`) remain available for future iterative strategies.

## `run_with_steps` Execution Lifecycle

1. **Initialize memory.** Start a timer (`Instant::now`), create `AgentMemory::with_default_system()`, and record the incoming task as a `Task` step.
2. **Iteration loop.** Repeat until `max_iterations` is reached:
   - Convert the accumulated steps into chat messages (`memory.as_messages()`).
   - Fetch available tools from the factory and inject them into the OpenRouter payload when present.
   - Submit the request via `make_raw_request` (with retries and timeout protection).
3. **Read the model response.** Pull the first choice, capture token usage metrics, and inspect the assistant message:
   - **Tool call path:** For each declared tool call
     - Parse arguments, record an `Action` step, and hand execution to `FunctionFactory::execute_function`.
     - Record the result (or error payload) as an `Observation` step.
   - **Final answer path:** When the model calls `final_answer`, validate the payload, record a `FinalAnswer` step, stop the loop, and build the `RunResult`.
   - **Invalid completion:** Plain assistant text without a `final_answer` call results in an error observation and another iteration.
4. **Timeout / safeguards.** Propagate structured `AgentError`s for timeouts, malformed responses, invalid tool arguments, or hitting `max_iterations` without an answer.

## Flowchart Overview

```mermaid
flowchart TB
    Task[User Task]
    Memory[agent.memory]
    Generate[Model Call]
    Final{Tool == final\_answer?}
    Action[Execute Tool]
    Observe[Log Observation]
    Answer[Return RunResult]

    Task -->|add Task step| Memory

    subgraph ReAct Loop
        Memory -->|as_messages()| Generate
        Generate --> Final
        Final -->|Yes| Answer
        Final -->|No| Action
        Action --> Observe -->|add steps| Memory
    end

    Answer -->|FinalAnswer step + metrics| End((Done))
```

The loop repeats until the model either calls `final_answer` (ending the run) or reaches the configured safety limits.

## Inspecting Results

```rust
let result = agent.run_with_steps("Calculate 157 * 89, then add 42").await?;
println!("{}", result.replay());   // concise overview
println!("{}", result.explain());  // full detail
assert!(result.is_success());
assert!(result.action_count() >= 1);
```

- `replay()` shows emoji markers (`🧭` Task, `🔧` Action, `👁` Observation, `✅` Final Answer, `❌` Error).
- `explain()` adds tool call IDs, serialized arguments, and error flags for debugging.
- `errors()` returns a list of tool failure payloads when something goes wrong.

## Running the Example

1. Add your OpenRouter key to `.env`:
   ```env
   OPENAI_API_KEY=sk-...
   MODEL=z-ai/glm-4.5-air   # optional override
   ```
2. Export the variables and run the demo:
   ```bash
   set -a && source .env && set +a && cargo run --example smolagents_demo
   ```
3. Inspect the printed trace, explanation, and stats. Adjust `with_max_iterations` or registered tools to explore different behaviors.

## Extending the Agent

- **Add tools:** Implement the `Tool` trait, register with `FunctionFactory::register_tool`, and the agent will automatically surface it to the model.
- **Change models:** Call `with_model("openai/gpt-4.1-mini")` (or any OpenRouter-supported alias) to swap providers.
- **Tune limits:** Use `with_max_tokens`, `with_timeout`, and `with_max_iterations` to guard cost and latency.
- **Use Jina reader:** Register `JinaReaderTool::new(std::env::var("JINA_API_KEY")?)` to fetch markdown snapshots of web pages via Jina's reader API.
- **Reinforce finalization:** Keep prompt templates reminding the model to end with the `final_answer` tool to avoid hitting the iteration ceiling.

This architecture keeps the runtime compatible with the prior API while exposing rich telemetry for debugging, replay, and integration testing.
