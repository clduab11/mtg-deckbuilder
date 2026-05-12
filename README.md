# MTG Deckbuilder

Offline Python foundation for MTG Arena deck intelligence. This milestone parses Arena import text, validates deterministic deck construction rules, extracts basic features, runs a smoke evaluator, and writes Arena-compatible import text.

## Boundaries

This project does not automate MTG Arena gameplay, scrape screens, control mouse or keyboard input, inspect live match state, or provide live match assistance. Outputs are limited to validated decklists, reports, feature summaries, and Arena import text.

## Usage

Run commands from the project root:

```bash
make test
make lint
make eval-smoke
make export-sample
```

Direct CLI usage:

```bash
python3 -m mtgdeckbuilder validate data/raw/sample_arena_deck.txt --cards data/processed/sample_cards.json
python3 -m mtgdeckbuilder export data/raw/sample_arena_deck.txt
python3 -m mtgdeckbuilder eval-smoke data/raw/sample_arena_deck.txt --cards data/processed/sample_cards.json
python3 -m mtgdeckbuilder normalize-cards scryfall tests/fixtures/scryfall_cards.json data/processed/normalized_cards.json
```

The default constructed legality format is `standard`. Override it with `--format`.

## Offline Card Normalization

`normalize-cards` converts already-downloaded local JSON into the compact catalog shape used by validation, feature extraction, and smoke evaluation:

```bash
python3 -m mtgdeckbuilder normalize-cards <source> <input_json> <output_json>
```

Supported sources:

- `scryfall`: local Scryfall bulk JSON as a list, or an object with a `cards` list.
- `mtgjson`: local MTGJSON-style JSON with `data.cards`, `cards`, or a plain card list.

The command writes deterministic pretty JSON with an embedded metadata object and a top-level `cards` list:

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

Normalized records preserve common local fields where available: `name`, `mana_cost`, `mana_value`, `colors`, `color_identity`, `type_line`, `oracle_text`, `legalities`, `rarity`, `set_code`, `collector_number`, `arena_id`, `digital`, `games`, and `layout`.

Malformed card records are skipped and counted in metadata instead of aborting the entire normalization run. Invalid source shapes still fail fast with an error.

## Data

- `data/raw/` stores input decklists.
- `data/processed/` stores normalized card records.
- `data/metagame/` is reserved for offline metagame snapshots.
- `data/cache/` is reserved for downloaded or generated caches.

Card records are JSON objects or arrays compatible with common Scryfall and MTGJSON field names such as `name`, `mana_cost`, `manaCost`, `type_line`, `type`, `colors`, `color_identity`, `colorIdentity`, `mana_value`, `manaValue`, `cmc`, `set`, `set_code`, `setCode`, `collector_number`, `number`, `identifiers`, and `legalities`.
