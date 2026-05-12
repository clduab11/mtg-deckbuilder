"""Basic deterministic deck feature extraction."""

from __future__ import annotations

from collections import Counter
from typing import Any

from mtgdeckbuilder.ingest.arena import Decklist
from mtgdeckbuilder.ingest.cards import Card, CardCatalog


CARD_TYPE_ORDER = (
    "Creature",
    "Instant",
    "Sorcery",
    "Artifact",
    "Enchantment",
    "Planeswalker",
    "Battle",
    "Land",
)


def extract_basic_features(deck: Decklist, catalog: CardCatalog | None = None) -> dict[str, Any]:
    """Extract simple offline features from the mainboard."""

    colors: set[str] = set()
    unknown_cards: set[str] = set()
    mana_curve: Counter[str] = Counter()
    type_counts: Counter[str] = Counter()
    unique_cards: set[str] = set()

    for entry in deck.mainboard:
        unique_cards.add(entry.name)
        card = catalog.get(entry.name) if catalog is not None else None
        if card is None:
            unknown_cards.add(entry.name)
            continue
        colors.update(card.color_identity or card.colors)
        mana_curve[_mana_bucket(card)] += entry.count
        for type_name in _card_types(card):
            type_counts[type_name] += entry.count

    return {
        "mainboard_count": deck.mainboard_count,
        "sideboard_count": deck.sideboard_count,
        "unique_mainboard_cards": len(unique_cards),
        "colors": sorted(colors),
        "mana_curve": dict(sorted(mana_curve.items(), key=lambda item: _bucket_sort_key(item[0]))),
        "type_counts": {key: type_counts[key] for key in CARD_TYPE_ORDER if type_counts[key]},
        "unknown_cards": sorted(unknown_cards),
    }


def _mana_bucket(card: Card) -> str:
    value = int(card.mana_value)
    return "7+" if value >= 7 else str(value)


def _bucket_sort_key(bucket: str) -> int:
    return 7 if bucket == "7+" else int(bucket)


def _card_types(card: Card) -> tuple[str, ...]:
    found = [type_name for type_name in CARD_TYPE_ORDER if type_name in card.type_line]
    return tuple(found)
