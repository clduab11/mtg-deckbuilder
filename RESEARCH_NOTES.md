# Research Notes

Date: 2026-05-16

## Wizards and MTG Arena Sources

- Wizards rules page: https://magic.wizards.com/rules. The page links Basic Rules and Comprehensive Rules. Architecture claim: this project should not replace official rules text and should treat detailed gameplay parity as out of scope unless separately validated.
- MTG Arena formats page: https://magic.wizards.com/en/news/mtg-arena/mtg-arena-formats. Wizards describes Constructed and Limited as primary Arena format categories and notes that Arena events change over time. Architecture claim: keep format/event data externally supplied and current.
- Standard format page: https://magic.wizards.com/en/formats/standard. The page states Standard has a 60-card minimum main deck, up to 15 sideboard cards, and can be best of one or best of three. Implementation claim: validator and Bo1/Bo3-oriented configs can use 60+ and sideboard sizing assumptions.
- MTG Arena Strixhaven state-of-game article: https://magic.wizards.com/en/news/mtg-arena/mtg-arena-state-game-strixhaven-school-mages-2021-04-07. Wizards states Constructed Best-of-One uses a 7-card sideboard while Best-of-Three continues to use 15. Implementation claim: Bo1 config uses 7 accessible sideboard slots and Bo3 uses 15.
- Wizards Fan Content Policy: https://company.wizards.com/en/legal/fancontentpolicy. The policy requires unofficial status, free fan content boundaries, and trademark/logo caution. Product claim: README and compliance notes must state unofficial status and avoid implying endorsement.
- Wizards General Terms: https://company.wizards.com/en/legal/terms. Terms restrict unauthorized data mining, reverse engineering, unauthorized connections, circumvention, and cheats/bots. Compliance claim: no protected API access, binary reverse engineering, gameplay automation, or client control.

## Market and Competitor Sources

- Untapped.gg Companion: https://mtga.untapped.gg/companion?hl=pt. Public page advertises deck tracking, personal stats, in-game overlay, land/draw insights, opponent tracking, Draftsmith, collection stats, collection-aware deck finding, and large match volume. Positioning: users pay for convenience, personal history, collection-aware recommendations, and dynamic draft support.
- AetherHub Premium: https://aetherhub.com/Premium/. Public page lists free base functionality, premium from $3.2/month, ad removal, MTGA Assistant premium status, more settings, deck notes, unlimited decks/folders/collections/binders/favorites, and NRG unlocks. Positioning: users pay to remove limits, remove ads, and manage larger personal data sets.
- MTGGoldfish Premium: https://www.mtggoldfish.com/premium. Public page lists $6/month, SuperBrew, unlimited card tracking, price alerts, collection import/export/backups, and card price history downloads. Positioning: users pay for collection economics, price data, collection-aware deck finding, and exportability.
- Arena Tutor on Overwolf: https://www.overwolf.com/app/draftsim-arena-tutor. Public page lists free app with ads and subscription available, AI-assisted deck tracker/draft assistant, opponent deck tracking, dynamic pick suggestions, match history stats, and limited deck building help. Positioning: users pay for in-client convenience and coaching.
- 8Pack: https://www.8pack.gg/. Public page offers daily draft challenges, shared packs, community pick percentages, history/progress, free daily Standard drafts, and premium formats through subscription/day-pass style extras. Positioning: users pay for practice formats, community comparison, and persistent progress.

## Draft and Statistics Sources

- 17Lands win-rate article: https://blog.17lands.com/posts/using-win-rate-data/. The article distinguishes GP WR, OH WR, GD WR, and GIH WR, explains why drawn-card metrics provide more signal than raw deck win rate, and warns about archetype/context bias. Implementation claim: include GIH/OH/IWD-style metrics, color-pair/archetype context, and sample-size warnings.
- 8Pack public page: https://www.8pack.gg/. The page explains comparable daily packs and first-eight-pick limitations. Implementation claim: include pack/pick context, pick-order, signal/wheel proxy fields, and caution that draft data shape matters.

## Rust Crate Review

- `csv`: kept for streaming CSV ingestion and writer output.
- `serde`, `serde_json`: kept for typed domain structs and JSON/JSONL.
- `serde_yaml`: added only for YAML catalog ingestion; it is deprecated upstream but remains a pragmatic serde-compatible option for this V1. Revisit if YAML becomes a primary production surface.
- `clap`: kept for the Rust CLI.
- `anyhow`, `thiserror`: kept for CLI/application errors and future typed domain errors.
- `tracing`, `tracing-subscriber`: added for structured runtime diagnostics.
- `rand`, `rand_chacha`: added for deterministic seeded simulation; replaces Python-compatible RNG.
- `statrs`: added for statistics primitives, especially confidence interval support.
- `schemars`: added to generate JSON Schema for catalog, API, and LLM artifacts.
- Polars/Arrow/Parquet: not added in V1. They are useful for large analytics storage, but they are heavy for the current CLI foundation.

## Prompt and Agent Practice Sources

- OpenAI Codex Prompting Guide: https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide. The guide emphasizes appropriate reasoning effort, autonomy, compaction, and structured context for coding agents.
- OpenAI AGENTS.md guide: https://developers.openai.com/codex/guides/agents-md. The guide documents `AGENTS.md` discovery and recommends repository-level expectations such as setup and test commands.
- agents.md: https://github.com/agentsmd/agents.md. The project describes AGENTS.md as a simple open format for coding-agent context.
- Claude context guidance: https://support.claude.com/en/articles/14553240-give-claude-context-claude-md-and-better-prompts. The guide recommends lean project context files with commands, conventions, architecture, constraints, and known gotchas.

## Product Positioning

The gap this repo can fill is not another opaque overlay. It can be a transparent Rust engine for user-supplied data, seeded simulation-over-time, reproducible report exports, backend-ready contracts, and structured LLM analysis artifacts. Commercial layers should sell compute, dashboards, team workspaces, creator exports, and API access around original software and user-owned data, not gated Wizards IP.
