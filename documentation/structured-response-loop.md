# Structured Response Termination Loop

## Context

- Feature: vacation planner example using `VacationPlan` completion schema.
- Change: enabling OpenAI `response_format` JSON schema enforcement while still requiring a `final_answer` tool call.

## Observed Behavior

- When schema response_format is active, the LLM returns the structured JSON **as plain content**, not via the `final_answer` tool.
- Agent logic insists on receiving a `final_answer` tool call and treats plain responses as errors.
- Result: the agent repeatedly reminds the model to call `final_answer`, burning iterations until `MaxIterations` is hit.

## Why It Happens

- JSON schema mode constrains every assistant turn; the API will not allow additional tool calls once it has emitted the schema object.
- Our agent currently expects both:
  1. The model to satisfy the schema (via `response_format`).
  2. The model to issue a `final_answer` tool call wrapping the same payload.

These requirements conflict. The LLM cannot meet them simultaneously under the new schema enforcement.

## Open Questions

1. Should we accept the schema-compliant assistant message as the terminal turn when `response_format` is active?
2. Alternatively, should we stop using `response_format` and rely solely on the `final_answer.structured` field for schema validation?
3. Do we want a fallback that detects the schema response and uses it to synthesize a `final_answer` automatically?

## Potential Directions

- **Option A:** Treat a valid schema response as completion and construct the `RunResult` without requiring `final_answer`.
- **Option B:** Disable `response_format` and let the model call `final_answer` with both natural language and structured payload, validating locally.
- **Option C:** Keep both but insert a system message clarifying the required sequence and verify that supported models can follow it.

## Next Steps

- Decide on desired agent contract (schema-only vs. schema + tool).
- Adjust agent loop accordingly.
- Add integration coverage once behavior is finalized.

we ended
