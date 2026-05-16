# mtg-deckbuilder

## What This Is

`mtg-deckbuilder` is a Rust-first, offline MTG Arena-oriented deck validation, ingestion, simulation, statistics, and reporting engine. It works from user-supplied card catalogs, decklists, collection files, and result records.

It is not an MTG Arena client, overlay, tracker, bot, or exact gameplay simulator.

## Why It Exists

Most MTG deck tools sell convenience around overlays, collection sync, pricing, draft helpers, metagame dashboards, or personal history. This project is aimed at a different layer: a deterministic, auditable Rust engine that can validate inputs, run seeded Bo1/Bo3-oriented simulations, calculate transparent metrics, and export structured artifacts for future dashboards, APIs, and LLM-assisted analysis.

## Current Capabilities

- Arena-style decklist parsing and export.
- Collection CSV parsing with conservative ownership detection.
- Local card catalog loading from Scryfall-like JSON, MTGJSON-like JSON, generic CSV, generic JSON, JSONL, and YAML.
- Local `result-log.v1` loading from user-supplied CSV, JSON, and JSONL match/draft records.
- Format-aware validation hooks for copy limits, sideboards, ownership, wildcards, bans, legalities, and Arena compatibility.
- Seeded opening-hand and first-three-turn simulation proxies.
- Bo1/Bo3-oriented simulation configs with explicit assumptions.
- Constructed and draft metric primitives, including Wilson confidence intervals and sample-size warnings.
- JSON, Markdown, and CSV report rendering.
- `llm_report.v1` structured artifact generation.
- Backend-ready API contract structs and route constants.
- Local-only Axum adapter for deterministic Rust services.

## Architecture

```text
src/
  main.rs              thin CLI entrypoint
  cli/                 command routing and output
  domain/              public domain structs
  catalog/             CSV/JSON/JSONL/YAML catalog ingestion and schemas
  ingest/              source-specific deck, collection, Scryfall, MTGJSON loaders
  result_log/          user-owned local match and draft result-log ingestion
  rules/               validation rules
  features/            deck feature extraction
  sim/                 seeded simulation primitives plus bo1/bo3 configs
  stats/               constructed and draft metric primitives
  report/              JSON, Markdown, CSV report rendering
  llm/                 structured LLM-ready artifacts only
  api_contract/        future web/API request and response structs
  web.rs               local-only Axum adapter
```

## Data Formats

Supported catalog inputs:

- CSV: generic card rows and Steam/Arena-style aliases such as `Quantity,Name,Set,Type,Mana Cost,CMC,Colors,Rarity`.
- JSON: Scryfall-like, MTGJSON-like, or `catalog.v1`.
- JSONL: one `catalog.v1` card record per line.
- YAML: `catalog.v1` document.

Examples live in `examples/sample_catalog.csv`, `examples/sample_catalog.json`, `examples/sample_catalog.jsonl`, and `examples/sample_catalog.yaml`.

Supported result-log inputs:

- CSV: typed rows with `record_type` set to `game` or `draft_pick`.
- JSON: `result-log.v1` document with `games` and `draft_picks`.
- JSONL: one typed `game` or `draft_pick` record per line.

Result logs are user-supplied local files only. They are not imported from live trackers, hosted APIs, MTG Arena clients, or external scrapers.

Optional future analytics storage: Arrow/Parquet. These are not dependencies in V1 because the current surface is a CLI/library foundation, not a large analytics warehouse.

## Quick Start

```bash
cargo build
cargo test --all-features
cargo run --bin mtgdeckbuilder -- --help
```

Validate the fixture deck:

```bash
cargo run --bin mtgdeckbuilder -- validate \
  --deck examples/sample_deck.txt \
  --cards tests/fixtures/cards_scryfall.json \
  --collection tests/fixtures/collection.csv \
  --format standard
```

## CLI Usage

```bash
cargo run --bin mtgdeckbuilder -- import-catalog \
  --input examples/sample_catalog.csv

cargo run --bin mtgdeckbuilder -- import-result-log \
  --input tests/fixtures/result_logs.csv

cargo run --bin mtgdeckbuilder -- simulate \
  --deck examples/sample_deck.txt \
  --cards tests/fixtures/cards_scryfall.json \
  --queue bo1 \
  --trials 500 \
  --seed 42

cargo run --bin mtgdeckbuilder -- report \
  --deck examples/sample_deck.txt \
  --cards tests/fixtures/cards_scryfall.json \
  --collection tests/fixtures/collection.csv \
  --format standard \
  --output markdown \
  --result-log tests/fixtures/result_logs.json

cargo run --bin mtgdeckbuilder -- schema --name catalog
cargo run --bin mtgdeckbuilder -- schema --name result-log

cargo run --bin mtgdeckbuilder -- llm-artifact \
  --deck examples/sample_deck.txt \
  --cards tests/fixtures/cards_scryfall.json \
  --format standard \
  --trials 100 \
  --seed 42
```

## Simulation Model

The simulator is Bo1/Bo3-oriented, not exact MTG Arena parity.

- Bo1 uses an Arena-like opening-hand approximation and a 7-card accessible sideboard assumption.
- Bo3 uses paper-random opening sampling and a 15-card sideboard assumption.
- Both modes use seeded `rand_chacha::ChaCha20Rng` reproducibility.
- Current outputs are opening-hand and early-turn quality proxies, not gameplay resolution or match win-rate truth.

## Constructed Metrics

The stats module supports:

- overall game win rate
- match win rate
- Bo1 performance
- Bo3 game performance
- sideboard impact
- mulligan sensitivity
- play/draw performance from user-supplied result logs
- opening-hand quality proxy via simulation reports
- matchup matrix
- Wilson confidence intervals
- sample-size warnings
- seeded reproducibility fields

## Draft Metrics

The draft metric primitives support:

- card win rate
- game-in-hand win rate
- opening-hand win rate
- improvement-when-drawn style delta
- average last seen at and average taken at equivalents
- pick order score
- color-pair and archetype fields
- trophy rate
- wheel rate / signal proxy
- pack and pick context
- sample-size reliability flags

## LLM Integration

LLM support is deliberately outside the deterministic core. The CLI can emit `llm_report.v1`, a structured JSON artifact containing validation, metrics, assumptions, source hashes, limitations, and prompt guidance. LLMs may explain or summarize that evidence, but they must not change validation, simulation, or metric outcomes.

## Web/API Readiness

The repo includes backend-ready structs and route constants for:

- `POST /deck/validate`
- `POST /simulation/run`
- `GET /simulation/{id}/status`
- `GET /simulation/{id}/results`
- `GET /simulation/{id}/report`
- `POST /export`

The local Axum adapter is a development convenience. This repo does not currently ship a hosted API or dashboard.

## Monetization / Product Direction

Research notes in `RESEARCH_NOTES.md` compare public positioning from Untapped.gg, AetherHub, MTGGoldfish, Arena Tutor/Draftsim, and 8Pack. Users pay for overlays, personal history, collection-aware recommendations, ad-free or unlimited storage, draft suggestions, pricing data, community comparisons, and dashboards.

This repo can differentiate through:

- free open-source CLI and deterministic engine
- hosted paid simulation runs
- pro dashboard subscription over user-owned data
- draft-analysis premium tier
- team testing workspace
- API access for creators and data users
- report exports for content creators

Commercial layers must respect Wizards IP, fan content, trademark, and terms boundaries.

## Verification

Expected validation commands:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
git diff --check
```

Smoke-test the main CLI surfaces:

```bash
cargo run --bin mtgdeckbuilder -- validate --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --collection tests/fixtures/collection.csv --format standard
cargo run --bin mtgdeckbuilder -- import-catalog --input examples/sample_catalog.csv
cargo run --bin mtgdeckbuilder -- simulate --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --queue bo1 --trials 25 --seed 7
cargo run --bin mtgdeckbuilder -- export --deck examples/sample_deck.txt
cargo run --bin mtgdeckbuilder -- report --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --collection tests/fixtures/collection.csv --format standard --output markdown --trials 25 --seed 7
cargo run --bin mtgdeckbuilder -- schema --name catalog
cargo run --bin mtgdeckbuilder -- llm-artifact --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --format standard --trials 25 --seed 7
```

## Limitations

- Fixture card data is not authoritative.
- Current legality, bans, restrictions, oracle text, Arena availability, and metagame state must be supplied from current trusted data.
- The Arena-like Bo1 smoother is an approximation because exact MTG Arena behavior is not public.
- Current simulation does not resolve full games, full match play, hidden information, combat, stack choices, or player decisions.
- No gameplay automation, screen scraping, protected API access, or Arena client control is included.
- No proprietary competitor schemas or code are copied.

## Roadmap

- Expand matchup matrix and sideboard impact reports from user-owned result data.
- Add archetype clustering from transparent feature vectors.
- Add optional Arrow/Parquet export behind a deliberate analytics feature gate.
- Add hosted-job adapters around the existing `api_contract` structs.
- Add web dashboard only after the CLI/library contracts stabilize.

## License / Disclaimer

This repository is licensed under the AGPL-3.0 license in `LICENSE`.

`mtg-deckbuilder` is unofficial Fan Content. It is not affiliated with, endorsed by, sponsored by, or approved by Wizards of the Coast, Hasbro, Magic: The Gathering, or MTG Arena. Portions of the materials referenced by users may be property of Wizards of the Coast LLC. Users are responsible for ensuring that their data sources and use comply with applicable law, Wizards policies, MTG Arena terms, and third-party terms.
