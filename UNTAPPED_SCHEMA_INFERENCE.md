# Lawful Schema Inference Report

Date: 2026-05-16

## Boundary

This report uses only public product pages and generic MTG Arena data concepts. It does not copy Untapped.gg proprietary schemas, code, private APIs, or binary behavior.

## Generic Data Models To Support

- `deck`: deck id, name, format, queue, mainboard card quantities, sideboard card quantities, companion, source path/hash.
- `card`: name, set code, collector number, mana cost, mana value, colors, color identity, type line, rarity, legalities, Arena availability, source path/hash.
- `collection`: card name, quantity, set code, rarity, owned count, wildcard/crafting proxy fields.
- `event`: event id, format, queue, rank/bracket, start/end timestamps, source path/hash.
- `match`: match id, event id, queue, format, opponent archetype label if user supplied, match result, rank context, timestamp.
- `game`: match id, game number, result, play/draw flag if supplied, mulligan count, opening-hand proxy fields, sideboarded flag.
- `draft`: draft id, set code, event type, seat id if user supplied, pack count, pick count, timestamp.
- `pick`: draft id, pack number, pick number, card name, seen-at-pick, taken flag, wheel flag, color-pair/archetype context.
- `report`: schema version, assumptions, source hashes, validation, simulation config, metrics, confidence intervals, sample-size warnings.

## Product Implication

Competitors sell convenience, overlays, collection-aware recommendations, personal history, metagame filtering, draft suggestions, and polished dashboards. This repo can differentiate by making a transparent offline engine that produces reproducible reports, structured exports, and backend-ready contracts without requiring proprietary client access.
