# Foundation-002 Assumptions

## Scope

- This milestone adds offline card catalog normalization only.
- It consumes already-downloaded local JSON files.
- It does not make network calls, download bulk data, automate MTG Arena, scrape screens, control the Arena client, or provide live match assistance.

## Supported Inputs

- `scryfall`: a local JSON array of Scryfall-style card records, or an object with a `cards` array.
- `mtgjson`: a local JSON array, an object with `cards`, or an object with `data.cards`.
- MTGJSON support is conservative and maps common local fields used by card/set payloads rather than trying to implement the full MTGJSON schema.

## Output Schema

The normalizer writes deterministic pretty JSON:

```json
{
  "cards": []
}
```

Normalized cards preserve these compact fields when available: `name`, `mana_cost`, `mana_value`, `colors`, `color_identity`, `type_line`, `oracle_text`, `legalities`, `rarity`, `set_code`, `collector_number`, `arena_id`, `digital`, `games`, and `layout`.

## Validation Commands

```bash
make lint
make test
python3 -m mtgdeckbuilder normalize-cards scryfall tests/fixtures/scryfall_cards.json /tmp/normalized_cards.json
make eval-smoke
```

## Next Step

Add optional metadata reporting for normalized catalogs, such as source name, input record count, normalized record count, skipped record count, and a list of missing high-value fields.
