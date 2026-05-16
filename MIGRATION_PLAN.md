# Rust-First Migration Plan

Date: 2026-05-16

## Goal

Move `mtg-deckbuilder` from the earlier Python-first prototype into a Rust-first MTG Arena-oriented deck validation, ingestion, simulation, reporting, and analysis foundation.

## Current Migration

- Replace the Python package entrypoints with a Rust crate and CLI.
- Keep the offline, deterministic core as the primary product surface.
- Preserve lawful reference-only lessons from external prototypes and competitors.
- Avoid gameplay automation, MTG Arena client control, protected API access, binary reverse engineering, and proprietary schema copying.
- Use "Bo1/Bo3-oriented" unless exact parity is separately validated.

## Architecture Target

- `src/main.rs`: thin binary entrypoint.
- `src/cli/`: command routing and CLI output.
- `src/domain/`: public domain types and match config types.
- `src/catalog/`: catalog ingestion, normalization, and schema support.
- `src/ingest/`: existing source-specific Scryfall, MTGJSON, decklist, and collection readers.
- `src/sim/`: seeded simulation primitives plus Bo1/Bo3-oriented config.
- `src/stats/`: win rates, Wilson confidence intervals, constructed metrics, and draft metrics.
- `src/report/`: JSON, Markdown, and CSV report rendering.
- `src/llm/`: structured LLM-ready artifacts only.
- `src/api_contract/`: backend-ready request/response structs and route constants.

## Execution Phases

1. Port the working Rust snapshot into a clean Git clone.
2. Remove Python runtime dependency from the main execution path.
3. Add V1 data ingestion across CSV, JSON, JSONL, and YAML.
4. Add seeded ChaCha RNG for reproducible simulation.
5. Add draft and constructed metric primitives with confidence intervals and sample-size warnings.
6. Add LLM-ready structured reports without allowing LLMs to control deterministic outcomes.
7. Rewrite README and add compliance/research notes.
8. Run format, clippy, tests, release build, CLI smoke tests, `git diff --check`, then commit and push.

## Non-Goals

- No exact MTG Arena simulator parity claim.
- No live tracker, screen scraper, or account integration.
- No proprietary competitor schema or code import.
- No hosted payment, dashboard, or API implementation in this V1.
- No Polars, Arrow, or Parquet dependency in V1; those remain future optional analytics storage.
