# Agent Implementation Guidelines

The points below summarize broad practices we should follow when evolving the agent. They are phrased generically so they remain useful even as the concrete code changes.

## Configuration & Secrets
- Load runtime configuration (API keys, models, iteration caps, timeouts) from the environment or a config layer instead of hard-coding values.
- Surface configuration errors as `Result` values where possible so applications can decide how to react rather than crashing the process.
- Avoid embedding secrets in source code, tests, or defaults. Rely on `.env` files, secret managers, or CI-provided variables.

## Error Handling
- Normalize HTTP failures before decoding payloads so upstream callers receive accurate status information.
- Preserve context when propagating errors (e.g., network layer vs. JSON parsing) to reduce debugging time.
- Treat tool invocation failures explicitly—either short-circuit with an error or return structured data the LLM can act upon.

## Resource Management
- Reuse HTTP clients or other heavyweight resources instead of recreating them each request; enable pooling, retry, and proxy configuration in one place.
- Keep request construction functions side-effect-free and avoid work that is immediately discarded (for example unused conversions or allocations).

## Observability & Debuggability
- Emit logs or metrics around retries, timeouts, and tool execution outcomes so production incidents can be diagnosed quickly.
- Validate and log tool argument parsing errors rather than silently substituting placeholder values.

## Extensibility
- Provide builder-style configuration for optional settings, keeping `Default` lightweight and infallible.
- Encapsulate protocol-specific constructs (e.g., OpenRouter tool schema) so swapping providers or adding features does not cascade through the codebase.

## Developer Workflow
- Configure the shared Git hooks so linting and tests run automatically before each commit:
  ```bash
  git config core.hooksPath githooks
  ```
- The pre-commit hook executes `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. Resolve any failures locally before committing.

## Security & Compliance
- Ensure outbound requests set only the headers required by the provider; review and document any identifiers or referers we send.
- Respect provider rate limits and introduce backoff strategies to avoid abusive patterns.

Adhering to these practices should keep the agent robust, maintainable, and adaptable as requirements evolve.
