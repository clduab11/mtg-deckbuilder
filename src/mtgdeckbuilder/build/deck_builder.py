"""Conservative deck candidate construction from local normalized datasets."""

from __future__ import annotations

from collections import defaultdict
import json
from pathlib import Path
from typing import Any

from mtgdeckbuilder.analysis.deck_rank import DEFAULT_MIN_GAMES, rank_decks
from mtgdeckbuilder.export.arena import export_arena_deck
from mtgdeckbuilder.ingest.arena import DeckEntry, Decklist
from mtgdeckbuilder.ingest.cards import CardCatalog
from mtgdeckbuilder.rules.validator import validate_deck


def build_deck_file(
    cards_path: str | Path,
    collection_path: str | Path | None,
    results_path: str | Path,
    format_name: str = "standard",
    queue: str = "bo1",
    min_games: int = DEFAULT_MIN_GAMES,
) -> str:
    catalog = CardCatalog.from_json_file(cards_path)
    collection = _load_optional_json(collection_path)
    results = json.loads(Path(results_path).read_text(encoding="utf-8"))
    deck_payload = _select_supported_deck(results, queue=queue, min_games=min_games)
    deck = _decklist_from_payload(deck_payload, collection)
    validation = validate_deck(deck, catalog=catalog)
    return json.dumps(
        {
            "deck_id": deck_payload["deck_id"],
            "deck_name": deck_payload.get("deck_name", ""),
            "queue": queue,
            "validation": {
                "valid": validation.is_valid,
                "issues": [issue.to_dict() for issue in validation.issues],
            },
            "arena_export": export_arena_deck(deck) if validation.is_valid else "",
            "evidence": deck_payload["evidence"],
        },
        indent=2,
        sort_keys=True,
    ) + "\n"


def _select_supported_deck(results: dict[str, Any], queue: str, min_games: int) -> dict[str, Any]:
    claims = [claim for claim in rank_decks(results, min_games=min_games) if claim.queue == queue.casefold()]
    supported = [claim for claim in claims if claim.label == "60_percent_supported"]
    if not supported:
        raise ValueError("no evidence-backed 60_percent_supported deck is available for the requested queue")
    claim = supported[0]
    deck_rows = _deck_rows(results, claim.deck_id)
    if not deck_rows:
        raise ValueError("selected performance record has no deck card rows")
    return {
        "deck_id": claim.deck_id,
        "deck_name": claim.deck_name,
        "rows": deck_rows,
        "evidence": claim.to_dict(),
    }


def _deck_rows(results: dict[str, Any], deck_id: str) -> list[dict[str, Any]]:
    rows = results.get("decks", [])
    if not isinstance(rows, list):
        return []
    return [row for row in rows if isinstance(row, dict) and str(row.get("deck_id")) == deck_id]


def _decklist_from_payload(deck_payload: dict[str, Any], collection: dict[str, int] | None) -> Decklist:
    main: dict[str, int] = defaultdict(int)
    side: dict[str, int] = defaultdict(int)
    for row in deck_payload["rows"]:
        name = str(row.get("card_name") or "")
        quantity = int(row.get("quantity", 0))
        section = str(row.get("section", "mainboard")).casefold()
        if not name or quantity <= 0:
            continue
        if collection is not None and collection.get(name.casefold(), 0) < quantity:
            raise ValueError(f"collection does not contain enough copies of {name}")
        if section in {"sideboard", "side"}:
            side[name] += quantity
        else:
            main[name] += quantity
    return Decklist(
        mainboard=tuple(DeckEntry(count, name) for name, count in main.items()),
        sideboard=tuple(DeckEntry(count, name) for name, count in side.items()),
    )


def _load_optional_json(path: str | Path | None) -> dict[str, int] | None:
    if path is None:
        return None
    payload = json.loads(Path(path).read_text(encoding="utf-8"))
    rows = payload.get("collection", []) if isinstance(payload, dict) else []
    collection: dict[str, int] = {}
    for row in rows:
        if isinstance(row, dict) and row.get("name"):
            collection[str(row["name"]).casefold()] = int(row.get("quantity", 0))
    return collection
