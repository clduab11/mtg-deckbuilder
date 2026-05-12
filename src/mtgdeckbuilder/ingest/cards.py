"""Card schema models and offline card catalog loading."""

from __future__ import annotations

from dataclasses import dataclass, field
import json
from pathlib import Path
from typing import Any, Mapping


BASIC_LAND_NAMES = frozenset({"plains", "island", "swamp", "mountain", "forest", "wastes"})


def normalize_card_name(name: str) -> str:
    return " ".join(name.casefold().split())


def is_basic_land_name(name: str) -> bool:
    return normalize_card_name(name) in BASIC_LAND_NAMES


@dataclass(frozen=True)
class Card:
    """Minimal Scryfall/MTGJSON-ready card record."""

    name: str
    type_line: str = ""
    colors: tuple[str, ...] = ()
    color_identity: tuple[str, ...] = ()
    mana_value: float = 0.0
    mana_cost: str = ""
    oracle_text: str = ""
    rarity: str = ""
    digital: bool = False
    games: tuple[str, ...] = ()
    layout: str = ""
    legalities: Mapping[str, str] = field(default_factory=dict)
    set_code: str | None = None
    collector_number: str | None = None
    scryfall_id: str | None = None
    arena_id: int | None = None

    @property
    def normalized_name(self) -> str:
        return normalize_card_name(self.name)

    @property
    def is_basic_land(self) -> bool:
        type_tokens = self.type_line.casefold().replace("-", " ").split()
        return ("basic" in type_tokens and "land" in type_tokens) or is_basic_land_name(self.name)

    @classmethod
    def from_mapping(cls, data: Mapping[str, Any]) -> "Card":
        mana_value = data.get("mana_value", data.get("manaValue", data.get("cmc", 0.0)))
        set_code = data.get("set_code", data.get("setCode", data.get("set")))
        scryfall_id = data.get("scryfall_id", data.get("id"))
        arena_id = data.get("arena_id", data.get("arenaId"))
        collector_number = data.get("collector_number", data.get("number"))
        return cls(
            name=str(data["name"]),
            type_line=str(data.get("type_line", data.get("typeLine", data.get("type", "")))),
            colors=_string_tuple(data.get("colors", ())),
            color_identity=_string_tuple(data.get("color_identity", data.get("colorIdentity", ()))),
            mana_value=float(mana_value or 0.0),
            mana_cost=str(data.get("mana_cost", data.get("manaCost", ""))),
            oracle_text=str(data.get("oracle_text", data.get("oracleText", data.get("text", "")))),
            rarity=str(data.get("rarity", "")),
            digital=_bool_value(data.get("digital", False)),
            games=_string_tuple(data.get("games", ())),
            layout=str(data.get("layout", "")),
            legalities=dict(data.get("legalities") or {}),
            set_code=str(set_code).upper() if set_code else None,
            collector_number=str(collector_number) if collector_number else None,
            scryfall_id=str(scryfall_id) if scryfall_id else None,
            arena_id=int(arena_id) if arena_id is not None else None,
        )


class CardCatalog:
    """Case-insensitive card lookup for offline validation and features."""

    def __init__(self, cards: list[Card]) -> None:
        cards_by_name: dict[str, list[Card]] = {}
        for card in cards:
            cards_by_name.setdefault(card.normalized_name, []).append(card)
        self._cards_by_name = {
            normalized_name: tuple(named_cards)
            for normalized_name, named_cards in cards_by_name.items()
        }

    @classmethod
    def from_records(cls, records: list[Mapping[str, Any]]) -> "CardCatalog":
        return cls([Card.from_mapping(record) for record in records])

    @classmethod
    def from_json_file(cls, path: str | Path) -> "CardCatalog":
        with Path(path).open("r", encoding="utf-8") as handle:
            payload = json.load(handle)
        if isinstance(payload, dict):
            records = payload.get("cards", [])
        else:
            records = payload
        if not isinstance(records, list):
            raise ValueError("card JSON must be a list or an object with a 'cards' list")
        return cls.from_records(records)

    def get(self, name: str) -> Card | None:
        matches = self.matches(name)
        if not matches:
            return None
        return matches[-1]

    def matches(self, name: str) -> tuple[Card, ...]:
        return self._cards_by_name.get(normalize_card_name(name), ())

    def resolve(
        self,
        name: str,
        set_code: str | None = None,
        collector_number: str | None = None,
    ) -> tuple[Card, ...]:
        matches = self.matches(name)
        if set_code is not None:
            normalized_set = set_code.upper()
            matches = tuple(card for card in matches if card.set_code == normalized_set)
        if collector_number is not None:
            normalized_collector = collector_number.strip()
            matches = tuple(
                card for card in matches if card.collector_number == normalized_collector
            )
        return matches

    def __contains__(self, name: object) -> bool:
        return isinstance(name, str) and self.get(name) is not None


def _string_tuple(value: Any) -> tuple[str, ...]:
    if value is None:
        return ()
    if isinstance(value, str):
        return (value,)
    return tuple(str(item) for item in value)


def _bool_value(value: Any) -> bool:
    if isinstance(value, str):
        return value.casefold() in {"1", "true", "yes"}
    return bool(value)
