"""Offline card catalog normalization for supported upstream JSON shapes."""

from __future__ import annotations

import json
from collections import Counter
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Mapping


NormalizedCard = dict[str, Any]
SCHEMA_VERSION = "foundation-003.v1"
HIGH_VALUE_FIELDS = (
    "mana_cost",
    "mana_value",
    "colors",
    "color_identity",
    "type_line",
    "oracle_text",
    "legalities",
    "rarity",
    "set_code",
    "collector_number",
    "arena_id",
    "digital",
    "games",
    "layout",
)


def normalize_cards_file(source: str, input_json: str | Path, output_json: str | Path) -> int:
    """Normalize a source catalog file and return the number of cards written."""

    input_path = Path(input_json)
    output_path = Path(output_json)
    with Path(input_json).open("r", encoding="utf-8") as handle:
        payload = json.load(handle)
    catalog = normalize_payload(
        source,
        payload,
        input_path=str(input_path),
        output_path=str(output_path),
    )
    output_path.write_text(
        json.dumps(catalog, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return int(catalog["metadata"]["normalized_count"])


def normalization_report_file(catalog_json: str | Path) -> str:
    """Read a normalized catalog and format its embedded metadata report."""

    with Path(catalog_json).open("r", encoding="utf-8") as handle:
        payload = json.load(handle)
    metadata = _catalog_metadata(payload)
    return format_normalization_report(metadata)


def normalize_payload(
    source: str,
    payload: Any,
    input_path: str | None = None,
    output_path: str | None = None,
    generated_at: str | None = None,
) -> dict[str, Any]:
    generated = generated_at or _utc_timestamp()
    input_count = 0
    skipped_reasons: Counter[str] = Counter()

    if source == "scryfall":
        records = _scryfall_records(payload)
        input_count = len(records)
        cards = _normalize_records(records, lambda record: _normalize_scryfall_card(record), skipped_reasons)
    elif source == "mtgjson":
        records, parent_set_code = _mtgjson_records(payload)
        input_count = len(records)
        cards = _normalize_records(
            records,
            lambda record: _normalize_mtgjson_card(record, parent_set_code),
            skipped_reasons,
        )
    else:
        raise ValueError("source must be one of: scryfall, mtgjson")

    missing_by_field = _missing_high_value_fields(cards)
    return {
        "metadata": {
            "schema_version": SCHEMA_VERSION,
            "generated_at": generated,
            "source": source,
            "input_path": input_path,
            "output_path": output_path,
            "input_count": input_count,
            "normalized_count": len(cards),
            "skipped_count": sum(skipped_reasons.values()),
            "skipped_reasons": dict(sorted(skipped_reasons.items())),
            "missing_high_value_fields_count": sum(missing_by_field.values()),
            "missing_high_value_fields_by_field_name": dict(sorted(missing_by_field.items())),
        },
        "cards": cards,
    }


def format_normalization_report(metadata: Mapping[str, Any]) -> str:
    """Format normalization metadata as deterministic human-readable diagnostics."""

    skipped_reasons = _mapping_value(metadata, "skipped_reasons")
    missing_fields = _mapping_value(metadata, "missing_high_value_fields_by_field_name")
    lines = [
        "Normalization Report",
        f"schema_version: {_display_value(metadata.get('schema_version'))}",
        f"source: {_display_value(metadata.get('source'))}",
        f"input_path: {_display_value(metadata.get('input_path'))}",
        f"output_path: {_display_value(metadata.get('output_path'))}",
        f"generated_at: {_display_value(metadata.get('generated_at'))}",
        f"input_count: {_display_value(metadata.get('input_count'))}",
        f"normalized_count: {_display_value(metadata.get('normalized_count'))}",
        f"skipped_count: {_display_value(metadata.get('skipped_count'))}",
        "skipped_reasons:",
    ]
    lines.extend(_format_mapping_lines(skipped_reasons))
    lines.extend(
        [
            "missing_high_value_fields:",
            f"  total: {_display_value(metadata.get('missing_high_value_fields_count'))}",
        ]
    )
    lines.extend(_format_mapping_lines(missing_fields))
    return "\n".join(lines) + "\n"


def _catalog_metadata(payload: Any) -> Mapping[str, Any]:
    if not isinstance(payload, Mapping):
        raise ValueError("normalized catalog must be a JSON object")
    if not isinstance(payload.get("cards"), list):
        raise ValueError("normalized catalog must include a top-level 'cards' list")
    metadata = payload.get("metadata")
    if metadata is None:
        raise ValueError("normalized catalog is missing embedded metadata")
    if not isinstance(metadata, Mapping):
        raise ValueError("normalized catalog metadata must be a JSON object")
    return metadata


def _scryfall_records(payload: Any) -> list[Mapping[str, Any]]:
    if isinstance(payload, list):
        records = payload
    elif isinstance(payload, dict) and isinstance(payload.get("cards"), list):
        records = payload["cards"]
    else:
        raise ValueError("scryfall JSON must be a list or an object with a 'cards' list")
    return records


def _mtgjson_records(payload: Any) -> tuple[list[Mapping[str, Any]], str | None]:
    if isinstance(payload, list):
        return payload, None
    if isinstance(payload, dict):
        if isinstance(payload.get("cards"), list):
            return payload["cards"], None
        data = payload.get("data")
        if isinstance(data, dict) and isinstance(data.get("cards"), list):
            parent_set_code = _optional_string(data.get("code"))
            return data["cards"], parent_set_code
    raise ValueError("mtgjson JSON must be a list or an object with 'cards' or data.cards")


def _normalize_records(
    records: list[Any],
    normalize_one: Any,
    skipped_reasons: Counter[str],
) -> list[NormalizedCard]:
    cards: list[NormalizedCard] = []
    for record in records:
        if not isinstance(record, Mapping):
            skipped_reasons["record_not_object"] += 1
            continue
        try:
            cards.append(normalize_one(record))
        except ValueError as exc:
            skipped_reasons[str(exc)] += 1
    return cards


def _normalize_scryfall_card(record: Mapping[str, Any]) -> NormalizedCard:
    return _compact_record(
        {
            "name": _required_string(record, "name", "scryfall"),
            "type_line": _string(record.get("type_line")),
            "colors": _string_list(record.get("colors")),
            "color_identity": _string_list(record.get("color_identity")),
            "mana_value": _number(record.get("mana_value", record.get("cmc"))),
            "mana_cost": _string(record.get("mana_cost")),
            "oracle_text": _string(record.get("oracle_text")),
            "rarity": _string(record.get("rarity")),
            "digital": _bool(record.get("digital", False)),
            "games": _string_list(record.get("games")),
            "layout": _string(record.get("layout")),
            "legalities": _string_mapping(record.get("legalities")),
            "set_code": _upper_optional(record.get("set_code", record.get("set"))),
            "collector_number": _optional_string(record.get("collector_number")),
            "scryfall_id": _optional_string(record.get("scryfall_id", record.get("id"))),
            "arena_id": _optional_int(record.get("arena_id")),
        }
    )


def _normalize_mtgjson_card(record: Mapping[str, Any], parent_set_code: str | None) -> NormalizedCard:
    identifiers = record.get("identifiers")
    if not isinstance(identifiers, Mapping):
        identifiers = {}
    arena_id = identifiers.get("mtgoArenaId", identifiers.get("mtgArenaId"))
    set_code = record.get("set_code", record.get("setCode", parent_set_code))
    return _compact_record(
        {
            "name": _required_string(record, "name", "mtgjson"),
            "type_line": _string(record.get("type_line", record.get("typeLine", record.get("type")))),
            "colors": _string_list(record.get("colors")),
            "color_identity": _string_list(record.get("color_identity", record.get("colorIdentity"))),
            "mana_value": _number(record.get("mana_value", record.get("manaValue"))),
            "mana_cost": _string(record.get("mana_cost", record.get("manaCost"))),
            "oracle_text": _string(record.get("oracle_text", record.get("oracleText", record.get("text")))),
            "rarity": _string(record.get("rarity")),
            "digital": _bool(record.get("digital", record.get("isOnlineOnly", False))),
            "games": _string_list(record.get("games")),
            "layout": _string(record.get("layout")),
            "legalities": _string_mapping(record.get("legalities")),
            "set_code": _upper_optional(set_code),
            "collector_number": _optional_string(record.get("collector_number", record.get("number"))),
            "scryfall_id": _optional_string(identifiers.get("scryfallId", record.get("scryfall_id"))),
            "arena_id": _optional_int(record.get("arena_id", arena_id)),
        }
    )


def _compact_record(record: dict[str, Any]) -> NormalizedCard:
    return {key: value for key, value in record.items() if value is not None}


def _required_string(record: Mapping[str, Any], field: str, source: str) -> str:
    value = record.get(field)
    if value is None or str(value) == "":
        raise ValueError(f"{source}_missing_required_{field}")
    return str(value)


def _string(value: Any) -> str:
    return "" if value is None else str(value)


def _optional_string(value: Any) -> str | None:
    if value is None or str(value) == "":
        return None
    return str(value)


def _upper_optional(value: Any) -> str | None:
    text = _optional_string(value)
    return text.upper() if text else None


def _string_list(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if not isinstance(value, list | tuple | set):
        return [str(value)]
    return [str(item) for item in value]


def _string_mapping(value: Any) -> dict[str, str]:
    if not isinstance(value, Mapping):
        return {}
    return {str(key).casefold(): str(item).casefold() for key, item in value.items()}


def _number(value: Any) -> float:
    return float(value or 0.0)


def _optional_int(value: Any) -> int | None:
    if value is None or value == "":
        return None
    try:
        return int(value)
    except (TypeError, ValueError) as exc:
        raise ValueError("invalid_arena_id") from exc


def _bool(value: Any) -> bool:
    if isinstance(value, str):
        return value.casefold() in {"1", "true", "yes"}
    return bool(value)


def _missing_high_value_fields(cards: list[NormalizedCard]) -> dict[str, int]:
    missing: Counter[str] = Counter()
    for card in cards:
        for field in HIGH_VALUE_FIELDS:
            if _is_missing_value(card.get(field)):
                missing[field] += 1
    return dict(missing)


def _is_missing_value(value: Any) -> bool:
    return value is None or value == "" or value == [] or value == {}


def _utc_timestamp() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _mapping_value(metadata: Mapping[str, Any], key: str) -> Mapping[str, Any]:
    value = metadata.get(key, {})
    if not isinstance(value, Mapping):
        return {}
    return value


def _format_mapping_lines(values: Mapping[str, Any]) -> list[str]:
    if not values:
        return ["  none"]
    return [f"  {key}: {values[key]}" for key in sorted(values)]


def _display_value(value: Any) -> str:
    return "None" if value is None else str(value)
