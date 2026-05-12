# Foundation-011: Catalog-Backed Deck Validation

## Purpose

Foundation-011 adds an opt-in validation mode that resolves parsed Arena decklist card names against a normalized local card catalog. The goal is to keep the original offline heuristic validator intact while allowing users with Scryfall/MTGJSON-derived catalogs to catch unknown names, ambiguous catalog records, and explicit legality failures.

## Scope

- Added `--catalog <normalized_catalog_json>` to `validate`.
- Added `--catalog <normalized_catalog_json>` to `eval-smoke`.
- Preserved `--cards` as a compatibility alias.
- Preserved no-catalog validation behavior.
- Added catalog diagnostics:
  - `unknown_card`
  - `ambiguous_card`
  - `missing_legality_data`
  - `illegal_in_format`
  - `banned_in_format`

## Behavior

When no catalog is supplied, validation remains a deterministic construction check: mainboard size, sideboard size, copy limits, fallback basic land names, and Arena export compatibility.

When a catalog is supplied, each unique deck entry is resolved by normalized card name. If the Arena entry includes set code and collector number, those fields are used to disambiguate catalog matches. Multiple remaining matches are treated as ambiguous because the validator cannot choose a printing safely.

Legality checks use the requested format, defaulting to `standard`. Missing legality data is reported as a warning so incomplete local catalogs remain usable. Explicit `not_legal`, `illegal`, or `banned` values are errors.

`eval-smoke --catalog` stops after catalog-backed validation failures and omits feature/export output for invalid decks. This avoids producing downstream summaries from unresolved or explicitly illegal catalog input.

## Assumptions

- The catalog is a local normalized JSON file with a top-level `cards` list or a plain card-record list.
- Card truth still comes from local Scryfall/MTGJSON/official-source data, not LLM memory.
- Duplicate names are allowed in catalogs, but deck entries must include enough set/collector metadata to resolve them when duplicates exist.
- No network calls, Arena automation, gameplay automation, screen scraping, or live match assistance are involved.

## Validation

Required commands:

```bash
make lint
make test
make eval-smoke
```

Additional manual checks:

```bash
python3 -m mtgdeckbuilder validate data/raw/sample_arena_deck.txt --catalog data/processed/sample_cards.json
python3 -m mtgdeckbuilder eval-smoke data/raw/sample_arena_deck.txt --catalog data/processed/sample_cards.json
```
