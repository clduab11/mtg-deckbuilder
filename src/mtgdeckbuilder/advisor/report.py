"""Deterministic offline advisor reports inspired by the vawlrathh prototype."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from mtgdeckbuilder.export.arena import export_arena_deck
from mtgdeckbuilder.features.basic import extract_basic_features
from mtgdeckbuilder.ingest.arena import parse_arena_deck
from mtgdeckbuilder.ingest.cards import CardCatalog
from mtgdeckbuilder.rules.validator import ValidationConfig, ValidationIssue, validate_deck


ADVISOR_SCHEMA_VERSION = "advisor-foundation-001.v1"


def advisor_report_file(
    deck_path: str | Path,
    catalog_path: str | Path,
    format_name: str = "standard",
    queue: str = "bo1",
) -> str:
    """Build a deterministic JSON advisor report from local files."""

    catalog = CardCatalog.from_json_file(catalog_path)
    deck_text = Path(deck_path).read_text(encoding="utf-8")
    report = advisor_report(deck_text, catalog=catalog, format_name=format_name, queue=queue)
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def advisor_report(
    deck_text: str,
    catalog: CardCatalog,
    format_name: str = "standard",
    queue: str = "bo1",
) -> dict[str, Any]:
    """Parse, validate, featureize, and summarize one deck without network or AI calls."""

    deck = parse_arena_deck(deck_text)
    validation = validate_deck(
        deck,
        catalog=catalog,
        config=ValidationConfig(format_name=format_name),
    )
    features = extract_basic_features(deck, catalog=catalog)
    valid = validation.is_valid
    return {
        "schema_version": ADVISOR_SCHEMA_VERSION,
        "mode": "offline",
        "reference": "vawlrathh",
        "format": format_name,
        "queue": queue,
        "valid": valid,
        "validation": {
            "issues": [issue.to_dict() for issue in validation.issues],
        },
        "features": features,
        "findings": _findings(valid, validation.issues, features, format_name, queue),
        "arena_export": export_arena_deck(deck) if valid else "",
    }


def _findings(
    valid: bool,
    issues: tuple[ValidationIssue, ...],
    features: dict[str, Any],
    format_name: str,
    queue: str,
) -> list[dict[str, str]]:
    findings = [
        {
            "code": "scope",
            "severity": "info",
            "message": f"Offline advisor report for {format_name} {queue}; no AI or network calls were used.",
        }
    ]
    if valid:
        findings.append(
            {
                "code": "validation_passed",
                "severity": "info",
                "message": "Deck passed deterministic catalog-backed validation.",
            }
        )
    else:
        findings.append(
            {
                "code": "validation_failed",
                "severity": "error",
                "message": f"Deck has {len(issues)} validation diagnostic(s); Arena export was withheld.",
            }
        )

    unknown_cards = features.get("unknown_cards", [])
    if unknown_cards:
        findings.append(
            {
                "code": "unknown_cards_present",
                "severity": "warning",
                "message": f"Unresolved cards: {', '.join(str(card) for card in unknown_cards)}.",
            }
        )
    else:
        findings.append(
            {
                "code": "catalog_resolved",
                "severity": "info",
                "message": "All mainboard cards resolved against the provided catalog.",
            }
        )

    colors = features.get("colors", [])
    color_text = ", ".join(str(color) for color in colors) if colors else "colorless or unresolved"
    findings.append(
        {
            "code": "color_summary",
            "severity": "info",
            "message": f"Detected mainboard colors: {color_text}.",
        }
    )
    return findings
