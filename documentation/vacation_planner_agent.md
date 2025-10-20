# Vacation Planner Agent Walkthrough

This guide explains how the `vacation_planner.rs` example wires together the refactored agent, the built-in `final_answer` workflow, and two tools (a Jina content fetcher and a custom budgeting helper) to produce a complete travel plan.

## Goals

- Demonstrate a multi-tool run where the agent reads live travel highlights and performs a quick cost estimate.
- Showcase the enforced `final_answer` tool: the model must call it to end the interaction.
- Provide a ready-to-run command users can copy after populating `.env`.

## Tooling Overview

| Tool | Purpose | Source |
| --- | --- | --- |
| `jina_reader` | Fetch markdown snapshots of the target URL via Jina’s API. | `src/tools/jina.rs` |
| `budget_calculator` | Compute trip lodging totals and per-person split. | Defined inline in `examples/vacation_planner.rs` |

Both tools expose JSON schemas so the model knows the required arguments. The example reuses `JinaReaderTool` from the library and defines a lightweight calculator struct that implements the `Tool` trait.

## Agent Setup (`examples/vacation_planner.rs`)

1. **Load credentials**
   ```rust
   let api_key = std::env::var("OPENAI_API_KEY")?;
   let jina_key = std::env::var("JINA_API_KEY")?;
   ```
2. **Register tools**
   ```rust
   let mut factory = FunctionFactory::new();
   factory.register_tool(JinaReaderTool::new(jina_key));
   factory.register_tool(BudgetCalculator);
   ```
3. **Configure agent**
   ```rust
   let agent = Agent::new(api_key, factory)
       .with_max_iterations(6);
   ```
4. **Task prompt** – Instructs the agent to:
   - Pull highlights from the Paris tourism page using `jina_reader`.
   - Estimate hotel cost for 3 nights at $240/night split between two travelers via `budget_calculator`.
   - Produce a concise itinerary plus budget notes.

5. **Execution** – `run_with_steps` records every ReAct step. The model must finish with:
   ```json
   { "tool": "final_answer", "arguments": { "answer": "..." } }
   ```
   so the agent returns a structured `RunResult`.

## Running the Example

Prepare environment variables:
```bash
export OPENAI_API_KEY=sk-...
export JINA_API_KEY=jina_...
```
Alternatively, add them to `.env` and source the file.

Run the demo from the repository root:
```bash
set -a && source .env && set +a && cargo run --example vacation_planner
```

The CLI prints:
- **Replay** – human-readable trace with task, actions, observations, and `final_answer`.
- **Detailed Explanation** – expanded view with tool call IDs, arguments, and observation payloads.

## What to Look For

- The first tool call grabs markdown describing “Paris in 1, 2 or 3 days”.
- The second tool call (budget calculator) outputs total and per-person lodging costs.
- The final step is a `final_answer` call summarizing itinerary highlights and budget notes.

If the model ever tries to answer directly without calling `final_answer`, the agent logs an error observation and asks it to retry, ensuring explicit termination.

## Extending

- Swap the Jina URL for a different city guide to generate new itineraries.
- Adjust the calculator to include flights or per-day spending.
- Integrate additional tools (weather, translation, booking APIs) by registering new `Tool` implementations.

This example demonstrates the full agentic flow: fetch → reason → calculate → finalize with `final_answer`.
