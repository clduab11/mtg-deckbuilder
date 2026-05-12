"""Smoke evaluator for the foundation milestone."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from mtgdeckbuilder.export.arena import export_arena_deck
from mtgdeckbuilder.features.basic import extract_basic_features
from mtgdeckbuilder.ingest.arena import ArenaParseError, parse_arena_deck
from mtgdeckbuilder.ingest.cards import CardCatalog
from mtgdeckbuilder.rules.validator import ValidationConfig, ValidationIssue, validate_deck


@dataclass(frozen=True)
class SmokeEvalResult:
    valid: bool
    issues: tuple[ValidationIssue, ...]
    features: dict[str, Any]
    arena_export: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "valid": self.valid,
            "issues": [issue.to_dict() for issue in self.issues],
            "features": self.features,
            "arena_export": self.arena_export,
        }


def run_smoke_evaluation(
    deck_text: str,
    catalog: CardCatalog | None = None,
    format_name: str = "standard",
    stop_on_validation_failure: bool = False,
) -> SmokeEvalResult:
    """Parse, validate, featureize, and export one decklist."""

    try:
        deck = parse_arena_deck(deck_text)
    except ArenaParseError as exc:
        issue = ValidationIssue(
            code="parse_error",
            severity="error",
            message=str(exc),
        )
        return SmokeEvalResult(valid=False, issues=(issue,), features={}, arena_export="")

    validation = validate_deck(
        deck,
        catalog=catalog,
        config=ValidationConfig(format_name=format_name),
    )
    if stop_on_validation_failure and not validation.is_valid:
        return SmokeEvalResult(
            valid=False,
            issues=validation.issues,
            features={},
            arena_export="",
        )
    return SmokeEvalResult(
        valid=validation.is_valid,
        issues=validation.issues,
        features=extract_basic_features(deck, catalog=catalog),
        arena_export=export_arena_deck(deck),
    )


def run_smoke_file(
    deck_path: str | Path,
    cards_path: str | Path | None = None,
    catalog_path: str | Path | None = None,
    format_name: str = "standard",
) -> SmokeEvalResult:
    resolved_catalog_path = catalog_path or cards_path
    catalog = CardCatalog.from_json_file(resolved_catalog_path) if resolved_catalog_path else None
    deck_text = Path(deck_path).read_text(encoding="utf-8")
    return run_smoke_evaluation(
        deck_text,
        catalog=catalog,
        format_name=format_name,
        stop_on_validation_failure=catalog_path is not None,
    )
