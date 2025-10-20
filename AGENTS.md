# Repository Guidelines

## Project Structure & Module Organization

- Core library entry point is `src/lib.rs`; agent planning, memory, and orchestration modules live under `src/agent/`.
- Built-in tools and registration utilities sit in `src/tools/`; implement new capabilities by extending the `Tool` trait in `tool.rs`.
- The CLI is defined in `src/main.rs` and `src/cli.rs`, guarded by the `cli` feature flag.
- Run integration tests from `tests/`; supporting notes live in `memory-bank/`, references in `documentation/`, and runnable scenarios in `examples/`.

## Build, Test, and Development Commands

- Always follow the ReAct pattern
  Example of ReAct loop:
  Reason: I need to know if a golden baseline test exists for this feature.
  Act: Search the tests/ directory for existing coverage.

- Before any work is doneyou myust make a commit rollback point even if empty
- `cargo check` validates the workspace quickly; run it before committing.
- `cargo fmt --all` enforces formatting; use `cargo fmt -- --check` to mirror CI.
- `cargo clippy --all-targets --all-features -D warnings` keeps lint issues out of PRs.
- `cargo test` executes unit and integration suites; add `-- --nocapture` when debugging.
- `cargo run --bin tiny-agent --features cli -- --help` confirms CLI wiring; `cargo run --example simple_agent --features cli` covers an end-to-end path.

## Coding Style & Naming Conventions

You MUST:

Enforce modern, clean and modular Rust patterns

Ensure clean modular design and flag code duplication (DRY violations).

Require linters and formatters as standard practice.

Verify that each feature is test-covered and tests accurately reflect intended behavior.

Enforce a clean, well-defined project structure (no misplaced logic or orphan files).

Require safe refactors: begin with characterization tests → apply refactor → verify with tests.

Suggest use of code intelligence tools (e.g., ASTGrep) for pattern detection and refactor support.

Demand updated README and clear directory structure for maintainability.

Treat tech debt proactively and as part of engineering hygiene.

You will be penalized for overlooking code smell, poor documentation, or unsafe changes.

Answer a question given in a natural, human-like manner. Use direct, affirmative language. Avoid filler, politeness, or passive voice.
- Stick to `rustfmt` defaults (4-space indent); document public APIs with `///` comments.
- Use `snake_case` for functions and modules, `PascalCase` for types/traits, and `SCREAMING_SNAKE_CASE` for constants.
- Prefer `tracing` spans for async logging, `thiserror` for structured errors, and feature-gate CLI additions with `#[cfg(feature = "cli")]`.
- Keep code files under 600, if needed you can temp havea file above but this must be minmal and commented to let the next agent know that is must be refactored for the next dev

## Testing Guidelines

- Write async tests with `#[tokio::test]`; rely on `tokio-test` helpers for deterministic timing.
- Use `mockito` for HTTP stubs when exercising OpenAI or OpenRouter integrations.
- Name tests after behaviors (`handle_multi_tool_call`); keep reusable fixtures in `tests/` or `examples/`.
- Add integration coverage when modifying planner logic or tool orchestration; document any env vars required by the test harness.

## Commit & Pull Request Guidelines

- Commit subjects should be imperative and Title Case (e.g., `Reorganize Source Structure and Fix Tool Choice Format`) with focused bodies.
- Reference issues with `Fixes #123` where relevant and separate mechanical from behavioral changes.
- PRs should include a crisp summary, test evidence (`cargo test`, `cargo clippy`), risk notes, and screenshots or JSON snippets when CLI output changes.
- Call out schema or API updates explicitly so downstream tooling can adapt.

## Configuration & Secrets

- Load API keys via `.env` consumed by `dotenvy`; never commit credentials.
- Document required environment variables (`OPENAI_API_KEY`, optional `OPENROUTER_API_KEY`) whenever new tools or examples depend on them.
