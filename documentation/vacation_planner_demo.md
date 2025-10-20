# Vacation Planner Demo: tiny-agent-rs in Action

This walkthrough captures a real run of `cargo run --example vacation_planner` and explains how tiny-agent-rs turns an LLM into a structured, tool-using travel planner.

## Scenario

- **Task:** Plan a 3-day Paris getaway for two adults.
- **Tools registered:** `jina_reader` (fetches markdown content) and `budget_calculator` (estimates lodging costs).
- **Requirements:** Use the `vacation_planner` schema, call `final_answer` to finish, keep the itinerary concise, and provide budget notes.

## Execution Highlights

1. **Traceability by default**  
   `AgentMemory` logs every step. During the run we observed:
   - 🧭 Task – the full multi-line instructions.
   - 🔧 Action – invocation of `jina_reader` with the Paris tourism URL.
   - 👁 Observation – fetched markdown payload returned to the model.
   - 🔧 Action – auto-generated budget calculator call with nightly rate and split.
   - ✅ Final Answer – structured schema payload delivered via the enforced `final_answer` tool.

2. **Schema-enforced final answer**  
   The agent rejected direct assistant text until the model produced:

   ```json
   {
     "tool": "final_answer",
     "arguments": {
       "answer": "Your Paris long-weekend is ready...",
       "structured": { ... vacation_planner schema ... }
     }
   }
   ```

   The response included day-by-day activities, highlights, and a budget summary matching the schema shape.

3. **Budget-aware insights**  
   The structured payload reported:

   - `budget_total`: `$1,140` per person (`$2,280` total).  
   - `accommodation`: “Central Paris hotel, 3 nights at $240/night ($720 total, $360 per person)”.  
   - Daily activity breakdowns (Eiffel Tower, Louvre, Montmartre, Latin Quarter, etc.).

4. **Replay & audit**  
   After completion, `RunResult::replay()` printed the annotated execution trace, making it easy to audit which tools were used and why. Token counts, elapsed time, and iteration totals were also returned.

## Why this matters

- **Deterministic orchestration:** The final answer had to pass schema validation; the agent would have retried otherwise.
- **Type-safe tools:** Tool parameters were `serde`-validated, so malformed arguments never hit business logic.
- **Observability built-in:** Developers get structured telemetry (`RunResult`, token usage, per-step logs) without additional plumbing.
- **Configurable transport:** The same run can target different OpenRouter-compatible backends via `--base-url` or env vars.

## Reproducing the demo

```bash
export OPENAI_API_KEY=...
export JINA_API_KEY=...
cargo run --example vacation_planner
```

Expect a ~60 second run (dominated by the Jina HTTP call) and structured output similar to:

```json
{
  "destination": "Paris, France",
  "days": 3,
  "currency": "USD",
  "budget_total": 1140.0,
  "itinerary": [
    { "day": 1, "activities": ["Eiffel Tower", "Seine cruise"], ... },
    { "day": 2, "activities": ["Louvre", "Montmartre"], ... },
    { "day": 3, "activities": ["Latin Quarter", "Shopping"], ... }
  ],
  "highlights": [
    "Eiffel Tower and Champs-Elysées",
    "Louvre Museum",
    "Montmartre and Sacre Coeur",
    "Seine River cruises"
  ],
  "notes": "Book attractions in advance... Total estimated cost: $1,140 per person."
}
```

This demo illustrates how tiny-agent-rs enforces ReAct discipline, keeps secrets out of logs, delivers structured outputs, and remains fully auditable—key ingredients for shipping a trustworthy agent beta.
