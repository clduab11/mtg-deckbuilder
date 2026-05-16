# AGENTS.md

## Commands

- Format: `cargo fmt --check`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Test: `cargo test --all-features`
- Release build: `cargo build --release`

## Boundaries

- Rust is the primary implementation language.
- No Python runtime dependency in the main execution path.
- Use "Bo1/Bo3-oriented"; do not claim exact MTG Arena parity without validated evidence.
- LLMs may consume structured reports but must not control deterministic simulation outcomes.
- Do not scrape, reverse engineer, automate, or control MTG Arena or competitor clients.
- Keep repo-level agent context short and tactical.
