# mtg-deckbuilder

`mtg-deckbuilder` is an offline-first Python foundation for Magic: The Gathering Arena deck intelligence. It turns local decklists and already-downloaded card data into validated deck records, deterministic reports, basic feature summaries, and Arena import text without touching the Arena client.

## Safety Boundary

This project is for deck intelligence only. It does not automate MTG Arena gameplay, inspect live matches, scrape screens, control the mouse or keyboard, drive the Arena client, or provide live match assistance.

## Status

Early foundation. The current implementation is intentionally small, deterministic, and offline-first.

## Current Capabilities

- Parse MTG Arena import text with mainboard and sideboard sections.
- Export canonical Arena import text.
- Validate deterministic deck construction rules:
  - minimum mainboard size
  - sideboard size
  - non-basic copy limits
  - basic land copy exception
  - banned legality when local card metadata provides it
  - Arena export round-trip compatibility
- Extract basic deck features:
  - colors
  - mana curve
  - card type counts
  - unknown cards
- Normalize already-downloaded Scryfall and MTGJSON-style card data into the compact local catalog shape.
- Embed normalization metadata and diagnostics in normalized catalogs.
- Print human-readable normalization diagnostics from embedded metadata.
- Profile and normalize user-provided CSV exports from Arena-adjacent source shapes.
- Rank decks from local BO1/BO3 performance CSVs with sample-size guardrails.
- Build conservative Arena-ready deck candidates only from validated local evidence.
- Run a smoke evaluator that parses, validates, featureizes, and exports a deck.

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

## Install / Setup

No runtime dependencies are required for the current foundation.

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

## CLI Usage

Validate an Arena decklist:

```bash
python3 -m mtgdeckbuilder validate data/raw/sample_arena_deck.txt --cards data/processed/sample_cards.json
```

Export canonical Arena import text:

```bash
python3 -m mtgdeckbuilder export data/raw/sample_arena_deck.txt
```

Run the smoke evaluator:

```bash
python3 -m mtgdeckbuilder eval-smoke data/raw/sample_arena_deck.txt --cards data/processed/sample_cards.json
```

Normalize already-downloaded Scryfall bulk-style data:

```bash
python3 -m mtgdeckbuilder normalize-cards scryfall tests/fixtures/scryfall_cards.json data/processed/normalized_cards.json
```

Normalize already-downloaded MTGJSON-style data:

```bash
python3 -m mtgdeckbuilder normalize-cards mtgjson tests/fixtures/mtgjson_cards.json data/processed/normalized_cards.json
```

Print a human-readable diagnostics report from a normalized catalog:

```bash
python3 -m mtgdeckbuilder normalize-report data/processed/normalized_cards.json
```

Legacy compact catalogs without embedded Foundation-003 metadata fail clearly:

```bash
python3 -m mtgdeckbuilder normalize-report data/processed/sample_cards.json
```

The default constructed legality format is `standard`. Override it with `--format`.

Inspect supported source profiles:

```bash
python3 -m mtgdeckbuilder source-profile list
python3 -m mtgdeckbuilder source-profile inspect untapped_like_csv
```

Profile and normalize local CSV exports:

```bash
python3 -m mtgdeckbuilder csv-profile tests/fixtures/csv/untapped_like_results.csv
python3 -m mtgdeckbuilder csv-normalize untapped_like_csv tests/fixtures/csv/untapped_like_results.csv data/processed/results.json
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

## Data Model And Catalog Normalization

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

## Source Profiles And CSV Compatibility

The CSV layer uses declarative source profiles. It is designed for user-provided local files, not scraping or automated downloads.

Current compatibility targets:

- `untapped_like_csv`: tracker/stat exports with wins, losses, rank scope, queue, and BO1/BO3 segmentation.
- `aetherhub_like_deck`: Arena deck/metagame deck rows with card counts and sections.
- `mtggoldfish_like_metagame`: tournament/metagame-style rows with archetype and metagame share.
- `generic_card_csv`: card catalog spreadsheets.
- `generic_collection_csv`: owned-card collection exports.
- `generic_deck_csv`: generic deck rows.
- `generic_match_results_csv`: generic performance rows.

External sites are compatibility targets only. Users must provide exports themselves and remain responsible for each source's terms. Tests use local fixtures only.

## Validation Philosophy

Validation is deterministic and local. The project does not infer current card legality, oracle text, bans, restrictions, or metagame truth from LLM memory. Those facts must come from already-downloaded Scryfall, MTGJSON, Wizards of the Coast, or other official/current sources supplied by the user.

Performance claims are also evidence-bound. A deck is not labeled as a “60% BO1” deck unless local performance data includes enough games for the configured threshold. Card-only CSVs can support legal deck construction, but cannot support winning-deck claims.

## Data Directories

- `data/raw/`: input decklists and raw local files.
- `data/processed/`: normalized local catalogs and processed sample data.
- `data/metagame/`: reserved for offline metagame snapshots.
- `data/cache/`: ignored cache area for generated or downloaded local artifacts.

## Experiment Logs

Milestone assumptions and validation notes live in `reports/experiments/`.

## Roadmap

- Offline Scryfall/MTGJSON bulk-data workflow documentation.
- Broader source profile fixtures for MTGDecks, MTGAZone, MTG Arena Pro, 17Lands, Moxfield, Archidekt, GitHub, GitLab, and Bitbucket-hosted CSV examples.
- Richer deck feature extraction.
- Format-aware legality profiles from local card catalogs.
- Matchup and metagame summaries from offline snapshots.
- Deterministic report generation for deck comparison.

## License

`mtg-deckbuilder` is licensed under the GNU Affero General Public License v3.0 only (`AGPL-3.0-only`). See `LICENSE`.
