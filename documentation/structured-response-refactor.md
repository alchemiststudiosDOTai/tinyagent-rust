# Structured Response Refactor

## Problem Statement

The original implementation used a two-step process for structured responses:
1. Agent calls `final_answer` with natural language summary
2. System injects follow-up prompt: "Now call `structured_response`"
3. Agent attempts to call `structured_response` with schema-compliant JSON

### Issues with Original Approach

1. **Tool Confusion**: Agent would repeatedly try to call `final_answer` after it was already called, causing `INVALID_FUNCTION_CALL` errors
2. **Schema Blindness**: The `structured_response` tool didn't expose the actual schema structure, forcing the model to guess field names and types
3. **Validation Barriers**: Multiple checks prevented `structured_response` from being called without `final_answer` first
4. **Prompt Conflicts**: Error messages and reminders referenced both tools simultaneously, confusing the model

### Example Error Loop
```
1. Agent calls final_answer ✓
2. System: "Now call structured_response"
3. Agent tries final_answer again ✗ (already called)
4. Agent tries structured_response with wrong structure ✗ (validation fails)
5. Agent tries final_answer again ✗ (already called)
... repeats until max iterations
```

---

## Solution: Single-Pass Structured Response

### Core Changes

#### 1. Conditional Tool Exposure
**File**: `src/agent/mod.rs` (lines ~132-137, ~593-598)

**Before**:
```rust
let mut tools = self.function_factory.get_openai_tools();
tools.push(Self::final_answer_tool_definition());
if let Some(schema) = &self.completion_schema {
    tools.push(Self::structured_response_tool_definition(schema));
}
```

**After**:
```rust
let mut tools = self.function_factory.get_openai_tools();
if let Some(schema) = &self.completion_schema {
    tools.push(Self::structured_response_tool_definition(schema));
} else {
    tools.push(Self::final_answer_tool_definition());
}
```

**Rationale**: When a completion schema is set, ONLY expose `structured_response`. When no schema is set, ONLY expose `final_answer`. This eliminates tool confusion by giving the agent exactly one way to complete the task.

---

#### 2. Schema Injection in Tool Definition
**File**: `src/agent/mod.rs` (lines ~1014-1055)

**Before**:
```rust
"structured": {
    "type": "object",
    "description": "JSON object that must satisfy the schema",
    "additionalProperties": true
}
```

**After**:
```rust
// Inject the actual schema properties
if let Some(schema_props) = schema.schema_json().get("properties") {
    structured_param.insert("properties".to_string(), schema_props.clone());
}
if let Some(schema_required) = schema.schema_json().get("required") {
    structured_param.insert("required".to_string(), schema_required.clone());
}
```

**Rationale**: This is the **critical fix**. By embedding the actual schema structure (e.g., VacationPlan's `destination`, `nights`, `itinerary` fields) directly into the tool definition, the model can see exactly what's required, just like it sees parameters for any other tool. Previously, the model had to blindly guess the structure.

**Example**: For VacationPlan, the model now sees:
```json
{
  "name": "structured_response",
  "parameters": {
    "structured": {
      "properties": {
        "destination": {"type": "string"},
        "nights": {"type": "integer"},
        "itinerary": {"type": "array", ...},
        ...
      },
      "required": ["destination", "nights", ...]
    }
  }
}
```

---

#### 3. Removed Final Answer Prerequisites
**File**: `src/agent/mod.rs` (lines ~446-451, ~869-871)

**Before**:
```rust
let answer_string = match &final_answer_value {
    Some(value) => value.clone(),
    None => {
        let payload = AgentError::InvalidFunctionCall(
            "Structured response received before final_answer was recorded"
        ).to_error_payload();
        // ... error handling ...
        continue;
    }
};
```

**After**:
```rust
let answer_string = final_answer_value
    .clone()
    .unwrap_or_else(|| "Task completed with structured response".to_string());
```

**Rationale**: Remove validation barriers. If `final_answer` was never called (which is expected in the new flow), use a default message. No errors, no confusion.

---

#### 4. Updated System Prompt
**File**: `src/agent/mod.rs` (lines ~1025-1030)

**Before**:
```rust
"when you finish, you MUST call the `final_answer` tool with a concise 
natural language summary for the user. After `final_answer` is acknowledged, 
you will receive instructions to call the `structured_response` tool..."
```

**After**:
```rust
"when you finish the task, you MUST call the `structured_response` tool 
with a JSON payload that strictly conforms to the `VacationPlan` schema. 
This is the ONLY way to complete the task."
```

**Rationale**: Clear, direct instruction. No mention of `final_answer`, no multi-step process. Just "use this one tool to finish."

---

#### 5. Removed Follow-Up Prompt Logic
**File**: `src/agent/mod.rs` (removed lines ~130-138, ~598-606)

**Before**:
```rust
if let Some(schema) = &self.completion_schema {
    if has_final_answer && !schema_follow_up_inserted {
        messages.push(json!({
            "role": "system",
            "content": Self::structured_follow_up_instruction(schema)
        }));
        schema_follow_up_inserted = true;
    }
}
```

**After**: *(removed entirely)*

**Rationale**: No longer needed. There's no second step, so no follow-up prompt to inject.

---

#### 6. Updated Error Reminders
**File**: `src/agent/mod.rs` (lines ~937-959)

**Before**:
```rust
let content = if !has_final_answer {
    // Remind to call final_answer
} else {
    // Remind to call structured_response
};
```

**After**:
```rust
let content = if self.completion_schema.is_some() {
    // Remind to call structured_response
} else {
    // Remind to call final_answer
};
```

**Rationale**: Base reminder on schema presence, not on whether `final_answer` was called. Consistent messaging throughout.

---

## Architecture: Before vs After

### Before (Two-Step)
```
┌─────────────────────────────────────────────────┐
│ Available Tools:                                │
│  - search_web, budget_calculator, ...           │
│  - final_answer                                 │
│  - structured_response                          │
└─────────────────────────────────────────────────┘
                    ↓
         Agent calls final_answer
                    ↓
         System injects follow-up prompt
                    ↓
         Agent must call structured_response
         (but still sees final_answer as option!)
                    ↓
         CONFUSION: Which tool to call?
                    ↓
         ❌ Errors and retries
```

### After (Single-Pass)
```
┌─────────────────────────────────────────────────┐
│ Available Tools:                                │
│  - search_web, budget_calculator, ...           │
│  - structured_response (with full schema spec)  │
│  [final_answer NOT exposed]                     │
└─────────────────────────────────────────────────┘
                    ↓
         Agent uses tools to gather data
                    ↓
         Agent calls structured_response
         (sees exact fields required)
                    ↓
         ✓ Success on first try
```

---

## Testing Results

### Vacation Planner Example
**Command**: `cargo run --example vacation_planner --features cli`

**Output**:
```
✓ Agent used jina_reader to fetch Paris tourism info
✓ Agent used budget_calculator to compute costs
✓ Agent called structured_response with valid VacationPlan JSON
✓ All required fields present (destination, nights, itinerary, etc.)
✓ No validation errors
✓ No tool confusion
✓ Single clean execution path
```

**Generated Structure**:
```json
{
  "destination": "Paris, France",
  "nights": 3,
  "travelers": 2,
  "total_budget": 720.0,
  "budget_per_person": 360.0,
  "itinerary": [
    {"day": 1, "activities": [...], "estimated_cost": 250.0},
    {"day": 2, "activities": [...], "estimated_cost": 200.0},
    {"day": 3, "activities": [...], "estimated_cost": 270.0}
  ],
  "highlights": ["Eiffel Tower", "Louvre Museum", ...],
  ...
}
```

---

## Key Insights

1. **Single Tool = Single Path**: When there's only one finishing tool available, the agent can't get confused about which to call.

2. **Schema Visibility = Correct Output**: Embedding the schema in the tool definition is like showing the model a form to fill out, rather than asking it to guess the form structure.

3. **Fewer State Transitions = Fewer Bugs**: The two-step process introduced state (`has_final_answer`, `schema_follow_up_inserted`) that had to be tracked and validated. Single-pass eliminates this complexity.

4. **Treat Structured Response Like Any Other Tool**: The breakthrough was realizing structured responses should work exactly like tools—with a clear parameter specification the model can see.

---

## Migration Notes

### For Users of the Library

**No Breaking Changes for Non-Schema Usage**: If you're not using `completion_schema`, the behavior is unchanged—`final_answer` works as before.

**With Schema**: The `final_answer` natural language summary is now optional. If your code expects both a text answer AND structured output:
- The text answer defaults to `"Task completed with structured response"`
- You can still provide context in the task prompt if you want richer output

### For Future Schema Definitions

Ensure your schemas are well-defined with:
- Clear `properties` for each field
- Explicit `required` arrays
- Proper `type` annotations (string, integer, array, object, etc.)
- Helpful `description` fields (these appear in the tool definition)

The better your schema, the better the model can generate compliant output.

---

## Files Modified

| File | Lines | Changes |
|------|-------|---------|
| `src/agent/mod.rs` | 132-137, 593-598 | Conditional tool exposure |
| `src/agent/mod.rs` | 1014-1055 | Schema injection in tool definition |
| `src/agent/mod.rs` | 446-451, 869-871 | Removed final_answer prerequisites |
| `src/agent/mod.rs` | 1025-1030 | Updated system prompt |
| `src/agent/mod.rs` | 130-138, 598-606 | Removed follow-up logic |
| `src/agent/mod.rs` | 937-959 | Updated error reminders |
| `src/agent/mod.rs` | 119-121, 566-568 | Removed unused variables |
| `src/agent/mod.rs` | 1073-1080 | Removed `structured_follow_up_instruction` |

---

## Conclusion

This refactor transforms structured responses from a fragile two-step dance into a robust single-pass operation. By treating the completion schema like any other tool parameter specification, we enable the LLM to generate correct, schema-compliant output on the first try.

The fix demonstrates a key principle in LLM tool usage: **make implicit knowledge explicit**. Don't ask the model to remember a schema—show it the schema in the tool definition.
