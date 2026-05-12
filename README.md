# mtg-deckbuilder

Offline, AI-ready Magic: The Gathering Arena deck intelligence.

`mtg-deckbuilder` is the working title for a personal MTG Arena deckbuilding assistant. It is designed to turn your local decklists, card catalogs, collection exports, metagame snapshots, and match-result CSVs into validated decklists, performance reports, feature summaries, and Arena import text. The long-term goal is simple: make it much easier to discover strong, legal, evidence-backed deck combinations with the help of AI and machine learning, without automating gameplay or trusting unsupported guesses.

This repository is the v0.1 foundation. It focuses on the trusted data layer that AI-assisted deckbuilding needs before any serious recommendation engine can be useful: clean cards, clear legality, normalized results, reproducible diagnostics, and conservative deck candidate generation from local evidence.

## Who This Is For

This project is for MTG Arena players who want a private, offline-first deck intelligence workflow:

- Competitive ladder players who want to evaluate BO1 and BO3 performance from their own exported results.
- Brewers who want to test whether a deck idea is legal, exportable, and supported by a local card catalog.
- Players managing collections who want candidate decklists constrained by cards they own.
- Data-minded players who track results in spreadsheets or tracker exports.
- Open-source contributors who want an AGPL-licensed foundation for Arena deck analytics, CSV adapters, and future AI/ML deck recommendation work.

It is not a gameplay bot, overlay, live match assistant, tracker, or Arena automation tool.

## Why AI And ML Matter Here

Good deckbuilding is a search problem. You are balancing legality, format, color requirements, mana curve, card roles, sideboard plans, metagame pressure, personal collection limits, matchup data, and observed win rates. Doing that manually across spreadsheets, deck sites, and tracker exports is slow.

`mtg-deckbuilder` is meant to make that work feel almost effortless by preparing the data an AI or ML layer would need:

- Normalize card truth from Scryfall or MTGJSON instead of relying on memory.
- Normalize performance data from user-provided CSVs.
- Separate BO1 and BO3 evidence so results are not mixed accidentally.
- Rank decks with sample-size guardrails.
- Validate generated lists before they ever become Arena import text.
- Preserve provenance so a future model can explain why a candidate was suggested.

The repo should eventually support AI-assisted recommendations such as “show me the strongest legal BO1 white aggro candidates from my collection” or “find decks with at least 60% observed win rate in my local data.” Those recommendations must remain evidence-labeled. A deck is not called a “winning deck” just because a model says so; it needs local performance data, enough games, and valid card legality.

## Safety Boundary

This project is for deck intelligence only.

It does not:

- automate MTG Arena gameplay
- inspect live matches
- scrape screens
- control the mouse or keyboard
- drive the Arena client
- bypass paywalls or source terms
- provide live match assistance

Inputs are local files supplied by the user. Outputs are validated decklists, diagnostics, reports, feature summaries, and Arena import text.

## Current Status

Status: early v0.1 foundation, offline-first, deterministic, and source-conscious.

The current implementation has no runtime dependencies and no advanced neural model. It builds the local data and validation substrate needed for later AI/ML work.

## What It Can Do Today

- Parse MTG Arena import text with mainboard and sideboard sections.
- Export canonical Arena import text.
- Validate deck construction rules:
  - minimum mainboard size
  - sideboard size
  - non-basic copy limits
  - basic land copy exception
  - Arena export compatibility
- Validate decklists against an optional normalized local card catalog:
  - `unknown_card`
  - `ambiguous_card`
  - `missing_legality_data`
  - `illegal_in_format`
  - `banned_in_format`
- Extract basic deck features:
  - colors
  - mana curve
  - card type counts
  - unknown cards
- Normalize already-downloaded Scryfall and MTGJSON-style card data.
- Embed normalization metadata and diagnostics in normalized card catalogs.
- Print human-readable catalog normalization reports.
- Profile and normalize user-provided CSV exports from Arena-adjacent source shapes.
- Rank decks from local BO1/BO3 performance CSVs with sample-size guardrails.
- Build conservative Arena-ready deck candidates from validated local evidence.
- Run a smoke evaluator that parses, validates, featureizes, and exports a deck.

## Quick Start

Clone the repo and run the test suite:

```bash
git clone https://github.com/clduab11/mtg-deckbuilder.git
cd mtg-deckbuilder
python3 -m mtgdeckbuilder --help
make test
```

Optional editable install:

```bash
python3 -m pip install -e .
mtgdeckbuilder --help
```

## Core Workflow

Validate an Arena decklist against a local catalog:

```bash
python3 -m mtgdeckbuilder validate \
  data/raw/sample_arena_deck.txt \
  --catalog data/processed/sample_cards.json
```

Export canonical Arena import text:

```bash
python3 -m mtgdeckbuilder export data/raw/sample_arena_deck.txt
```

Run the smoke evaluator:

```bash
python3 -m mtgdeckbuilder eval-smoke \
  data/raw/sample_arena_deck.txt \
  --catalog data/processed/sample_cards.json
```

Normalize already-downloaded Scryfall bulk-style data:

```bash
python3 -m mtgdeckbuilder normalize-cards \
  scryfall \
  tests/fixtures/scryfall_cards.json \
  data/processed/normalized_cards.json
```

Normalize already-downloaded MTGJSON-style data:

```bash
python3 -m mtgdeckbuilder normalize-cards \
  mtgjson \
  tests/fixtures/mtgjson_cards.json \
  data/processed/normalized_cards.json
```

Print catalog normalization diagnostics:

```bash
python3 -m mtgdeckbuilder normalize-report data/processed/normalized_cards.json
```

Inspect supported source profiles:

```bash
python3 -m mtgdeckbuilder source-profile list
python3 -m mtgdeckbuilder source-profile inspect untapped_like_csv
```

Profile and normalize local CSV exports:

```bash
python3 -m mtgdeckbuilder csv-profile tests/fixtures/csv/untapped_like_results.csv
python3 -m mtgdeckbuilder csv-normalize \
  untapped_like_csv \
  tests/fixtures/csv/untapped_like_results.csv \
  data/processed/results.json
python3 -m mtgdeckbuilder csv-report data/processed/results.json
```

Rank decks from local performance data:

```bash
python3 -m mtgdeckbuilder deck-rank data/processed/results.json --min-games 30
```

Build a conservative deck candidate from local card, collection, deck, and performance data:

```bash
python3 -m mtgdeckbuilder deck-build \
  --cards data/processed/sample_cards.json \
  --results data/processed/results.json \
  --collection data/processed/collection.json \
  --format standard \
  --queue bo1
```

The default constructed legality format is `standard`. Override it with `--format`. The older `--cards` option remains accepted as a compatibility alias for catalog input on validation commands.

## Data Sources And Compatibility

The project treats external sites as compatibility targets, not data dependencies. Users provide local exports and remain responsible for each source's terms.

Current source profile targets:

- `untapped_like_csv`: Untapped.gg-style tracker/stat exports with wins, losses, rank scope, queue, and BO1/BO3 segmentation.
- `aetherhub_like_deck`: AetherHub-style Arena deck rows with card counts and deck sections.
- `mtggoldfish_like_metagame`: MTGGoldfish-style metagame rows with archetype and metagame share.
- `generic_card_csv`: card catalog spreadsheets.
- `generic_collection_csv`: owned-card collection exports.
- `generic_deck_csv`: generic deck rows.
- `generic_match_results_csv`: generic performance rows.

Future adapters can cover MTGDecks, MTGAZone, MTG Arena Pro, 17Lands, Moxfield, Archidekt, and CSV fixtures from GitHub, GitLab, or Bitbucket. Tests should stay fixture-only and network-free.

## Data Model

Normalized catalogs are deterministic JSON documents with a top-level `metadata` object and a top-level `cards` list. Existing catalog consumers read the `cards` list directly.

```json
{
  "metadata": {
    "schema_version": "foundation-003.v1",
    "generated_at": "2026-05-12T00:00:00Z",
    "source": "scryfall",
    "input_path": "tests/fixtures/scryfall_cards.json",
    "output_path": "data/processed/normalized_cards.json",
    "input_count": 2,
    "normalized_count": 2,
    "skipped_count": 0,
    "skipped_reasons": {},
    "missing_high_value_fields_count": 0,
    "missing_high_value_fields_by_field_name": {}
  },
  "cards": []
}
```

Normalized card records preserve common local fields where available:

- `name`
- `mana_cost`
- `mana_value`
- `colors`
- `color_identity`
- `type_line`
- `oracle_text`
- `legalities`
- `rarity`
- `set_code`
- `collector_number`
- `arena_id`
- `digital`
- `games`
- `layout`

Malformed card records inside an otherwise valid payload are skipped and counted in metadata. Invalid source shapes fail fast.

## Validation Philosophy

Validation is deterministic and local. The project does not infer current card legality, oracle text, bans, restrictions, or metagame truth from LLM memory. Those facts must come from already-downloaded Scryfall, MTGJSON, Wizards of the Coast, or other official/current sources supplied by the user.

Without `--catalog`, validation stays in heuristic deck-construction mode: it checks counts, sideboard size, basic-land copy exceptions by known Arena names, and export compatibility. With `--catalog`, each parsed Arena card name is resolved against the normalized local catalog. Missing legality is a warning because the catalog may be incomplete; explicit illegal or banned legality is an error.

Performance claims are also evidence-bound. A deck is not labeled as a “60% BO1” deck unless local performance data includes enough games for the configured threshold. Card-only CSVs can support legal deck construction, but cannot support winning-deck claims.

## Architecture

The code is organized as a Python `src/` package:

- `mtgdeckbuilder.ingest`: Arena deck parser, card models, and catalog normalization.
- `mtgdeckbuilder.sources`: source profiles plus CSV profiling and normalization.
- `mtgdeckbuilder.analysis`: offline deck performance aggregation.
- `mtgdeckbuilder.build`: conservative evidence-backed deck candidate construction.
- `mtgdeckbuilder.rules`: deterministic deck validator.
- `mtgdeckbuilder.features`: basic offline feature extraction.
- `mtgdeckbuilder.export`: Arena import text writer.
- `mtgdeckbuilder.eval`: smoke evaluator.
- `mtgdeckbuilder.observability`: lightweight logging helpers.

The top-level `mtgdeckbuilder/` package is a small repo-root shim so `python3 -m mtgdeckbuilder` works from an uninstalled checkout.

## Repository Layout

- `data/raw/`: input decklists and raw local files.
- `data/processed/`: normalized local catalogs and processed sample data.
- `data/metagame/`: reserved for offline metagame snapshots.
- `data/cache/`: ignored cache area for generated or downloaded local artifacts.
- `reports/experiments/`: milestone assumptions, validation notes, and implementation logs.
- `tests/`: deterministic fixture-only tests.

## Roadmap

- Choose a public project name and branding beyond the current working title.
- Expand offline Scryfall/MTGJSON bulk-data workflow documentation.
- Add broader source profile fixtures for MTGDecks, MTGAZone, MTG Arena Pro, 17Lands, Moxfield, Archidekt, GitHub, GitLab, and Bitbucket-hosted CSV examples.
- Add richer deck feature extraction, including role tags, curve pressure, interaction density, and sideboard coverage.
- Add richer format-aware legality and restriction profiles from local card catalogs.
- Add matchup and metagame summaries from offline snapshots.
- Add deterministic report generation for deck comparison.
- Add AI/ML-assisted candidate scoring once the local evidence contracts are mature enough to support it responsibly.

## License

`mtg-deckbuilder` is licensed under the GNU Affero General Public License v3.0 only (`AGPL-3.0-only`). See `LICENSE`.

Magic: The Gathering and Magic: The Gathering Arena are trademarks of Wizards of the Coast. This project is unofficial and is not affiliated with or endorsed by Wizards of the Coast.
