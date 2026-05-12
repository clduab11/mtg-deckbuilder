"""Deterministic MTG deck validation."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass

from mtgdeckbuilder.export.arena import assert_arena_export_compatible
from mtgdeckbuilder.ingest.arena import Decklist
from mtgdeckbuilder.ingest.cards import CardCatalog, is_basic_land_name, normalize_card_name


@dataclass(frozen=True)
class ValidationConfig:
    min_mainboard_size: int = 60
    max_sideboard_size: int = 15
    max_non_basic_copies: int = 4
    format_name: str = "standard"


@dataclass(frozen=True)
class ValidationIssue:
    code: str
    severity: str
    message: str
    card_name: str | None = None
    section: str | None = None

    def to_dict(self) -> dict[str, str | None]:
        return {
            "code": self.code,
            "severity": self.severity,
            "message": self.message,
            "card_name": self.card_name,
            "section": self.section,
        }


@dataclass(frozen=True)
class ValidationResult:
    issues: tuple[ValidationIssue, ...]

    @property
    def is_valid(self) -> bool:
        return not any(issue.severity == "error" for issue in self.issues)

    @property
    def errors(self) -> tuple[ValidationIssue, ...]:
        return tuple(issue for issue in self.issues if issue.severity == "error")


def validate_deck(
    deck: Decklist,
    catalog: CardCatalog | None = None,
    config: ValidationConfig | None = None,
) -> ValidationResult:
    """Validate offline deck construction and Arena export compatibility."""

    cfg = config or ValidationConfig()
    issues: list[ValidationIssue] = []

    if deck.mainboard_count < cfg.min_mainboard_size:
        issues.append(
            ValidationIssue(
                code="mainboard_too_small",
                severity="error",
                message=(
                    f"Mainboard has {deck.mainboard_count} cards; "
                    f"minimum is {cfg.min_mainboard_size}."
                ),
                section="mainboard",
            )
        )

    if deck.sideboard_count > cfg.max_sideboard_size:
        issues.append(
            ValidationIssue(
                code="sideboard_too_large",
                severity="error",
                message=(
                    f"Sideboard has {deck.sideboard_count} cards; "
                    f"maximum is {cfg.max_sideboard_size}."
                ),
                section="sideboard",
            )
        )

    counts = _combined_counts(deck)
    display_names = _display_names(deck)
    for normalized_name, count in sorted(counts.items()):
        name = display_names[normalized_name]
        if _is_basic_land(name, catalog):
            continue
        if count > cfg.max_non_basic_copies:
            issues.append(
                ValidationIssue(
                    code="too_many_copies",
                    severity="error",
                    message=(
                        f"{name} has {count} total copies across mainboard and sideboard; "
                        f"maximum is {cfg.max_non_basic_copies}."
                    ),
                    card_name=name,
                )
            )

    if catalog is not None:
        for normalized_name, name in sorted(display_names.items()):
            card = catalog.get(name)
            if card is None:
                continue
            legality = card.legalities.get(cfg.format_name)
            if legality and legality.casefold() == "banned":
                issues.append(
                    ValidationIssue(
                        code="banned_card",
                        severity="error",
                        message=f"{name} is banned in {cfg.format_name}.",
                        card_name=name,
                    )
                )

    try:
        assert_arena_export_compatible(deck)
    except ValueError as exc:
        issues.append(
            ValidationIssue(
                code="arena_export_incompatible",
                severity="error",
                message=str(exc),
            )
        )

    return ValidationResult(tuple(issues))


def _combined_counts(deck: Decklist) -> Counter[str]:
    counts: Counter[str] = Counter()
    for entry in deck.entries():
        counts[normalize_card_name(entry.name)] += entry.count
    return counts


def _display_names(deck: Decklist) -> dict[str, str]:
    names: dict[str, str] = {}
    for entry in deck.entries():
        names.setdefault(normalize_card_name(entry.name), entry.name)
    return names


def _is_basic_land(name: str, catalog: CardCatalog | None) -> bool:
    if catalog is not None:
        card = catalog.get(name)
        if card is not None:
            return card.is_basic_land
    return is_basic_land_name(name)
